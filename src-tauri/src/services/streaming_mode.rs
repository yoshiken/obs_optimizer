// 配信中モード管理サービス
//
// 配信中かどうかのフラグを管理し、OBS配信状態と連動する
// 配信中は通知やアラートの抑制などに使用
//
// TOCTOU対策:
// 設定適用時は acquire_settings_lock() でロックを取得することで、
// ロック保持中は配信状態の変更をブロックし、一貫した操作を保証する。

use crate::error::AppError;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, OwnedMutexGuard, RwLock};

/// 設定操作時のロックタイムアウト（デフォルト30秒）
const SETTINGS_LOCK_TIMEOUT_SECS: u64 = 30;

/// 配信中モード状態を管理するサービス
#[derive(Debug, Clone)]
pub struct StreamingModeService {
    /// 配信中フラグ（スレッドセーフ）
    is_streaming: Arc<RwLock<bool>>,
    /// 設定変更ロック（TOCTOU対策）
    /// このロックを保持している間は配信状態の変更がブロックされる
    settings_lock: Arc<Mutex<()>>,
}

/// 設定変更ロックガード
///
/// このガードがドロップされるまで、配信状態の変更はブロックされる。
/// 設定変更操作はこのガードを保持している間のみ行うべき。
pub struct SettingsLockGuard {
    /// ロックガード（ドロップ時に自動解放）
    _guard: OwnedMutexGuard<()>,
    /// 配信中フラグへの参照
    is_streaming: Arc<RwLock<bool>>,
}

impl SettingsLockGuard {
    /// ロック保持中に配信状態をチェック
    ///
    /// ロックを保持している間は配信状態が変わらないことが保証される
    pub async fn is_streaming(&self) -> bool {
        let is_streaming = self.is_streaming.read().await;
        *is_streaming
    }

    /// 配信中でないことを確認
    ///
    /// 配信中の場合はエラーを返す
    pub async fn ensure_not_streaming(&self) -> Result<(), AppError> {
        if self.is_streaming().await {
            return Err(AppError::obs_state(
                "配信中のため設定を変更できません。配信を停止してから再度お試しください。",
            ));
        }
        Ok(())
    }
}

impl StreamingModeService {
    /// 新しいStreamingModeServiceインスタンスを作成
    pub fn new() -> Self {
        Self {
            is_streaming: Arc::new(RwLock::new(false)),
            settings_lock: Arc::new(Mutex::new(())),
        }
    }

