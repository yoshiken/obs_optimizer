// メトリクス履歴管理
//
// システムメトリクスとOBS統計情報の履歴を保存・取得する
// SQLiteを使用した永続化

use crate::error::AppError;
use crate::monitor::{GpuMetrics, NetworkMetrics};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 履歴メトリクス（保存用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalMetrics {
    /// タイムスタンプ（UNIX epoch秒）
    pub timestamp: i64,
    /// セッションID
    pub session_id: String,
    /// システムメトリクス
    pub system: SystemMetricsSnapshot,
    /// OBSステータススナップショット
    pub obs: ObsStatusSnapshot,
}

/// システムメトリクスのスナップショット
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMetricsSnapshot {
    /// CPU使用率（%）
    pub cpu_usage: f32,
    /// メモリ使用量（バイト）
    pub memory_used: u64,
    /// メモリ総容量（バイト）
    pub memory_total: u64,
    /// GPU使用率（%）
    pub gpu_usage: Option<f32>,
    /// GPU メモリ使用量（バイト）
    pub gpu_memory_used: Option<u64>,
    /// アップロード速度（バイト/秒）
    pub network_upload: u64,
    /// ダウンロード速度（バイト/秒）
    pub network_download: u64,
}

/// OBSステータスのスナップショット
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsStatusSnapshot {
    /// 配信中かどうか
    pub streaming: bool,
    /// 録画中かどうか
    pub recording: bool,
    /// FPS
    pub fps: Option<f32>,
    /// レンダリングドロップフレーム
    pub render_dropped_frames: Option<u64>,
    /// 出力ドロップフレーム
    pub output_dropped_frames: Option<u64>,
    /// 配信ビットレート（kbps）
    pub stream_bitrate: Option<u64>,
}

/// セッションサマリー（統計情報）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    /// セッションID
    pub session_id: String,
    /// 開始時刻（UNIX epoch秒）
    pub start_time: i64,
    /// 終了時刻（UNIX epoch秒）
    pub end_time: i64,
    /// 平均CPU使用率（%）
    pub avg_cpu: f64,
    /// 平均GPU使用率（%）
    pub avg_gpu: f64,
    /// トータルドロップフレーム数
    pub total_dropped_frames: u64,
    /// ピークビットレート（kbps）
    pub peak_bitrate: u64,
    /// 品質スコア（0-100）
    pub quality_score: f64,
}

/// メトリクス履歴ストア（将来のSQLite永続化で使用予定）
#[allow(dead_code)]
pub struct MetricsHistoryStore {
    /// データベースファイルパス
    db_path: PathBuf,
    /// 現在のセッションID
    current_session_id: Arc<Mutex<Option<String>>>,
}

#[allow(dead_code)]
impl MetricsHistoryStore {
    /// 新しいストアを作成
    ///
    /// # Arguments
    /// * `db_path` - データベースファイルのパス
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            current_session_id: Arc::new(Mutex::new(None)),
        }
    }

    /// データベースを初期化
    ///
    /// テーブルが存在しない場合は作成する
    pub async fn initialize(&self) -> Result<(), AppError> {
        // 現時点ではファイルシステムベースの実装
        // 将来的にSQLite統合時に実装を追加

        // データベースディレクトリを作成
        if let Some(parent) = self.db_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::database_error(&format!("Failed to create database directory: {e}")))?;
        }

        Ok(())
    }

    /// 新しいセッションを開始
    ///
    /// # Returns
    /// セッションID
    pub async fn start_session(&self) -> Result<String, AppError> {
        let session_id = format!("session_{}", chrono::Utc::now().timestamp());
        let mut current = self.current_session_id.lock().await;
        *current = Some(session_id.clone());
        Ok(session_id)
    }

    /// 現在のセッションを終了
    pub async fn end_session(&self) -> Result<(), AppError> {
        let mut current = self.current_session_id.lock().await;
        *current = None;
        Ok(())
    }

    /// メトリクスを保存
    ///
    /// # Arguments
    /// * `system` - システムメトリクス
    /// * `obs` - OBSステータス
    pub async fn save_metrics(
        &self,
        system: SystemMetricsSnapshot,
        obs: ObsStatusSnapshot,
    ) -> Result<(), AppError> {
        let session_id = {
            let current = self.current_session_id.lock().await;
            current.clone().unwrap_or_else(|| "default".to_string())
        };

        let metrics = HistoricalMetrics {
            timestamp: chrono::Utc::now().timestamp(),
            session_id,
            system,
            obs,
        };

        // TODO: SQLite実装後、ここでデータベースに保存
        // 現在はメモリ内のみで保持（Phase 2b初期実装）

        // デバッグログ（将来的にはtracingクレート導入予定）
        #[cfg(debug_assertions)]
        eprintln!(
            "[DEBUG] Saved metrics: cpu={:.1}%, mem={} MB, fps={:?}",
            metrics.system.cpu_usage,
            metrics.system.memory_used / 1024 / 1024,
            metrics.obs.fps
        );

        // metricsは将来使用予定
        let _ = metrics;

        Ok(())
    }

    /// 指定期間のメトリクスを取得
    ///
    /// # Arguments
    /// * `from` - 開始時刻（UNIX epoch秒）
    /// * `to` - 終了時刻（UNIX epoch秒）
    #[allow(clippy::unused_async)]
    pub async fn get_metrics_range(
        &self,
        _from: i64,
        _to: i64,
    ) -> Result<Vec<HistoricalMetrics>, AppError> {
        // TODO: SQLite実装後、データベースから取得
        // 現在は空のベクタを返す
        Ok(Vec::new())
    }

    /// セッションサマリーを取得
    ///
    /// # Arguments
    /// * `session_id` - セッションID
    #[allow(clippy::unused_async)]
    pub async fn get_session_summary(&self, session_id: &str) -> Result<SessionSummary, AppError> {
        // TODO: SQLite実装後、データベースから計算
        // 現在はダミーデータを返す
        Ok(SessionSummary {
            session_id: session_id.to_string(),
            start_time: chrono::Utc::now().timestamp() - 3600,
            end_time: chrono::Utc::now().timestamp(),
            avg_cpu: 45.0,
            avg_gpu: 60.0,
            total_dropped_frames: 0,
            peak_bitrate: 6000,
            quality_score: 85.0,
        })
    }

    /// 全セッションの一覧を取得
    #[allow(clippy::unused_async)]
    pub async fn get_all_sessions(&self) -> Result<Vec<String>, AppError> {
        // TODO: SQLite実装後、データベースから取得
        Ok(Vec::new())
    }
}

