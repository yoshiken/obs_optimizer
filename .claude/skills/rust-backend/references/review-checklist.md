# Rustバックエンド セルフレビューチェックリスト

## 1. エラーハンドリング【最優先】

### 必須確認項目

- [ ] **`unwrap()` / `expect()` の使用禁止（本番コード）**
  - テストコード（`#[cfg(test)]`、`tests/`）のみ許可

- [ ] **`Result<T, AppError>` の一貫した使用**
  ```rust
  // 良い例
  #[tauri::command]
  async fn get_obs_status() -> Result<ObsStatus, AppError> {
      let client = get_obs_client().await?;
      client.get_status().await.map_err(AppError::from)
  }
  ```

- [ ] **エラー変換の適切な実装**
  - `From<外部エラー> for AppError` の実装確認
  - エラーコンテキストの保持（元のエラーメッセージを失わない）

- [ ] **`?` 演算子の適切な使用**

## 2. 型安全性

- [ ] **すべての公開関数に明示的な型注釈**

- [ ] **Tauriコマンドの型定義の整合性**
  - `src-tauri/src/commands/*.rs` の関数シグネチャ
  - `src/types/commands.ts` のTypeScript型定義
  - **両者が一致していることを確認**

- [ ] **serde シリアライゼーションの検証**
  - `#[serde(rename_all = "camelCase")]` の適切な使用

- [ ] **Option / Result の適切な使い分け**

## 3. 非同期処理

- [ ] **async/await の適切な使用**
  - 不要な `.await` がないか
  - デッドロックの可能性がないか

- [ ] **タスクのキャンセル処理**
  - 長時間実行タスクにはキャンセルメカニズムを実装

## 4. リソース管理

- [ ] **Mutex / RwLock の適切な使用**
  - デッドロックを避けるためのロック順序の一貫性
  - ロックスコープの最小化

- [ ] **メモリリークの防止**
  - `Arc` / `Rc` の循環参照チェック

## 5. セキュリティ

- [ ] **すべてのTauriコマンドの入力検証**
  ```rust
  if params.host.is_empty() {
      return Err(AppError::config_error("Host cannot be empty"));
  }
  ```

- [ ] **機密情報の適切な管理**
  - パスワード保存時の暗号化
  - ログ出力に機密情報を含めない

- [ ] **ファイルパスのサニタイゼーション**
  - パストラバーサル攻撃の防止

## 6. パフォーマンス

- [ ] **GPU情報取得の頻度制限**
  - NVML呼び出しはコストが高い

- [ ] **大きなデータ構造のClone回避**
  - `Arc` や参照を活用

- [ ] **並列処理可能な部分の特定**
  - `tokio::join!` / `futures::join_all` の活用

## 7. 保守性・可読性

- [ ] **長すぎる関数の分割**
  - 1関数50行を目安

- [ ] **マジックナンバーの排除**
  ```rust
  const MIN_USER_PORT: u16 = 1024;
  const MAX_PORT: u16 = 65535;
  ```

- [ ] **ユニットテストの記述**
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn test_function_name() {
          // arrange, act, assert
      }
  }
  ```

## 8. 静的解析の実行

```bash
cd src-tauri

# フォーマットチェック
cargo fmt -- --check

# Clippy実行（全警告をエラー扱い）
cargo clippy --all-targets -- -D warnings

# テスト実行
cargo test
cargo test --features testing  # 統合テスト含む
```

## 9. unwrap/expect の確認

```bash
# 本番コード内のunwrap/expectを検索（テストコードを除外）
rg '\.(unwrap|expect)\(' src-tauri/src --type rust \
  | grep -v '#\[cfg(test)\]' \
  | grep -v 'mod tests'
```

## 優先度別チェックリスト

### Critical（必須修正）

1. `unwrap()` / `expect()` の本番コード使用
2. 入力検証の欠落（Tauriコマンド）
3. エラーハンドリングの欠如
4. セキュリティ脆弱性
5. `unsafe_code` の使用

### High（修正推奨）

1. メモリリーク・リソースリーク
2. デッドロックの可能性
3. パフォーマンスボトルネック
4. テストカバレッジ不足
5. Clippy警告の放置

### Medium（改善検討）

1. コードの重複
2. 長すぎる関数
3. マジックナンバー
4. ドキュメント不足
5. 命名の不適切さ