    /// 配信中モードを設定（ロック待機あり）
    ///
    /// 設定変更操作がロックを保持している場合は待機する。
    /// これにより、設定変更操作中に配信状態が変わることを防ぐ。
    ///
    /// # Arguments
    /// * `enabled` - 配信中の場合はtrue、配信停止の場合はfalse
    pub async fn set_streaming_mode(&self, enabled: bool) {
        // 設定変更ロックを取得（設定操作中は待機）
        let _lock = self.settings_lock.lock().await;
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

    /// 設定変更ロックを取得（タイムアウト付き）
    ///
    /// このロックを保持している間は、配信状態の変更がブロックされる。
    /// TOCTOU競合条件を防ぐために、設定を変更する前にこのロックを取得すること。
    ///
    /// # Returns
    /// 成功時はSettingsLockGuard、タイムアウト時はエラー
    ///
    /// # Example
    /// ```ignore
    /// let guard = service.acquire_settings_lock().await?;
    /// guard.ensure_not_streaming().await?;
    /// // ここで設定を適用（配信状態は変わらないことが保証される）
    /// apply_settings().await?;
    /// // guardがドロップされるとロックが解放される
    /// ```
    pub async fn acquire_settings_lock(&self) -> Result<SettingsLockGuard, AppError> {
        self.acquire_settings_lock_with_timeout(Duration::from_secs(SETTINGS_LOCK_TIMEOUT_SECS))
            .await
    }

    /// 設定変更ロックを取得（カスタムタイムアウト）
    ///
    /// # Arguments
    /// * `timeout` - ロック取得のタイムアウト時間
    pub async fn acquire_settings_lock_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<SettingsLockGuard, AppError> {
        let lock_result =
            tokio::time::timeout(timeout, self.settings_lock.clone().lock_owned()).await;

        match lock_result {
            Ok(guard) => {
                tracing::debug!("設定変更ロックを取得しました");
                Ok(SettingsLockGuard {
                    _guard: guard,
                    is_streaming: self.is_streaming.clone(),
                })
            },
            Err(_) => {
                tracing::warn!(
                    "設定変更ロックの取得がタイムアウトしました（{}秒）",
                    timeout.as_secs()
                );
                Err(AppError::obs_state(
                    "設定変更ロックの取得がタイムアウトしました。他の操作が進行中の可能性があります。"
                ))
            },
        }
    }

    /// 配信中でない場合にのみ操作を実行
    ///
    /// 内部的にロックを取得し、配信中でないことを確認してから操作を実行する。
    /// 操作完了まで配信状態の変更はブロックされる。
    ///
    /// # Arguments
    /// * `operation` - 実行する非同期操作
    ///
    /// # Returns
    /// 操作の結果。配信中の場合はエラー。
    ///
    /// # Example
    /// ```ignore
    /// let result = service.execute_if_not_streaming(|| async {
    ///     apply_video_settings(1920, 1080, 60).await
    /// }).await?;
    /// ```
    pub async fn execute_if_not_streaming<F, Fut, T>(&self, operation: F) -> Result<T, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, AppError>>,
    {
        let guard = self.acquire_settings_lock().await?;
        guard.ensure_not_streaming().await?;

        tracing::info!("配信中でないことを確認、設定操作を実行します");
        let result = operation().await;

        if result.is_ok() {
            tracing::info!("設定操作が正常に完了しました");
        } else {
            tracing::warn!("設定操作がエラーで終了しました");
        }

        result
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
                tokio::spawn(async move { svc.is_streaming_mode().await })
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

    // =====================================================================
    // TOCTOU対策テスト
    // =====================================================================

    /// 設定変更ロックの基本動作をテスト
    #[tokio::test]
    async fn test_acquire_settings_lock_basic() {
        let service = StreamingModeService::new();

        // ロックを取得
        let guard = service.acquire_settings_lock().await;
        assert!(guard.is_ok());

        // ロックガードを取得
        let guard = guard.unwrap();

        // ロック保持中は配信中でないことを確認
        assert!(!guard.is_streaming().await);

        // ガードがドロップされるとロックが解放される
        drop(guard);

        // ロックが解放された後も正常に動作
        assert!(!service.is_streaming_mode().await);
    }

    /// ensure_not_streaming が配信中の場合にエラーを返すことをテスト
    #[tokio::test]
    async fn test_ensure_not_streaming_when_streaming() {
        let service = StreamingModeService::new();

        // 配信中に設定
        service.set_streaming_mode(true).await;

        // ロックを取得
        let guard = service.acquire_settings_lock().await.unwrap();

        // 配信中の場合はエラー
        let result = guard.ensure_not_streaming().await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code(), "OBS_STATE");
        assert!(err.message().contains("配信中"));
    }

    /// ensure_not_streaming が配信中でない場合に成功することをテスト
    #[tokio::test]
    async fn test_ensure_not_streaming_when_not_streaming() {
        let service = StreamingModeService::new();

        // ロックを取得
        let guard = service.acquire_settings_lock().await.unwrap();

        // 配信中でない場合は成功
        let result = guard.ensure_not_streaming().await;
        assert!(result.is_ok());
    }

    /// ロック保持中に配信状態の変更がブロックされることをテスト（TOCTOU対策の核心）
    #[tokio::test]
    async fn test_settings_lock_blocks_streaming_state_change() {
        let service = StreamingModeService::new();
        let service_clone = service.clone();

        // ロックを取得（これにより set_streaming_mode がブロックされる）
        let guard = service.acquire_settings_lock().await.unwrap();

        // 別タスクで配信状態の変更を試みる
        let handle = tokio::spawn(async move {
            // 100ms後に配信開始を試みる
            tokio::time::sleep(Duration::from_millis(100)).await;
            let start = std::time::Instant::now();
            service_clone.set_streaming_mode(true).await;
            start.elapsed()
        });

        // ロックを保持したまま200ms待機
        tokio::time::sleep(Duration::from_millis(200)).await;

        // ロック保持中は配信中でないことを確認
        assert!(!guard.is_streaming().await);

        // ロックを解放
        drop(guard);

        // 別タスクの完了を待機
        let elapsed = handle.await.unwrap();

        // set_streaming_mode はロック解放まで待機したため、少なくとも100ms以上かかったはず
        assert!(elapsed.as_millis() >= 100);

        // ロック解放後は配信状態が変更されている
        assert!(service.is_streaming_mode().await);
    }

    /// execute_if_not_streaming が配信中の場合にエラーを返すことをテスト
    #[tokio::test]
    async fn test_execute_if_not_streaming_when_streaming() {
        let service = StreamingModeService::new();
        service.set_streaming_mode(true).await;

        let result: Result<(), AppError> =
            service.execute_if_not_streaming(|| async { Ok(()) }).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code(), "OBS_STATE");
    }

