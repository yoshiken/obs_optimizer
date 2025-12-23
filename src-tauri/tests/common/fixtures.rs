// 統合テスト用フィクスチャ
//
// 統合テストで使用する標準的なテストデータを提供する。
// 注意: ここでは外部クレートに依存しない基本的な型のみを定義する。
// obs_optimizer_app_libの型を使用する場合は、各テストファイルでインポートすること。

use std::path::PathBuf;

// =============================================================================
// テスト用パス
// =============================================================================

/// テスト用の一時ディレクトリパスを取得
pub fn test_temp_dir() -> PathBuf {
    std::env::temp_dir().join("obs_optimizer_tests")
}

/// テスト用の設定ファイルパスを取得
pub fn test_config_path() -> PathBuf {
    test_temp_dir().join("config.json")
}

/// テスト用のデータベースパスを取得
pub fn test_db_path() -> PathBuf {
    test_temp_dir().join("test.db")
}

// =============================================================================
// テスト環境セットアップ
// =============================================================================

/// テスト用ディレクトリを作成
pub fn setup_test_dir() -> std::io::Result<PathBuf> {
    let dir = test_temp_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// テスト用ディレクトリをクリーンアップ
pub fn cleanup_test_dir() -> std::io::Result<()> {
    let dir = test_temp_dir();
    if dir.exists() {
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}

/// テスト用のユニークなパスを生成（テスト間の衝突を避ける）
pub fn unique_test_path(prefix: &str) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    test_temp_dir().join(format!("{}_{}", prefix, timestamp))
}

// =============================================================================
// テスト用JSON
// =============================================================================

/// 有効な設定JSONを返す
pub fn valid_config_json() -> &'static str {
    r#"{
        "obsConnection": {
            "host": "localhost",
            "port": 4455,
            "password": null,
            "autoConnect": true
        },
        "streamingPlatform": "youtube",
        "streamingStyle": "gaming",
        "uiSettings": {
            "theme": "dark",
            "language": "ja",
            "showSystemTray": true,
            "minimizeToTray": true
        }
    }"#
}

/// 不正な設定JSONを返す
pub fn invalid_config_json() -> &'static str {
    r#"{ "invalid": json, }"#
}

// =============================================================================
// テスト用環境変数
// =============================================================================

/// テスト用の環境変数を設定し、クリーンアップ用のガードを返す
pub struct EnvGuard {
    key: String,
    original: Option<String>,
}

impl EnvGuard {
    pub fn set(key: &str, value: &str) -> Self {
        let original = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self {
            key: key.to_string(),
            original,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(value) => std::env::set_var(&self.key, value),
            None => std::env::remove_var(&self.key),
        }
    }
}

// =============================================================================
// テスト用ビットレートデータ
// =============================================================================

/// 安定したビットレート履歴（変動係数 < 5%）
pub fn stable_bitrates() -> Vec<u64> {
    vec![6000, 6010, 5990, 6005, 5995, 6000, 6008, 5992, 6000, 6000]
}

/// 不安定なビットレート履歴（変動係数 > 20%）
pub fn unstable_bitrates() -> Vec<u64> {
    vec![6000, 4500, 6200, 3500, 5800, 4000, 6500, 3000, 5500, 4200]
}

/// 帯域不足のビットレート履歴（目標の80%未満）
pub fn insufficient_bitrates(target: u64) -> Vec<u64> {
    let avg = (target as f64 * 0.6) as u64;
    vec![avg, avg + 200, avg - 300, avg + 100, avg - 200,
         avg + 50, avg - 100, avg + 150, avg - 250, avg]
}

// =============================================================================
// テスト用待機
// =============================================================================

/// 指定ミリ秒だけ待機（非同期）
pub async fn wait_ms(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

/// 指定ミリ秒だけ待機（同期）
pub fn wait_ms_sync(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
