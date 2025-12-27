# OBS配信最適化ツール - Claude Code 指示書

> **重要**: このファイルはClaude Codeセッションのシステムプロンプトとして機能する。
> 並列セッション運用を前提とし、最重要ルールから順に記載。

## プロジェクト概要

| 項目 | 値 |
|------|-----|
| 名前 | OBS配信最適化ツール (obs_optimizer) |
| 技術 | Tauri 2.x (Rust) + React 18 + TypeScript 5.x |
| 目的 | OBS設定の自動最適化、リアルタイム監視 |
| 対象 | Windows 10/11 デスクトップアプリ |
| スコープ | **配信特化**（録画機能は対象外） |

### 対応GPU世代

| ベンダー | 世代 | 主要モデル | AV1 | 備考 |
|----------|------|------------|-----|------|
| NVIDIA | Blackwell | RTX 50シリーズ | 対応 | 最新世代 |
| NVIDIA | Ada | RTX 40シリーズ | 対応 | - |
| NVIDIA | Ampere | RTX 30シリーズ | - | - |
| NVIDIA | Turing | RTX 20/GTX 16 | - | - |
| NVIDIA | Pascal | GTX 10シリーズ | - | Bフレーム非対応 |
| AMD | VCN 4.0 | RX 7000シリーズ | - | Bフレーム対応 |
| AMD | VCN 3.0 | RX 6000シリーズ | - | - |
| Intel | Arc | A770/A750等 | 対応 | - |
| Intel | QuickSync | 内蔵GPU | - | - |

> **AV1注意**: 現時点でAV1配信対応はYouTubeのみ。Twitch/ニコニコ等はH.264を使用

## 1. 絶対禁止事項

**以下の操作は理由を問わず禁止。違反はシステム破損につながる。**

| 禁止対象 | 理由 |
|----------|------|
| `tauri.conf.json` の編集 | 人間のみが管理 |
| `package.json` の依存関係変更 | SESSION_COMMANDER経由で申請 |
| `Cargo.toml` の依存関係変更 | SESSION_COMMANDER経由で申請 |
| `Dockerfile` / `docker-compose.yml` の編集 | SESSION_COMMANDER専用 |
| `.env` ファイルの作成・編集 | セキュリティリスク |
| `contracts/` 内の既存定義の削除・変更 | 追加のみ許可 |
| 他セッションのロック領域への書き込み | `.claude-locks/` で確認 |
| `unwrap()` / `expect()` の本番コード使用 | Rust |
| `any` / `as unknown as T` / `@ts-ignore` | TypeScript |

## 2. 条件付き禁止（人間への確認必須）

以下は実行前に**人間に確認を求める**こと:
- ファイル削除（理由を説明）
- ディレクトリ構造の変更
- 外部API接続コードの追加
- グローバル状態の追加

## 3. 必須事項

すべてのセッションが遵守すべきルール:

1. **型安全性**: すべての型を明示的に定義
2. **エラーハンドリング**: 非同期処理には必ずエラー処理
3. **変更前確認**: 関連ファイルを読み込んでから変更
4. **ビルド確認**: 変更後にビルド可能か確認
5. **契約優先**: `src/types/commands.ts` が信頼の源泉

## 4. セッション管理

### 4.1 セッション種別

| セッション | 責務 | 担当ディレクトリ |
|------------|------|------------------|
| SESSION_CORE | Rustバックエンド、Tauriコマンド | `src-tauri/` |
| SESSION_UI | Reactフロントエンド、状態管理 | `src/` |
| SESSION_OBS | OBS連携、WebSocket | `src-tauri/src/obs/`, `src/features/obs/` |
| SESSION_MONITOR | システム監視、メトリクス | `src-tauri/src/monitor/`, `src/features/monitor/` |
| SESSION_COMMANDER | 依存関係管理、Docker管理 | ホスト側で実行 |

### 4.2 ファイル所有権

| パス | 所有者 | ルール |
|------|--------|--------|
| `src-tauri/src/commands/*` | SESSION_CORE | 自由変更可 |
| `src-tauri/src/error.rs` | SESSION_CORE | 追加のみ |
| `src/components/*` | SESSION_UI | 自由変更可 |
| `src/types/commands.ts` | 統合ポイント | **変更時は全セッションに通知** |
| `src/stores/*` | SESSION_UI | 型定義との整合性必須 |
| `contracts/*` | 全セッション | 追加のみ、削除禁止 |

