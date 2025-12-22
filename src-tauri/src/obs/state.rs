// OBSクライアントのグローバル状態管理
//
// アプリケーション全体で共有されるObsClientインスタンスを管理
//
// 設計ノート:
// ObsClient は内部で Arc<RwLock<>> を使用しており、既にスレッドセーフ
// そのため、外側の Mutex は不要で、OnceCell で初期化のみを保護する

use once_cell::sync::OnceCell;

use super::client::ObsClient;

/// `グローバルなObsClientインスタンス`
///
/// `OnceCell` を使用して、初回アクセス時に一度だけ初期化される
/// `ObsClient` は Clone 可能で、内部で Arc を使用しているため
/// `get_obs_client()` で返されるのは同じ内部状態を共有するクローン
///
/// # 使用例
/// ```ignore
/// use crate::obs::state::get_obs_client;
///
/// async fn example() {
///     let client = get_obs_client();
///     let status = client.get_status().await;
/// }
/// ```
static OBS_CLIENT: OnceCell<ObsClient> = OnceCell::new();

/// `ObsClientへのアクセスを提供するヘルパー関数`
///
/// 初回呼び出し時にクライアントを初期化し、以降は同じインスタンスを返す
/// `ObsClient` は Clone を実装しており、内部で Arc を使用しているため
/// 複数のタスクから安全に同時アクセス可能
///
/// # Returns
/// ObsClientのクローン（内部状態は Arc で共有）
pub fn get_obs_client() -> ObsClient {
    OBS_CLIENT.get_or_init(ObsClient::new).clone()
}

/// ObsClientをリセット（主にテスト用）
///
/// 注意: `OnceCell` は再初期化できないため、このメソッドは
/// 既存クライアントの接続を切断するのみで、インスタンスは再利用される
/// 完全なリセットが必要な場合は、プロセス再起動を検討すること
#[allow(dead_code)]
pub async fn reset_obs_client() {
    if let Some(client) = OBS_CLIENT.get() {
        // 接続を切断するが、インスタンス自体は維持
        let _ = client.disconnect().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_obs_client() {
        let client = get_obs_client();
        // クライアントが取得できることを確認
        // 実際の接続テストは統合テストで行う
        drop(client);
    }

    #[tokio::test]
    async fn test_reset_obs_client() {
        // リセットが正常に動作することを確認
        reset_obs_client().await;
        let client = get_obs_client();
        drop(client);
    }

    #[test]
    fn test_multiple_access() {
        // 複数回のアクセスが正常に動作することを確認
        let client1 = get_obs_client();
        let client2 = get_obs_client();
        drop(client1);
        drop(client2);
    }

    #[tokio::test]
    async fn test_client_state_shared() {
        // 複数回の取得で同じ接続状態を共有していることを確認
        let client1 = get_obs_client();
        let client2 = get_obs_client();

        // 両方とも未接続状態であることを確認
        assert!(!client1.is_connected().await);
        assert!(!client2.is_connected().await);
    }
}
