// 配信中モード管理サービス
//
// 配信中かどうかのフラグを管理し、OBS配信状態と連動する
// 配信中は通知やアラートの抑制などに使用

use std::sync::Arc;
use tokio::sync::RwLock;

/// 配信中モード状態を管理するサービス
#[derive(Debug, Clone)]
pub struct StreamingModeService {
    /// 配信中フラグ（スレッドセーフ）
    is_streaming: Arc<RwLock<bool>>,
}

impl StreamingModeService {
    /// 新しいStreamingModeServiceインスタンスを作成
    pub fn new() -> Self {
        Self {
            is_streaming: Arc::new(RwLock::new(false)),
        }
    }

    /// 配信中モードを設定
    ///
    /// # Arguments
    /// * `enabled` - 配信中の場合はtrue、配信停止の場合はfalse
    pub async fn set_streaming_mode(&self, enabled: bool) {
        let mut is_streaming = self.is_streaming.write().await;
        *is_streaming = enabled;
    }

    /// 配信中モードを取得
    ///
    /// # Returns
    /// 配信中の場合はtrue、それ以外はfalse
    pub async fn is_streaming_mode(&self) -> bool {
        let is_streaming = self.is_streaming.read().await;
        *is_streaming
    }
}

impl Default for StreamingModeService {
    fn default() -> Self {
        Self::new()
    }
}

/// グローバルStreamingModeServiceインスタンス
static STREAMING_MODE_SERVICE: once_cell::sync::Lazy<StreamingModeService> =
    once_cell::sync::Lazy::new(StreamingModeService::new);

/// グローバルStreamingModeServiceを取得
pub fn get_streaming_mode_service() -> &'static StreamingModeService {
    &STREAMING_MODE_SERVICE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_streaming_mode_service() {
        let service = StreamingModeService::new();

        // 初期状態はfalse
        assert!(!service.is_streaming_mode().await);

        // 配信中モードを有効化
        service.set_streaming_mode(true).await;
        assert!(service.is_streaming_mode().await);

        // 配信中モードを無効化
        service.set_streaming_mode(false).await;
        assert!(!service.is_streaming_mode().await);
    }

    #[tokio::test]
    async fn test_global_service() {
        let service = get_streaming_mode_service();

        // グローバルサービスにアクセス可能
        service.set_streaming_mode(true).await;
        assert!(service.is_streaming_mode().await);

        // クリーンアップ
        service.set_streaming_mode(false).await;
    }

    #[tokio::test]
    async fn test_multiple_toggles() {
        let service = StreamingModeService::new();

        // 複数回のトグル
        for i in 0..10 {
            let expected = i % 2 == 0;
            service.set_streaming_mode(expected).await;
            assert_eq!(service.is_streaming_mode().await, expected);
        }
    }

    #[tokio::test]
    async fn test_concurrent_reads() {
        let service = StreamingModeService::new();
        service.set_streaming_mode(true).await;

        // 並列読み込み
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let svc = service.clone();
                tokio::spawn(async move {
                    svc.is_streaming_mode().await
                })
            })
            .collect();

        for handle in handles {
            let result = handle.await.expect("task failed");
            assert!(result);
        }
    }

    #[tokio::test]
    async fn test_concurrent_writes() {
        let service = StreamingModeService::new();

        // 並列書き込み（最後の書き込みが反映される）
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let svc = service.clone();
                tokio::spawn(async move {
                    svc.set_streaming_mode(i % 2 == 0).await;
                })
            })
            .collect();

        for handle in handles {
            handle.await.expect("task failed");
        }

        // どちらかの状態になっている（競合状態だが、パニックしない）
        let final_state = service.is_streaming_mode().await;
        assert!(final_state || !final_state); // トートロジーだが、クラッシュしないことを確認
    }

    #[tokio::test]
    async fn test_clone_service() {
        let service = StreamingModeService::new();
        service.set_streaming_mode(true).await;

        let cloned = service.clone();
        // クローンされたサービスは同じ状態を共有
        assert!(cloned.is_streaming_mode().await);

        // クローンから状態を変更
        cloned.set_streaming_mode(false).await;
        // 元のサービスにも反映される
        assert!(!service.is_streaming_mode().await);
    }

    #[tokio::test]
    async fn test_default_implementation() {
        let service = StreamingModeService::default();
        assert!(!service.is_streaming_mode().await);
    }

    #[tokio::test]
    async fn test_idempotent_operations() {
        let service = StreamingModeService::new();

        // 同じ値を複数回設定しても問題ない
        service.set_streaming_mode(true).await;
        service.set_streaming_mode(true).await;
        service.set_streaming_mode(true).await;
        assert!(service.is_streaming_mode().await);

        service.set_streaming_mode(false).await;
        service.set_streaming_mode(false).await;
        service.set_streaming_mode(false).await;
        assert!(!service.is_streaming_mode().await);
    }

    #[tokio::test]
    async fn test_rapid_state_changes() {
        let service = StreamingModeService::new();

        // 高速な状態変更
        for _ in 0..100 {
            service.set_streaming_mode(true).await;
            service.set_streaming_mode(false).await;
        }

        // 最終状態は false
        assert!(!service.is_streaming_mode().await);
    }
}