### 4.3 ロック確認

作業開始時に `.claude-locks/` を確認。ロックがある領域には書き込み禁止。

```
.claude-locks/
├── backend.lock      # src-tauri/ 作業中
├── frontend.lock     # src/ 作業中
├── config.lock       # 設定ファイル編集中
└── integration.lock  # 統合テスト実行中
```

## 5. コーディング規約

### 5.1 Rust

```rust
// 必須パターン: Result型でエラーハンドリング
#[tauri::command]
async fn get_obs_status() -> Result<ObsStatus, AppError> {
    let client = obs_client().await?;
    client.get_status().await.map_err(AppError::from)
}
```

**ルール**:
- エラー型: `src-tauri/src/error.rs` の `AppError`
- 非同期: `tokio` ランタイム
- OBS連携: `obws` クレート
- メトリクス: `sysinfo` クレート
- 命名: コマンド `snake_case`、構造体 `PascalCase`

### 5.2 TypeScript

```typescript
// 必須パターン: 厳格な型定義
interface ObsStatus {
  connected: boolean;
  streaming: boolean;
}
const status = await invoke<ObsStatus>('get_obs_status');
```

**ルール**:
- 状態管理: Zustand（Redux禁止）
- スタイリング: Tailwind CSS
- Tauri連携: `@tauri-apps/api/core` の `invoke`
- 命名: コンポーネント `PascalCase.tsx`、フック `useCamelCase.ts`

### 5.3 共通

- コメントは**日本語**
- 新規関数にはテスト追加
- `strict: true` (TS)、`clippy` 警告ゼロ (Rust)

## 6. 判断ガイドライン

**迷ったときの優先順位**:
1. 安全性 > パフォーマンス > 簡潔さ
2. 型安全 > 柔軟性 > 記述量の少なさ
3. 明示的 > 暗黙的 > マジカル

**判断フロー**:
- 他セッションに影響 → 統合リクエスト作成
- 破壊的変更の可能性 → 人間に確認
- 複数の実装方法 → 既存パターンに従う、なければシンプルな方

## 7. インターフェース契約

### 契約ファイル

```
contracts/
├── api.md          # Tauriコマンド定義
├── events.md       # Tauriイベント定義
├── types.md        # 共通型定義
└── obs-protocol.md # OBS WebSocketプロトコル
```

### コマンド定義（信頼の源泉）

新コマンド追加時は `src/types/commands.ts` を**先に更新**すること。

```typescript
// src/types/commands.ts
export interface Commands {
  connect_obs: (params: { host: string; port: number; password?: string }) => Promise<void>;
  disconnect_obs: () => Promise<void>;
  get_obs_status: () => Promise<ObsStatus>;
  get_system_metrics: () => Promise<SystemMetrics>;
  analyze_settings: () => Promise<AnalysisResult>;
  apply_optimization: (params: { preset: OptimizationPreset }) => Promise<void>;
}
```

## 8. 依存関係リクエスト

パッケージ追加が必要な場合、**直接編集せず**以下の手順:

1. `.claude/dependency-requests.md` に追記:
```markdown
### REQ-XXX
- **From**: [自セッション名]
- **Priority**: critical | normal
- **Type**: apt-package | cargo-crate | npm-package
- **Package**: [パッケージ名]
- **Reason**: [必要な理由]
```

2. `critical` → 作業中断して待機
3. `normal` → 可能なら代替手段で継続

## 9. コミュニケーション

### セッション開始時（必須）

```markdown
## SESSION_XXX 開始
- 作業内容: [タスク]
- 影響ファイル: [リスト]
- 依存セッション: [なし or セッション名]
```

### 作業完了時（必須）

```markdown
## SESSION_XXX 完了
- Status: Completed | Blocked
- 変更ファイル: [リスト]
- 他セッションへの通知: [情報 or なし]
```

### 問題発生時

```markdown
## Issue: [問題の一行要約]
- Session: SESSION_XXX
- Severity: Critical | High | Medium | Low
- File: [ファイルパス]
- Error: [エラーメッセージ]
- Needed: [解決に必要なもの]
```