/// SystemMetricsSnapshotを作成するヘルパー
impl SystemMetricsSnapshot {
    /// システムメトリクスから作成
    pub fn from_metrics(
        cpu_usage: f32,
        memory_used: u64,
        memory_total: u64,
        gpu: Option<&GpuMetrics>,
        network: &NetworkMetrics,
    ) -> Self {
        Self {
            cpu_usage,
            memory_used,
            memory_total,
            gpu_usage: gpu.map(|g| g.usage_percent),
            gpu_memory_used: gpu.map(|g| g.memory_used_bytes),
            network_upload: network.upload_bytes_per_sec,
            network_download: network.download_bytes_per_sec,
        }
    }
}

/// ObsStatusSnapshotを作成するヘルパー（将来のOBS状態追跡で使用予定）
#[allow(dead_code)]
impl ObsStatusSnapshot {
    /// 空のスナップショットを作成
    pub fn empty() -> Self {
        Self {
            streaming: false,
            recording: false,
            fps: None,
            render_dropped_frames: None,
            output_dropped_frames: None,
            stream_bitrate: None,
        }
    }

    /// OBS接続時のスナップショットを作成
    pub fn from_obs_status(
        streaming: bool,
        recording: bool,
        fps: Option<f32>,
        render_dropped: Option<u64>,
        output_dropped: Option<u64>,
        bitrate: Option<u64>,
    ) -> Self {
        Self {
            streaming,
            recording,
            fps,
            render_dropped_frames: render_dropped,
            output_dropped_frames: output_dropped,
            stream_bitrate: bitrate,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_creation() {
        let store = MetricsHistoryStore::new(PathBuf::from("/tmp/test_metrics.db"));
        assert!(store.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_session_management() {
        let store = MetricsHistoryStore::new(PathBuf::from("/tmp/test_metrics.db"));
        store.initialize().await.unwrap();

        let session_id = store.start_session().await.unwrap();
        assert!(session_id.starts_with("session_"));

        store.end_session().await.unwrap();
    }

    #[tokio::test]
    async fn test_save_metrics() {
        let store = MetricsHistoryStore::new(PathBuf::from("/tmp/test_metrics.db"));
        store.initialize().await.unwrap();
        store.start_session().await.unwrap();

        let system = SystemMetricsSnapshot {
            cpu_usage: 50.0,
            memory_used: 8_000_000_000,
            memory_total: 16_000_000_000,
            gpu_usage: Some(60.0),
            gpu_memory_used: Some(4_000_000_000),
            network_upload: 1_000_000,
            network_download: 500_000,
        };

        let obs = ObsStatusSnapshot::empty();

        assert!(store.save_metrics(system, obs).await.is_ok());
    }
}
