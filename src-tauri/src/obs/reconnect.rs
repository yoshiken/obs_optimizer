// 自動再接続ロジック
//
// バックグラウンドで接続断を検出し、自動的に再接続を試行する
// requirements_v2.md 仕様:
// - 初回失敗: 即座に再試行
// - 1回目: 1秒後、2回目: 2秒後、3回目: 4秒後、4回目: 8秒後
// - 5回目以降: 30秒間隔
// - 最大試行: 無制限（手動停止まで）
//
// 注意: このモジュールは将来的な自動再接続機能の実装用です
// 現在は未使用ですが、設計済みのため保持しています

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{watch, RwLock};

use super::client::ObsClient;
use super::types::ConnectionConfig;

/// 再接続タスクの状態（将来使用予定）
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconnectTaskState {
    /// アイドル状態（再接続タスク未起動）
    Idle,
    /// 待機中（次回試行まで待機）
    Waiting,
    /// 再接続試行中
    Attempting,
    /// 成功して終了
    Succeeded,
    /// キャンセルされて終了
    Cancelled,
}

/// 再接続タスクのハンドル（将来使用予定）
///
/// このハンドルを保持することで、バックグラウンドの再接続タスクを制御可能
#[allow(dead_code)]
#[derive(Clone)]
pub struct ReconnectHandle {
    /// キャンセル送信チャネル
    cancel_tx: watch::Sender<bool>,
    /// 状態監視チャネル
    state_rx: watch::Receiver<ReconnectTaskState>,
}

#[allow(dead_code)]
impl ReconnectHandle {
    /// 再接続タスクをキャンセル
    pub fn cancel(&self) {
        let _ = self.cancel_tx.send(true);
    }

    /// 現在の状態を取得
    pub fn state(&self) -> ReconnectTaskState {
        *self.state_rx.borrow()
    }

    /// タスクが終了したかどうかを確認
    pub fn is_finished(&self) -> bool {
        matches!(
            self.state(),
            ReconnectTaskState::Succeeded | ReconnectTaskState::Cancelled
        )
    }
}

/// 自動再接続マネージャー（将来使用予定）
///
/// 再接続タスクのライフサイクルを管理する
#[allow(dead_code)]
pub struct ReconnectManager {
    /// 現在のハンドル（タスク実行中の場合）
    current_handle: Arc<RwLock<Option<ReconnectHandle>>>,
}

impl Default for ReconnectManager {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl ReconnectManager {
    /// 新しいマネージャーを作成
    pub fn new() -> Self {
        Self {
            current_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// 自動再接続を開始
    ///
    /// 既に再接続タスクが実行中の場合は、それをキャンセルして新しいタスクを開始
    ///
    /// # Arguments
    /// * `client` - OBSクライアント
    /// * `config` - 接続設定
    pub async fn start(&self, client: ObsClient, config: ConnectionConfig) -> ReconnectHandle {
        // 既存タスクをキャンセル
        self.stop().await;

        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (state_tx, state_rx) = watch::channel(ReconnectTaskState::Idle);

        let handle = ReconnectHandle {
            cancel_tx,
            state_rx,
        };

        // ハンドルを保存
        {
            let mut current = self.current_handle.write().await;
            *current = Some(handle.clone());
        }

        // バックグラウンドタスクを起動
        tokio::spawn(reconnect_task(client, config, cancel_rx, state_tx));

        handle
    }

    /// 再接続タスクを停止
    pub async fn stop(&self) {
        let mut current = self.current_handle.write().await;
        if let Some(handle) = current.take() {
            handle.cancel();
        }
    }

    /// 現在のハンドルを取得（存在する場合）
    pub async fn current_handle(&self) -> Option<ReconnectHandle> {
        let current = self.current_handle.read().await;
        current.clone()
    }
}

/// バックグラウンド再接続タスク
async fn reconnect_task(
    client: ObsClient,
    config: ConnectionConfig,
    mut cancel_rx: watch::Receiver<bool>,
    state_tx: watch::Sender<ReconnectTaskState>,
) {
    let mut attempt = 0u32;

    loop {
        // キャンセルチェック
        if *cancel_rx.borrow() {
            let _ = state_tx.send(ReconnectTaskState::Cancelled);
            return;
        }

        // 再接続設定を取得（クライアントから最新設定を取得）
        let reconnect_config = client.get_reconnect_config().await;

        // 再試行可否をチェック
        if !reconnect_config.should_retry(attempt) {
            let _ = state_tx.send(ReconnectTaskState::Cancelled);
            return;
        }

        // 待機時間を計算
        let delay_ms = reconnect_config.calculate_delay(attempt);
        if delay_ms > 0 {
            let _ = state_tx.send(ReconnectTaskState::Waiting);

            // キャンセル可能な待機
            tokio::select! {
                () = tokio::time::sleep(Duration::from_millis(delay_ms)) => {}
                _ = cancel_rx.changed() => {
                    let _ = state_tx.send(ReconnectTaskState::Cancelled);
                    return;
                }
            }
        }

        // 再接続試行
        let _ = state_tx.send(ReconnectTaskState::Attempting);

        match client.connect(config.clone()).await {
            Ok(()) => {
                // 接続成功、試行回数をリセット
                client.reset_reconnect_attempts().await;
                let _ = state_tx.send(ReconnectTaskState::Succeeded);
                return;
            }
            Err(e) => {
                // 接続失敗、ログ出力（将来的にはイベント通知）
                tracing::warn!(
                    target: "obs_reconnect",
                    attempt = attempt.saturating_add(1),
                    error = %e.message(),
                    "Reconnect attempt failed"
                );
                // オーバーフロー防止: saturating_add を使用
                attempt = attempt.saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnect_task_state() {
        assert_eq!(ReconnectTaskState::Idle, ReconnectTaskState::Idle);
        assert_ne!(ReconnectTaskState::Idle, ReconnectTaskState::Waiting);
    }

    #[tokio::test]
    async fn test_reconnect_manager_new() {
        let manager = ReconnectManager::new();
        let handle = manager.current_handle().await;
        assert!(handle.is_none());
    }

    #[tokio::test]
    async fn test_reconnect_handle_is_finished() {
        let (cancel_tx, _) = watch::channel(false);
        let (state_tx, state_rx) = watch::channel(ReconnectTaskState::Idle);

        let handle = ReconnectHandle {
            cancel_tx,
            state_rx,
        };

        assert!(!handle.is_finished());

        let _ = state_tx.send(ReconnectTaskState::Succeeded);
        assert!(handle.is_finished());
    }
}
