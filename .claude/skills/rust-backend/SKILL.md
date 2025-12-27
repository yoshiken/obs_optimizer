---
name: rust-backend
description: |
  OBS Optimizer プロジェクトのRustバックエンド開発用スキル。
  このスキルは、Rustファイル(src-tauri/src/)の編集時にTDD駆動でベストプラクティスに従った実装とセルフコードレビューを自動的に行う。
  バックエンドの機能実装・改善、新規Tauriコマンド追加時に使用する。
---

# Rust Backend Development Skill

OBS Optimizer (Tauri 2.x + Rust) のバックエンド開発を支援するスキル。

## 開発方針: TDD駆動開発

このスキルは**テスト駆動開発（TDD）**を採用する。すべての機能実装は以下のサイクルで行う:

```
Red → Green → Refactor
```

1. **Red（失敗するテストを書く）**: 実装前に期待する動作をテストとして記述
2. **Green（テストを通す）**: テストをパスする最小限のコードを実装
3. **Refactor（リファクタリング）**: テストが通る状態を維持しながらコードを改善

## 適用条件

以下の条件でこのスキルを使用する:

- `src-tauri/src/` 配下のRustファイルを編集する場合
- 新しいTauriコマンドを追加する場合
- バックエンドの機能実装・改善を行う場合

## 開発ワークフロー

### Phase 1: 事前確認

1. **ロック確認**: `.claude-locks/` で担当領域がロックされていないか確認
2. **契約確認**: 新規コマンド追加時は `src/types/commands.ts` を先に確認
3. **依存関係**: 新規クレート必要時は `.claude/dependency-requests.md` に記載

### Phase 2: 実装（TDDサイクル）

#### Step 1: テストを先に書く（Red）

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_command_name_success() {
        let params = InputParams { field_name: "test".to_string() };
        let result = command_name(params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().result_field, "expected_value");
    }

    #[tokio::test]
    async fn test_command_name_validation_error() {
        let params = InputParams { field_name: "".to_string() };
        let result = command_name(params).await;
        assert!(result.is_err());
    }
}
```

#### Step 2: テストを通す実装（Green）

#### Tauriコマンドの標準パターン

```rust
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputParams {
    pub field_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputData {
    pub result_field: String,
}

#[tauri::command]
pub async fn command_name(params: InputParams) -> Result<OutputData, AppError> {
    // 1. バリデーション
    if params.field_name.is_empty() {
        return Err(AppError::new("INVALID_PARAM", "field_name is required"));
    }

    // 2. サービス層呼び出し
    let result = service::process(&params.field_name).await?;

    // 3. 結果返却
    Ok(OutputData { result_field: result })
}
```

#### Step 3: リファクタリング（Refactor）

テストが通ったら、以下を確認しながらコードを改善:
- 重複コードの抽出
- 命名の改善
- エラーハンドリングの強化
- パフォーマンス最適化

**リファクタリング中は常にテストを実行して動作を保証する。**

```bash
cd src-tauri && cargo test
```

#### 絶対禁止事項

- `unwrap()` / `expect()` の本番コード使用
- `panic!()` / `todo!()` の使用
- `static mut` の使用
- `tauri.conf.json` / `Cargo.toml` の直接編集

#### 必須事項

- すべての関数で `Result<T, AppError>` を返す
- `#[serde(rename_all = "camelCase")]` を Deserialize/Serialize 型に付与
- 日本語でコメントを記述
- エラーにはコンテキスト情報を含める

### Phase 3: セルフレビュー

実装完了後、以下のチェックを実行:

#### 3.1 静的解析（必須）

```bash
cd src-tauri
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

#### 3.2 コードレビューチェックリスト

詳細は `references/review-checklist.md` を参照。主要項目:

**Critical（必須）**:
- `unwrap()` / `expect()` を使用していない
- すべての入力にバリデーションがある
- `Result<T, AppError>` を返している
- TypeScript型定義と整合性がある

**High（推奨）**:
- エラーメッセージにコンテキストがある
- 非同期処理にタイムアウトがある
- ロックスコープが最小化されている
- テストが追加されている

#### 3.3 アーキテクチャ確認

詳細は `references/architecture.md` を参照。主要ポイント:

- `commands/` → `services/` → `obs/`, `monitor/` → `storage/` の依存方向を守る
- 循環依存を作らない
- コマンド層にビジネスロジックを置かない

### Phase 4: 完了処理

1. **lib.rs登録**: 新コマンドは `generate_handler![]` に追加
2. **型定義更新**: `src/types/commands.ts` を更新
3. **ビルド確認**: `pnpm tauri dev` で動作確認

## クイックリファレンス

### エラーハンドリング

```rust
// Good: エラー変換とコンテキスト追加
let data = load_file(path)
    .map_err(|e| AppError::config_error(&format!("Failed to load {}: {}", path, e)))?;

// Bad: unwrap使用
let data = load_file(path).unwrap();
```

### 状態管理

```rust
use once_cell::sync::Lazy;
use tokio::sync::RwLock;

static STATE: Lazy<RwLock<MyState>> = Lazy::new(|| {
    RwLock::new(MyState::default())
});
```

### 非同期処理

```rust
// タイムアウト付き処理
use tokio::time::{timeout, Duration};

let result = timeout(Duration::from_secs(10), async_operation())
    .await
    .map_err(|_| AppError::system_monitor("Operation timed out"))??;
```

## 参考資料

- `references/review-checklist.md`: 詳細なレビューチェックリスト
- `references/architecture.md`: アーキテクチャガイドライン
- `references/best-practices.md`: Rustベストプラクティス集
- `/home/yskn/git/obs_optimizer/CLAUDE.md`: プロジェクト全体のルール
