# Rustバックエンド ベストプラクティス集

## 1. エラーハンドリング

### 統一エラー型の使用

```rust
use crate::error::AppError;

// Good: 統一エラー型でResult返却
#[tauri::command]
async fn connect_obs(params: ConnectionParams) -> Result<ObsStatus, AppError> {
    let client = obs_client().await?;
    client.connect(params).await.map_err(AppError::from)
}

// Bad: unwrap/expect の使用（絶対禁止）
#[tauri::command]
async fn bad_example() -> ObsStatus {
    let client = obs_client().await.unwrap(); // NG
    client.status().await.expect("failed") // NG
}
```

### エラーコンテキストの追加

```rust
// Good: エラーメッセージに状況を含める
async fn load_settings(path: &Path) -> Result<Settings, AppError> {
    std::fs::read_to_string(path)
        .map_err(|e| AppError::config_error(
            &format!("Failed to load settings from {:?}: {}", path, e)
        ))?;
    // ...
}
```

### カスタムエラー型は`From`トレイトで変換

```rust
impl From<obws::Error> for AppError {
    fn from(err: obws::Error) -> Self {
        Self::obs_connection(&format!("OBS WebSocket error: {}", err))
    }
}
```

## 2. Tauriコマンド設計パターン

### 標準形式

```rust
use serde::{Deserialize, Serialize};

// Step 1: 入力パラメータ定義（camelCase変換）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyCommandParams {
    pub user_id: String,
    pub max_count: u32,
}

// Step 2: 出力型定義
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MyCommandResult {
    pub total_items: usize,
    pub items: Vec<Item>,
}

// Step 3: コマンド実装
#[tauri::command]
pub async fn my_command(params: MyCommandParams) -> Result<MyCommandResult, AppError> {
    // バリデーション
    if params.max_count == 0 || params.max_count > 1000 {
        return Err(AppError::new("INVALID_PARAM", "max_count must be 1-1000"));
    }

    // サービス層呼び出し
    let items = my_service::fetch_items(&params.user_id, params.max_count).await?;

    Ok(MyCommandResult {
        total_items: items.len(),
        items,
    })
}
```

### 状態管理パターン

```rust
use once_cell::sync::Lazy;
use tokio::sync::RwLock;

// Good: Lazy + RwLock でスレッドセーフなグローバル状態
static STATE: Lazy<RwLock<AppState>> = Lazy::new(|| {
    RwLock::new(AppState::new())
});

#[tauri::command]
pub async fn get_state() -> Result<StateSnapshot, AppError> {
    let state = STATE.read().await;
    Ok(state.snapshot())
}

// ロックスコープの最小化
async fn update_value() -> Result<(), AppError> {
    let value = {
        let state = STATE.read().await;
        state.value.clone() // ロック解放前にクローン
    }; // ここでロック解放

    process(value).await // ロックなしで処理
}
```

### イベント発火パターン

```rust
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusChangedPayload {
    status: String,
    timestamp: i64,
}

pub fn emit_status_changed(app: &AppHandle, status: String) -> Result<(), AppError> {
    let payload = StatusChangedPayload {
        status,
        timestamp: chrono::Utc::now().timestamp(),
    };

    app.emit("status-changed", payload)
        .map_err(|e| AppError::new("EVENT_ERROR", &format!("Failed to emit: {}", e)))
}
```

## 3. 非同期処理

### タイムアウト付き処理

```rust
use tokio::time::{timeout, Duration};

async fn with_timeout<T>(
    future: impl Future<Output = Result<T, AppError>>,
) -> Result<T, AppError> {
    timeout(Duration::from_secs(10), future)
        .await
        .map_err(|_| AppError::system_monitor("Timeout"))??
}
```

### 並列処理

```rust
// tokio::joinでエラーハンドリング
async fn fetch_all_metrics() -> Result<AllMetrics, AppError> {
    let (cpu, gpu, network) = tokio::join!(
        fetch_cpu_metrics(),
        fetch_gpu_metrics(),
        fetch_network_metrics(),
    );

    Ok(AllMetrics {
        cpu: cpu?,
        gpu: gpu?,
        network: network?,
    })
}

// try_joinで最初のエラーで即座に失敗
use futures::future::try_join_all;

async fn fetch_multiple() -> Result<Vec<Data>, AppError> {
    let futures = ids.iter().map(|id| fetch_by_id(*id));
    try_join_all(futures).await
}
```

