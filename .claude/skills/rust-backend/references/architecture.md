# OBS Optimizer アーキテクチャガイドライン

## 1. レイヤードアーキテクチャ

このプロジェクトは明確な4層構造を採用している。

```
┌─────────────────────────────────────────────────────┐
│  Tauriコマンド層 (commands/)                         │
│  - フロントエンドからの入力受付                      │
│  - パラメータのバリデーション                        │
│  - サービス層への委譲                               │
└───────────────────────┬─────────────────────────────┘
                        │ 呼び出し
┌───────────────────────▼─────────────────────────────┐
│  サービス層 (services/)                             │
│  - ビジネスロジックの集約                           │
│  - ドメイン層の統合・オーケストレーション            │
│  - エラーハンドリングとロギングのフックポイント       │
└───────────────────────┬─────────────────────────────┘
                        │ 呼び出し
┌───────────────────────▼─────────────────────────────┐
│  ドメイン層 (obs/, monitor/)                        │
│  - 技術的な実装詳細                                 │
│  - OBS WebSocket通信、システム監視                  │
│  - 外部システムとの接続                             │
└───────────────────────┬─────────────────────────────┘
                        │ データ永続化
┌───────────────────────▼─────────────────────────────┐
│  ストレージ層 (storage/)                            │
│  - 設定ファイルの読み書き (JSON)                    │
│  - セッション履歴 (SQLite)                          │
│  - アプリケーションデータ管理                       │
└─────────────────────────────────────────────────────┘
```

## 2. 依存関係の方向性

### 許可される依存方向

```
commands/ --> services/ --> obs/, monitor/ --> storage/
              services/ --> storage/
```

### 禁止される依存方向

- `storage/` から上位層への依存
- `obs/`, `monitor/` から `commands/` への依存
- `services/` から `commands/` への依存
- 循環依存は全て禁止

## 3. ファイル配置ルール

| 新規追加内容 | 配置先 |
|--------------|--------|
| 新Tauriコマンド | `src-tauri/src/commands/` |
| 新サービス | `src-tauri/src/services/` |
| OBS関連の技術実装 | `src-tauri/src/obs/` |
| システム監視の技術実装 | `src-tauri/src/monitor/` |
| 永続化処理 | `src-tauri/src/storage/` |
| テストユーティリティ | `src-tauri/src/testing/` |

## 4. SOLID原則の適用

### Single Responsibility Principle (単一責任)

- 1つのモジュール/構造体が複数の責務を持たない
- サービス層が肥大化していないか確認
- コマンド層にビジネスロジックを置かない

### Open/Closed Principle (開放閉鎖)

- 新しい配信プラットフォームを追加する際、既存コードの変更が最小限
- enumで拡張ポイントを設ける

```rust
pub enum StreamingPlatform {
    YouTube,
    Twitch,
    NicoNico,
    TwitCasting,
    Other,  // 将来の拡張ポイント
}
```

### Liskov Substitution Principle (リスコフの置換)

- エラー変換が情報を失っていないか確認
- 抽象型（trait）の実装が契約を破っていないか

### Interface Segregation Principle (インターフェース分離)

- モジュール構造自体がインターフェースを分離
- 巨大なtraitを定義しない

### Dependency Inversion Principle (依存性逆転)

- テスタビリティのためにtraitを導入
- 外部依存がDI可能か確認

## 5. 型定義の信頼の源泉

### フロントエンドとの契約

- `src/types/commands.ts` がTypeScript側の型定義
- `contracts/api.md` がAPI契約書
- Rustの構造体は `serde` の `#[serde(rename_all = "camelCase")]` でTypeScriptと整合

## 6. コードレビュー時のアーキテクチャチェックリスト

| カテゴリ | チェック項目 |
|----------|-------------|
| **レイヤー違反** | commands/がservices/以外に直接依存していないか |
| **エラーハンドリング** | `unwrap()` / `expect()` を使用していないか |
| **型安全性** | `AppError`型を返しているか |
| **契約整合性** | `commands.ts`と構造体が一致しているか |
| **セッション所有権** | 担当外ディレクトリを変更していないか |

## 7. アーキテクチャ影響度評価

### High Impact（要注意）

- `src/types/commands.ts` の変更（全セッション影響）
- `src-tauri/src/error.rs` への新エラーコード追加
- 新サービスモジュールの追加
- ストレージスキーマの変更

### Medium Impact

- 既存サービスへのメソッド追加
- 新Tauriコマンドの追加
- ドメイン層内部の変更

### Low Impact

- 既存ロジックのリファクタリング（インターフェース変更なし）
- テストの追加
- コメント・ドキュメントの改善

## 8. 命名規約

| 対象 | 規約 | 例 |
|------|------|-----|
| Tauriコマンド | `snake_case` | `get_obs_status` |
| Rust構造体 | `PascalCase` | `ObsStatus` |
| TypeScriptプロパティ | `camelCase` | `bitrateKbps` |
| モジュール | `snake_case` | `encoder_selector.rs` |
