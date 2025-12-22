// OBS WebSocketクライアント実装
//
// obwsクレートを使用してOBS WebSocketサーバーと通信する

use obws::client::ConnectConfig;
use obws::Client;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::error::AppError;
use super::error::ObsResult;
use super::types::{ConnectionConfig as AppConnectionConfig, ConnectionState, ObsStatus, ReconnectConfig};

/// ビットレート計算用の統計情報
#[derive(Debug, Clone)]
struct BitrateStats {
    /// 前回のストリーム送信バイト数
    last_stream_bytes: u64,
    /// 前回の録画書き込みバイト数
    last_record_bytes: u64,
    /// 前回のサンプリング時刻
    last_sample_time: Option<Instant>,
}

impl Default for BitrateStats {
    fn default() -> Self {
        Self {
            last_stream_bytes: 0,
            last_record_bytes: 0,
            last_sample_time: None,
        }
    }
}

/// 最小サンプリング間隔（秒）- ノイズ防止のため
const MIN_BITRATE_SAMPLE_INTERVAL_SECS: f64 = 0.1;

impl BitrateStats {
    /// ストリームビットレートを差分計算 (kbps)
    ///
    /// # 差分計算の仕様
    /// - 初回呼び出し: `None` を返し、基準値を保存
    /// - 2回目以降: 前回からの差分でビットレートを計算
    /// - 100ms未満の経過時: `None` を返し、統計を更新しない（ノイズ防止）
    fn calculate_stream_bitrate(&mut self, current_bytes: u64) -> Option<u32> {
        let now = Instant::now();

        if let Some(last_time) = self.last_sample_time {
            let elapsed_secs = last_time.elapsed().as_secs_f64();

            // 最低100ms経過していないと計算しない（ノイズ防止）
            if elapsed_secs >= MIN_BITRATE_SAMPLE_INTERVAL_SECS {
                let diff_bytes = current_bytes.saturating_sub(self.last_stream_bytes);
                // bytes -> kbps: diff_bytes * 8 (bits) / elapsed_secs / 1000
                let kbps = (diff_bytes as f64 * 8.0 / elapsed_secs / 1000.0) as u32;

                // 計算成功時のみ値を更新
                self.last_stream_bytes = current_bytes;
                self.last_sample_time = Some(now);

                Some(kbps)
            } else {
                // 時間不足: 統計を更新せず None を返す
                None
            }
        } else {
            // 初回サンプリング: 次回計算のために基準値を保存
            self.last_stream_bytes = current_bytes;
            self.last_sample_time = Some(now);
            None
        }
    }

    /// 統計情報をリセット
    fn reset(&mut self) {
        self.last_stream_bytes = 0;
        self.last_record_bytes = 0;
        self.last_sample_time = None;
    }
}

/// OBSクライアントの内部状態
struct ObsClientInner {
    /// obwsクライアントインスタンス
    client: Option<Client>,
    /// 現在の接続設定
    config: Option<AppConnectionConfig>,
    /// 再接続設定
    reconnect_config: ReconnectConfig,
    /// 現在の接続状態
    connection_state: ConnectionState,
    /// 再接続試行回数
    reconnect_attempts: u32,
    /// ビットレート計算用統計
    bitrate_stats: BitrateStats,
}

impl ObsClientInner {
    fn new() -> Self {
        Self {
            client: None,
            config: None,
            reconnect_config: ReconnectConfig::default(),
            connection_state: ConnectionState::Disconnected,
            reconnect_attempts: 0,
            bitrate_stats: BitrateStats::default(),
        }
    }
}

/// スレッドセーフなOBS WebSocketクライアント
///
/// 内部状態はRwLockで保護されており、複数のタスクから安全にアクセス可能
#[derive(Clone)]
pub struct ObsClient {
    inner: Arc<RwLock<ObsClientInner>>,
}