## 10. ディレクトリ構造

```
obs_optimizer/
├── CLAUDE.md                    # このファイル
├── contracts/                   # セッション間契約
├── .claude-locks/               # セッションロック
├── .claude/                     # 依存関係管理
│   ├── dependency-requests.md   # リクエストキュー
│   └── dependency-history.md    # 処理済みログ
├── src-tauri/                   # Rustバックエンド [SESSION_CORE]
│   ├── Cargo.toml
│   ├── tauri.conf.json          # 編集禁止
│   └── src/
│       ├── lib.rs
│       ├── error.rs             # AppError定義
│       ├── commands/            # Tauriコマンド
│       ├── obs/                 # [SESSION_OBS]
│       └── monitor/             # [SESSION_MONITOR]
└── src/                         # Reactフロントエンド [SESSION_UI]
    ├── main.tsx
    ├── App.tsx
    ├── components/
    ├── features/
    │   ├── obs/                 # [SESSION_OBS]
    │   └── monitor/             # [SESSION_MONITOR]
    ├── hooks/
    ├── stores/
    └── types/
        └── commands.ts          # 契約の型定義（信頼の源泉）
```

## 11. ビルドコマンド

```bash
# 開発
pnpm tauri dev

# ビルド
pnpm tauri build

# Lint
pnpm lint
cd src-tauri && cargo clippy

# テスト
pnpm test
cd src-tauri && cargo test
```

## 12. Docker運用

```bash
# 司令塔（ホスト側）
claude --session SESSION_COMMANDER

# セッション起動
docker compose --profile core --profile ui up -d    # Phase 1
docker compose --profile parallel up -d              # 全セッション

# セッション接続
docker attach claude-session-core
docker attach claude-session-ui
```

## 13. クイックリファレンス

### Tauriコマンド (Rust)

```rust
use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Input { pub value: String }

#[derive(Debug, Serialize)]
pub struct Output { pub result: String }

#[tauri::command]
pub async fn my_command(input: Input) -> Result<Output, AppError> {
    Ok(Output { result: format!("Processed: {}", input.value) })
}
```

### Tauri呼び出し (TypeScript)

```typescript
import { invoke } from '@tauri-apps/api/core';

interface Output { result: string }

const result = await invoke<Output>('my_command', { input: { value: 'test' } });
```

### Zustandストア

```typescript
import { create } from 'zustand';

interface State {
  value: string;
  setValue: (v: string) => void;
}

export const useStore = create<State>((set) => ({
  value: '',
  setValue: (value) => set({ value }),
}));
```

---

## 14. エンコーダー設定ガイドライン

### 統合ティア（EffectiveTier）システム

GPU世代とグレード（型番）を組み合わせて最終的な性能ティアを決定:

```
              | Flagship | HighEnd | UpperMid | Mid  | Entry |
Blackwell(50) |    S     |    S    |    S     |  A   |   B   |
Ada (40)      |    S     |    S    |    A     |  A   |   B   |
Ampere (30)   |    A     |    A    |    B     |  B   |   C   |
Turing (20)   |    B     |    B    |    C     |  C   |   D   |
Pascal (10)   |    C     |    C    |    D     |  D   |   E   |
```

### ティア別設定調整

| ティア | プリセット調整 | マルチパス | 用途例 |
|--------|---------------|-----------|--------|
| TierS | 調整なし | 有効 | RTX 4090/5090 |
| TierA | 調整なし | 有効 | RTX 3090, 4060 |
| TierB | -1段階 | 有効 | RTX 3070, 4050 |
| TierC | -1段階 | 無効 | RTX 3050, 2070 |
| TierD | -2段階 | 無効 | GTX 1660 |
| TierE | -3段階 | 無効 | GTX 1050 |

### AV1エンコーダー使用条件

1. プラットフォーム: **YouTubeのみ**
2. GPU: Ada/Blackwell（NVIDIA）またはIntel Arc
3. エンコーダーID: `jim_av1_nvenc`（NVIDIA）、`obs_qsv11_av1`（Intel）

> **重要**: Twitch/ニコニコ/ツイキャスではH.264を使用すること

---

*Version: 5.1.0 | Last Updated: 2024-12-27*
