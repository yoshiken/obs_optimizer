// OBSサービス - ビジネスロジック層
//
// OBS WebSocket操作のサービスレイヤー。
// Tauriコマンドから呼び出され、obs::clientモジュールを利用する。
//
// 設計方針:
// - 既存のObsClientをラップし、統一的なAPIを提供
// - エラーハンドリングとバリデーションを一元化
// - 将来的なロギング、メトリクス収集のフックポイントを提供

use crate::error::AppError;
use crate::obs::{
    get_obs_client, ConnectionConfig, ConnectionState, ObsClient, ObsStatus,
};

/// OBSサービスのインスタンス
///
/// `グローバルなObsClientへのアクセスを提供する薄いラッパー`。
/// `ObsClient自体が` Arc<`RwLock`<>> でスレッドセーフなため、
/// このサービスは単にアクセスポイントとして機能する。
#[derive(Clone)]
pub struct ObsService {
    client: ObsClient,
}

impl Default for ObsService {
    fn default() -> Self {
        Self::new()
    }
}

impl ObsService {
    /// `新しいObsServiceインスタンスを作成`
    ///
    /// `内部でグローバルなObsClientを取得する`
    pub fn new() -> Self {
        Self {
            client: get_obs_client(),
        }
    }

    /// OBS `WebSocketサーバーに接続`
    ///
    /// # Arguments
    /// * `config` - 接続設定（ホスト、ポート、パスワード）
    ///
    /// # Returns
    /// 成功時はOk(()), `失敗時はAppError`
    pub async fn connect(&self, config: ConnectionConfig) -> Result<(), AppError> {
        self.client.connect(config).await
    }

    /// OBS `WebSocketサーバーから切断`
    ///
    /// # Returns
    /// 成功時はOk(()), `失敗時はAppError`
    pub async fn disconnect(&self) -> Result<(), AppError> {
        self.client.disconnect().await
    }

    /// 接続されているかどうかを確認
    ///
    /// # Returns
    /// 接続中の場合はtrue、それ以外はfalse
    pub async fn is_connected(&self) -> bool {
        self.client.is_connected().await
    }

    /// 現在の接続状態を取得
    ///
    /// # Returns
    /// ConnectionState（Disconnected, Connecting, Connected, Reconnecting, Error）
    pub async fn connection_state(&self) -> ConnectionState {
        self.client.connection_state().await
    }

    /// ObsClientへの参照を取得（高度な操作用）（将来使用予定）
    ///
    /// 通常はこのサービスのメソッドを使用すべきだが、
    /// 直接クライアントにアクセスする必要がある場合に使用
    ///
    /// # Returns
    /// ObsClientのクローン（内部状態はArcで共有）
    #[allow(dead_code)]
    pub const fn client(&self) -> &ObsClient {
        &self.client
    }

    /// OBSの現在のステータスを取得
    ///
    /// 接続されていない場合は未接続ステータスを返す
    ///
    /// # Returns
    /// OBSステータス（配信状態、録画状態、FPSなど）
    pub async fn get_status(&self) -> Result<ObsStatus, AppError> {
        if !self.is_connected().await {
            return Ok(ObsStatus::disconnected());
        }
        self.client.get_status().await
    }

    /// シーンリストを取得
    ///
    /// # Returns
    /// シーン名の配列
    pub async fn get_scene_list(&self) -> Result<Vec<String>, AppError> {
        self.ensure_connected().await?;
        self.client.get_scene_list().await
    }

    /// 現在のシーンを変更
    ///
    /// # Arguments
    /// * `scene_name` - 切り替え先のシーン名
    pub async fn set_current_scene(&self, scene_name: &str) -> Result<(), AppError> {
        self.ensure_connected().await?;
        self.client.set_current_scene(scene_name).await
    }

    /// 配信を開始
    pub async fn start_streaming(&self) -> Result<(), AppError> {
        self.ensure_connected().await?;
        self.client.start_streaming().await
    }

    /// 配信を停止
    pub async fn stop_streaming(&self) -> Result<(), AppError> {
        self.ensure_connected().await?;
        self.client.stop_streaming().await
    }

    /// 録画を開始
    pub async fn start_recording(&self) -> Result<(), AppError> {
        self.ensure_connected().await?;
        self.client.start_recording().await
    }

    /// 録画を停止
    ///
    /// # Returns
    /// 録画ファイルのパス
    pub async fn stop_recording(&self) -> Result<String, AppError> {
        self.ensure_connected().await?;
        self.client.stop_recording().await
    }

    /// 接続チェックヘルパー
    ///
    /// 接続されていない場合はエラーを返す
    async fn ensure_connected(&self) -> Result<(), AppError> {
        if !self.is_connected().await {
            return Err(AppError::obs_state("OBSに接続されていません"));
        }
        Ok(())
    }
}

/// `グローバルなObsServiceインスタンスを取得`
///
/// `複数回呼び出しても同じObsClientの状態を共有する`
///
/// # Returns
/// `ObsServiceインスタンス`
pub fn obs_service() -> ObsService {
    ObsService::new()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_obs_service_new() {
        let service = ObsService::new();
        assert!(!service.is_connected().await);
    }

    #[tokio::test]
    async fn test_obs_service_global_instance() {
        let service1 = obs_service();
        let service2 = obs_service();

        // 両方とも未接続状態であることを確認
        assert!(!service1.is_connected().await);
        assert!(!service2.is_connected().await);
    }

    #[tokio::test]
    async fn test_get_status_when_not_connected() {
        let service = obs_service();
        let status = service.get_status().await.unwrap();
        assert!(!status.connected);
    }

    #[tokio::test]
    async fn test_ensure_connected_fails_when_not_connected() {
        let service = obs_service();
        let result = service.get_scene_list().await;
        assert!(result.is_err());
    }
}