impl Default for ObsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ObsClient {
    /// 新しいObsClientインスタンスを作成
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ObsClientInner::new())),
        }
    }

    /// 再接続設定を更新
    pub async fn set_reconnect_config(&self, config: ReconnectConfig) {
        let mut inner = self.inner.write().await;
        inner.reconnect_config = config;
    }

    /// OBS WebSocketサーバーに接続
    ///
    /// # Arguments
    /// * `config` - 接続設定
    ///
    /// # Returns
    /// 接続成功時はOk(()), 失敗時はエラー
    pub async fn connect(&self, config: AppConnectionConfig) -> ObsResult<()> {
        // バリデーション（エラーメッセージは最大100文字に制限してログ肥大化を防止）
        config.validate().map_err(|e| {
            let msg = e.chars().take(100).collect::<String>();
            let msg = if e.len() > 100 {
                format!("{}...", msg)
            } else {
                msg
            };
            AppError::obs_connection(&msg)
        })?;

        // 状態を接続中に更新
        {
            let mut inner = self.inner.write().await;
            inner.connection_state = ConnectionState::Connecting;
            inner.config = Some(config.clone());
        }

        // obws ConnectConfigを構築
        let connect_config = ConnectConfig {
            host: config.host.clone(),
            port: config.port,
            password: config.password.clone(),
            event_subscriptions: None,
            broadcast_capacity: obws::client::DEFAULT_BROADCAST_CAPACITY,
            connect_timeout: obws::client::DEFAULT_CONNECT_TIMEOUT,
            dangerous: None,
            #[cfg(feature = "tls")]
            tls: false,
        };

        // obwsクライアントを作成して接続
        let client_result = Client::connect_with_config(connect_config).await;

        match client_result {
            Ok(client) => {
                let mut inner = self.inner.write().await;
                inner.client = Some(client);
                inner.connection_state = ConnectionState::Connected;
                inner.reconnect_attempts = 0;
                inner.bitrate_stats.reset(); // 新規接続時は統計をリセット
                Ok(())
            }
            Err(e) => {
                let mut inner = self.inner.write().await;
                inner.connection_state = ConnectionState::Error;
                Err(AppError::from(e))
            }
        }
    }

    /// OBS WebSocketサーバーから切断
    pub async fn disconnect(&self) -> ObsResult<()> {
        let mut inner = self.inner.write().await;

        // クライアントを破棄することで接続を切断
        inner.client = None;
        inner.connection_state = ConnectionState::Disconnected;
        inner.reconnect_attempts = 0;
        inner.bitrate_stats.reset(); // 統計もリセット

        Ok(())
    }

    /// 接続されているかどうかを確認
    pub async fn is_connected(&self) -> bool {
        let inner = self.inner.read().await;
        inner.client.is_some() && inner.connection_state == ConnectionState::Connected
    }

    /// 現在の接続状態を取得
    pub async fn connection_state(&self) -> ConnectionState {
        let inner = self.inner.read().await;
        inner.connection_state
    }

    /// OBSの現在のステータスを取得
    ///
    /// ビットレートは差分計算で算出される（前回取得時との差分から実際の転送速度を計算）
    pub async fn get_status(&self) -> ObsResult<ObsStatus> {
        // ビットレート統計更新のため書き込みロックを使用
        let mut inner = self.inner.write().await;

        let client = inner.client.as_ref().ok_or_else(|| {
            AppError::obs_state("OBSに接続されていません")
        })?;

        // OBSから各種情報を取得
        let version_info = client.general().version().await?;
        let stream_status = client.streaming().status().await.ok();
        let record_status = client.recording().status().await.ok();
        let virtual_cam_status = client.virtual_cam().status().await.ok();
        let current_scene = client.scenes().current_program_scene().await.ok();

        // 統計情報を取得
        let stats = client.general().stats().await.ok();

        // ビットレートを差分計算
        let stream_bitrate = if let Some(ref stream) = stream_status {
            if stream.active {
                // 配信中: 累積バイト数から差分計算でビットレートを算出
                inner.bitrate_stats.calculate_stream_bitrate(stream.bytes)
            } else {
                // 配信停止時は統計をリセット
                inner.bitrate_stats.reset();
                None
            }
        } else {
            None
        };

        let status = ObsStatus {
            connected: true,
            streaming: stream_status.as_ref().map(|s| s.active).unwrap_or(false),
            recording: record_status.as_ref().map(|r| r.active).unwrap_or(false),
            virtual_cam_active: virtual_cam_status.unwrap_or(false),
            current_scene: current_scene.map(|s| s.id.name),
            obs_version: Some(version_info.obs_version.to_string()),
            websocket_version: Some(version_info.obs_web_socket_version.to_string()),
            stream_timecode: None,
            record_timecode: None,
            stream_bitrate,
            record_bitrate: None,
            fps: stats.as_ref().map(|s| s.active_fps),
            render_dropped_frames: stats.as_ref().map(|s| s.render_skipped_frames),
            output_dropped_frames: stats.as_ref().map(|s| s.output_skipped_frames),
        };

        Ok(status)
    }

    /// ステータスを更新して返す (refresh_statusのエイリアス)
    pub async fn refresh_status(&self) -> ObsResult<ObsStatus> {
        self.get_status().await
    }

    /// 現在のシーンリストを取得
    pub async fn get_scene_list(&self) -> ObsResult<Vec<String>> {
        let inner = self.inner.read().await;

        let client = inner.client.as_ref().ok_or_else(|| {
            AppError::obs_state("OBSに接続されていません")
        })?;

        let scenes = client.scenes().list().await?;
        Ok(scenes.scenes.into_iter().map(|s| s.id.name).collect())
    }

    /// シーンを切り替え
    pub async fn set_current_scene(&self, scene_name: &str) -> ObsResult<()> {
        let inner = self.inner.read().await;

        let client = inner.client.as_ref().ok_or_else(|| {
            AppError::obs_state("OBSに接続されていません")
        })?;

        client.scenes().set_current_program_scene(scene_name).await?;
        Ok(())
    }

    /// 配信を開始
    pub async fn start_streaming(&self) -> ObsResult<()> {
        let inner = self.inner.read().await;

        let client = inner.client.as_ref().ok_or_else(|| {
            AppError::obs_state("OBSに接続されていません")
        })?;

        client.streaming().start().await?;
        Ok(())
    }

    /// 配信を停止
    pub async fn stop_streaming(&self) -> ObsResult<()> {
        let inner = self.inner.read().await;

        let client = inner.client.as_ref().ok_or_else(|| {
            AppError::obs_state("OBSに接続されていません")
        })?;

        client.streaming().stop().await?;
        Ok(())
    }

    /// 録画を開始
    pub async fn start_recording(&self) -> ObsResult<()> {
        let inner = self.inner.read().await;

        let client = inner.client.as_ref().ok_or_else(|| {
            AppError::obs_state("OBSに接続されていません")
        })?;

        client.recording().start().await?;
        Ok(())
    }

    /// 録画を停止
    ///
    /// # Returns
    /// 録画ファイルのパスを返す
    pub async fn stop_recording(&self) -> ObsResult<String> {
        let inner = self.inner.read().await;

        let client = inner.client.as_ref().ok_or_else(|| {
            AppError::obs_state("OBSに接続されていません")
        })?;

        let path = client.recording().stop().await?;
        Ok(path)
    }

    /// 再接続を試行（シングルショット）
    ///
    /// 保存された設定を使用して単一の再接続試行を行う
    /// バックグラウンドでの自動再接続には `ReconnectManager` を使用すること
    pub async fn reconnect(&self) -> ObsResult<()> {
        // 書き込みロックで設定取得と試行回数インクリメントを原子的に実行
        // (レース条件を防止)
        let (config, reconnect_config, current_attempt) = {
            let mut inner = self.inner.write().await;

            let config = inner.config.clone();
            let reconnect_config = inner.reconnect_config.clone();
            let attempts = inner.reconnect_attempts;

            // should_retry() で再試行可否を判定
            if !reconnect_config.should_retry(attempts) {
                return if !reconnect_config.enabled {
                    Err(AppError::obs_state("自動再接続が無効です"))
                } else {
                    Err(AppError::obs_connection("再接続の試行回数が上限に達しました"))
                };
            }

            // 状態を原子的に更新
            inner.connection_state = ConnectionState::Reconnecting;
            inner.reconnect_attempts = attempts.saturating_add(1);

            (config, reconnect_config, attempts)
        };

        let config = config.ok_or_else(|| {
            AppError::obs_state("接続設定がありません")
        })?;

        // 待機時間を計算（ロック解放後）
        let delay = reconnect_config.calculate_delay(current_attempt);
        if delay > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }

        // 再接続を試行
        self.connect(config).await
    }

    /// 現在の接続設定を取得
    pub async fn get_config(&self) -> Option<AppConnectionConfig> {
        let inner = self.inner.read().await;
        inner.config.clone()
    }

    /// 再接続設定を取得
    pub async fn get_reconnect_config(&self) -> ReconnectConfig {
        let inner = self.inner.read().await;
        inner.reconnect_config.clone()
    }

    /// 再接続試行回数をリセット
    pub async fn reset_reconnect_attempts(&self) {
        let mut inner = self.inner.write().await;
        inner.reconnect_attempts = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_obs_client_new() {
        let client = ObsClient::new();
        assert!(!client.is_connected().await);
        assert_eq!(client.connection_state().await, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_obs_client_disconnect_when_not_connected() {
        let client = ObsClient::new();
        let result = client.disconnect().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_obs_client_get_status_when_not_connected() {
        let client = ObsClient::new();
        let result = client.get_status().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_reconnect_config() {
        let client = ObsClient::new();
        let config = ReconnectConfig {
            enabled: false,
            max_attempts: 10,
            ..Default::default()
        };
        client.set_reconnect_config(config).await;
        // 設定が反映されているか確認（内部状態なので直接確認は難しい）
    }
}