    /// execute_if_not_streaming が配信中でない場合に操作を実行することをテスト
    #[tokio::test]
    async fn test_execute_if_not_streaming_success() {
        let service = StreamingModeService::new();
        let counter = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = service
            .execute_if_not_streaming(|| async move {
                counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok::<_, AppError>(42)
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    /// execute_if_not_streaming が操作実行中に配信状態の変更をブロックすることをテスト
    #[tokio::test]
    async fn test_execute_if_not_streaming_blocks_state_change() {
        let service = StreamingModeService::new();
        let service_clone = service.clone();
        let operation_started = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let operation_started_clone = operation_started.clone();

        // 操作中に配信状態を変更しようとするタスク
        let handle = tokio::spawn(async move {
            // 操作が開始されるまで待機
            while !operation_started_clone.load(std::sync::atomic::Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }

            let start = std::time::Instant::now();
            service_clone.set_streaming_mode(true).await;
            start.elapsed()
        });

        // 時間のかかる操作を実行
        let result = service
            .execute_if_not_streaming(|| async {
                operation_started.store(true, std::sync::atomic::Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok::<_, AppError>(())
            })
            .await;

        assert!(result.is_ok());

        // 配信状態変更はロック解放後に完了
        let elapsed = handle.await.unwrap();

        // 操作完了まで待機したため、少なくとも操作時間分かかったはず
        assert!(elapsed.as_millis() >= 150); // マージンを持たせる
    }

    /// ロックタイムアウトのテスト
    #[tokio::test]
    async fn test_settings_lock_timeout() {
        let service = StreamingModeService::new();
        let service_clone = service.clone();

        // ロックを取得して保持
        let _guard = service.acquire_settings_lock().await.unwrap();

        // 別タスクでタイムアウト付きでロック取得を試みる
        let handle = tokio::spawn(async move {
            service_clone
                .acquire_settings_lock_with_timeout(Duration::from_millis(100))
                .await
        });

        // タイムアウトまで待機
        let result = handle.await.unwrap();

        // タイムアウトエラーになるはず
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code(), "OBS_STATE");
        assert!(err.message().contains("タイムアウト"));
    }

    /// TOCTOU競合条件のシミュレーションテスト
    ///
    /// このテストは、ロックを使用しない場合のTOCTOU問題をシミュレートし、
    /// ロックを使用することで問題が解決されることを確認する
    #[tokio::test]
    async fn test_toctou_race_condition_prevention() {
        let service = StreamingModeService::new();
        let operation_count = Arc::new(std::sync::atomic::AtomicU32::new(0));

        // 複数の同時操作をシミュレート
        let mut handles = vec![];

        for _ in 0..10 {
            let svc = service.clone();
            let count = operation_count.clone();

            handles.push(tokio::spawn(async move {
                // execute_if_not_streaming を使用することで、
                // 配信状態チェックと操作が原子的に行われる
                let result = svc
                    .execute_if_not_streaming(|| async {
                        count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        // 操作に時間がかかるシミュレーション
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        Ok::<_, AppError>(())
                    })
                    .await;
                result.is_ok()
            }));
        }

        // すべての操作が完了するまで待機
        for handle in handles {
            let _ = handle.await.unwrap();
        }

        // すべての操作が成功したはず（配信中でないため）
        assert_eq!(
            operation_count.load(std::sync::atomic::Ordering::SeqCst),
            10
        );

        // 配信中に同じことを試みる
        service.set_streaming_mode(true).await;
        let blocked_count = Arc::new(std::sync::atomic::AtomicU32::new(0));

        let mut handles = vec![];
        for _ in 0..5 {
            let svc = service.clone();
            let count = blocked_count.clone();

            handles.push(tokio::spawn(async move {
                let result: Result<(), AppError> = svc
                    .execute_if_not_streaming(|| async {
                        count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        Ok(())
                    })
                    .await;
                result.is_ok()
            }));
        }

        for handle in handles {
            let succeeded = handle.await.unwrap();
            // 配信中なのですべて失敗するはず
            assert!(!succeeded);
        }

        // 操作は一つも実行されていないはず
        assert_eq!(blocked_count.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    /// ネストされたロック取得のテスト（デッドロック防止の確認）
    /// 注意: 現在の実装では同一タスクでの再帰的ロック取得はデッドロックする
    /// このテストはその動作を確認する
    #[tokio::test]
    async fn test_nested_lock_attempt_times_out() {
        let service = StreamingModeService::new();

        // ロックを取得
        let _guard = service.acquire_settings_lock().await.unwrap();

        // 同じタスク内で再度ロック取得を試みる（短いタイムアウト）
        let result = service
            .acquire_settings_lock_with_timeout(Duration::from_millis(50))
            .await;

        // タイムアウトになるはず（デッドロック防止）
        assert!(result.is_err());
    }
}