### 重い処理の分離

```rust
// Good: spawn_blockingで分離
#[tauri::command]
pub async fn heavy_computation(data: Vec<u8>) -> Result<Output, AppError> {
    tokio::task::spawn_blocking(move || {
        process_large_data(data)
    })
    .await
    .map_err(|e| AppError::system_monitor(&format!("Task failed: {}", e)))?
}
```

## 4. よくある間違いと回避方法

### エラーハンドリング

| 間違い | 正しい方法 |
|--------|------------|
| `unwrap()` 使用 | `?` 演算子 + `Result` |
| `expect("msg")` 使用 | `map_err()` でエラー変換 |
| `panic!()` 使用 | `Err(AppError::new(...))` |
| エラー情報の喪失 `.map_err(\|_\| ...)` | `.map_err(\|e\| format!("{}", e))` |

### 非同期処理

| 間違い | 正しい方法 |
|--------|------------|
| `block_on()` in async | `await` を使用 |
| タイムアウトなし | `tokio::time::timeout` |
| `spawn` の戻り値を無視 | `join_handle.await?` |

### 型定義

| 間違い | 正しい方法 |
|--------|------------|
| `#[serde(rename_all)]` 未指定 | `camelCase` 指定 |
| `pub` 漏れ | 公開フィールドに `pub` |
| `Option<T>` の `unwrap()` | `ok_or_else()` + エラー |

## 5. セキュリティ

### パスワードの扱い

```rust
use zeroize::Zeroize;

pub struct Credentials {
    username: String,
    password: String,
}

impl Drop for Credentials {
    fn drop(&mut self) {
        self.password.zeroize(); // メモリから安全に削除
    }
}
```

### ファイルパス検証

```rust
fn validate_path(base: &Path, user_input: &str) -> Result<PathBuf, AppError> {
    let path = base.join(user_input);

    // 正規化してベースディレクトリ外を防ぐ
    let canonical = path.canonicalize()
        .map_err(|e| AppError::new("INVALID_PATH", &format!("Invalid path: {}", e)))?;

    if !canonical.starts_with(base) {
        return Err(AppError::new("INVALID_PATH", "Path traversal detected"));
    }

    Ok(canonical)
}
```

## 6. パフォーマンス最適化

### 不要なクローンの回避

```rust
// Bad: 不要なclone
fn process_data(data: &Vec<String>) {
    let cloned = data.clone();
    for item in cloned {
        println!("{}", item);
    }
}

// Good: 参照を活用
fn process_data(data: &[String]) {
    for item in data {
        println!("{}", item);
    }
}
```

### キャッシング

```rust
static CACHE: Lazy<RwLock<HashMap<String, Arc<Data>>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

async fn get_cached_data(key: &str) -> Result<Arc<Data>, AppError> {
    // Read lock: キャッシュチェック
    {
        let cache = CACHE.read().await;
        if let Some(data) = cache.get(key) {
            return Ok(Arc::clone(data));
        }
    }

    // Write lock: データ取得 + キャッシュ更新
    let data = fetch_from_source(key).await?;
    let arc_data = Arc::new(data);

    {
        let mut cache = CACHE.write().await;
        cache.insert(key.to_string(), Arc::clone(&arc_data));
    }

    Ok(arc_data)
}
```

## 7. テストパターン

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let params = MyParams { value: 100 };
        assert!(validate_params(&params).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let params = MyParams { value: 0 };
        let err = validate_params(&params).unwrap_err();
        assert_eq!(err.code(), "INVALID_PARAM");
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

## 8. ドキュメントコメント

```rust
/// OBS接続を確立する
///
/// # Arguments
/// * `params` - 接続パラメータ（ホスト、ポート、パスワード）
///
/// # Errors
/// * `AppError::obs_connection` - 接続に失敗した場合
#[tauri::command]
pub async fn connect_obs(params: ObsConnectionParams) -> Result<(), AppError> {
    // ...
}
```
