# OBS配信最適化ツール - 並列開発ガイド

> このファイルは Claude Code セッションのシステムプロンプトとして機能する。
> 複数セッションが並列で作業することを前提に設計されている。

---

## Project Identity

- **名前**: OBS配信最適化ツール (obs_optimizer)
- **技術スタック**: Tauri 2.x (Rust) + React 18 + TypeScript 5.x
- **目的**: 配信者向けOBS設定の自動最適化、リアルタイム監視
- **ターゲット**: Windows 10/11（配信者向けデスクトップアプリ）

---

## 1. セッション分割戦略

### 1.1 セッション種別と責務

| セッション名 | 責務 | 担当ディレクトリ | 独立性 |
|-------------|------|-----------------|--------|
| **SESSION_CORE** | Rustバックエンド、Tauriコマンド、エラーハンドリング | `src-tauri/` | 高 |
| **SESSION_UI** | Reactフロントエンド、コンポーネント、状態管理 | `src/` | 高 |
| **SESSION_OBS** | OBS連携、WebSocket通信 | `src-tauri/src/obs/`, `src/features/obs/` | 中 |
| **SESSION_MONITOR** | システム監視、メトリクス収集 | `src-tauri/src/monitor/`, `src/features/monitor/` | 中 |

### 1.2 ファイルロック規則

セッションは作業開始時に `.claude-locks/` ディレクトリを確認すること。

```
.claude-locks/
├── backend.lock      # src-tauri/ 作業中
├── frontend.lock     # src/ 作業中
├── config.lock       # 設定ファイル編集中
└── integration.lock  # 統合テスト実行中
```

**ロックファイル形式**:
```
SESSION_ID: [セッション名]
TASK: [タスク概要]
STARTED: [ISO8601 timestamp]
ESTIMATED: [予想完了時間 in minutes]
```

### 1.3 ファイル所有権マップ

| パスパターン | 所有セッション | 変更ルール |
|-------------|---------------|-----------|
| `src-tauri/src/commands/*` | SESSION_CORE | 自由に変更可 |
| `src-tauri/src/error.rs` | SESSION_CORE | 追加のみ許可 |
| `src/components/*` | SESSION_UI | 自由に変更可 |
| `src/types/commands.ts` | 統合ポイント | **変更時は全セッションに通知** |
| `src/stores/*` | SESSION_UI | 型定義との整合性必須 |
| `contracts/*` | 全セッション | 追加のみ、削除禁止 |
| `tauri.conf.json` | 人間のみ | **セッション編集禁止** |
| `Dockerfile` | SESSION_COMMANDER | **他セッション編集禁止** |
| `docker-compose.yml` | SESSION_COMMANDER | **他セッション編集禁止** |
| `Cargo.toml` | SESSION_CORE | 依存追加はリクエスト経由 |
| `package.json` | SESSION_UI | 依存追加はリクエスト経由 |
| `.claude/dependency-*.md` | 全セッション | 追記のみ、削除禁止 |

---

## 2. コーディング規約

### 2.1 Rust (Backend)

```rust
// 必須: Result型でエラーハンドリング
// 禁止: unwrap(), expect() の本番コード使用

// Good
#[tauri::command]
async fn get_obs_status() -> Result<ObsStatus, AppError> {
    let client = obs_client().await?;
    client.get_status().await.map_err(AppError::from)
}

// Bad
#[tauri::command]
async fn get_obs_status() -> ObsStatus {
    obs_client().await.unwrap().get_status().await.unwrap()
}
```

**必須パターン**:
- エラー型は `src-tauri/src/error.rs` の `AppError` を使用
- 非同期処理は `tokio` ランタイム
- OBS連携は `obws` クレート
- メトリクス収集は `sysinfo` クレート

**命名規則**:
- Tauriコマンド: `snake_case` (例: `get_cpu_usage`)
- 構造体: `PascalCase`
- モジュール: `snake_case`

### 2.2 TypeScript (Frontend)

```typescript
// 必須: 厳格な型定義
// 禁止: any, as unknown as T, @ts-ignore

// Good
interface ObsStatus {
  connected: boolean;
  streaming: boolean;
  recording: boolean;
  cpuUsage: number;
}

const status = await invoke<ObsStatus>('get_obs_status');

// Bad
const status = await invoke('get_obs_status') as any;
```

**必須パターン**:
- 状態管理: Zustand（Redux禁止）
- スタイリング: Tailwind CSS
- コンポーネント: 関数コンポーネント + フック
- Tauri連携: `@tauri-apps/api/core` の `invoke`

**命名規則**:
- コンポーネント: `PascalCase.tsx`
- フック: `useCamelCase.ts`
- ユーティリティ: `camelCase.ts`
- 型定義: `types/camelCase.ts`

### 2.3 共通ルール

- コード内コメントは**日本語**
- 新規関数には対応するテストを追加
- TypeScript `strict: true`, Rust `clippy` 警告ゼロ

---

## 3. インターフェース契約

### 3.1 契約ファイル構成

```
contracts/
├── api.md              # Tauriコマンド定義（信頼の源泉）
├── events.md           # Tauriイベント定義
├── types.md            # 共通型定義
└── obs-protocol.md     # OBS WebSocket プロトコル
```

### 3.2 コマンド定義フォーマット

```typescript
// src/types/commands.ts - このファイルが信頼の源泉

export interface Commands {
  // OBS Connection
  connect_obs: (params: { host: string; port: number; password?: string }) => Promise<void>;
  disconnect_obs: () => Promise<void>;
  get_obs_status: () => Promise<ObsStatus>;

  // Metrics
  get_system_metrics: () => Promise<SystemMetrics>;
  start_metrics_stream: () => Promise<void>;
  stop_metrics_stream: () => Promise<void>;

  // Optimizer
  analyze_settings: () => Promise<AnalysisResult>;
  apply_optimization: (params: { preset: OptimizationPreset }) => Promise<void>;
}
```

**重要**: 新しいコマンド追加時は必ず `src/types/commands.ts` を先に更新すること。

---

## 4. 禁止事項と必須事項

### 4.1 絶対禁止（システム破損の可能性）

1. **`tauri.conf.json` の直接編集** - 設定ファイルは人間が管理
2. **`package.json` の依存関係変更** - 事前承認が必要
3. **`Cargo.toml` の依存関係変更** - 事前承認が必要
4. **`.env` ファイルの作成/編集** - セキュリティリスク
5. **他セッションのロックファイルがある領域への書き込み**
6. **`contracts/` 内の既存定義の削除・変更**（追加のみ許可）

### 4.2 条件付き禁止（確認なしでの実行禁止）

1. ファイル削除 - 必ず理由を説明してから
2. ディレクトリ構造の変更 - 設計ドキュメント参照
3. 外部APIへの接続コード追加 - セキュリティレビュー必要
4. グローバル状態の追加 - 状態管理戦略との整合性確認

### 4.3 必須事項

1. **型安全性の維持**: すべての型を明示的に定義
2. **エラーハンドリング**: すべての非同期処理に適切なエラー処理
3. **変更前確認**: 関連ファイルを読み込んでから変更
4. **ビルド確認**: 変更後にビルド可能か確認

---

## 5. 判断ガイドライン

### 5.1 判断に迷った時のフローチャート

```
Q: この変更は他のセッションに影響するか？
├─ Yes → 変更を中断し、統合リクエストを作成
└─ No → 続行

Q: この変更は破壊的か？(既存機能を壊す可能性)
├─ Yes → 変更を中断し、人間に確認を求める
└─ No → 続行

Q: 複数の実装方法がある。どれを選ぶ？
├─ 既存コードに類似パターンがある → そのパターンに従う
├─ パフォーマンスが重要 → 計測可能な形で実装
└─ どちらでもよい → よりシンプルな方を選択
```

### 5.2 技術選択の優先順位

1. **安全性** > パフォーマンス > 簡潔さ
2. **型安全** > 柔軟性 > 記述量の少なさ
3. **明示的** > 暗黙的 > マジカル

---

## 6. コミュニケーションテンプレート

### 6.1 セッション開始宣言

```markdown
## SESSION_XXX 開始

- **セッション**: SESSION_XXX
- **作業内容**: [具体的なタスク]
- **想定影響ファイル**: [ファイルリスト]
- **依存セッション**: [他セッション名、なければ "なし"]
- **開始時刻**: [ISO8601]
```

### 6.2 進捗報告（作業完了時）

```markdown
## SESSION_XXX 進捗報告

**Status**: Completed | In Progress | Blocked

### 完了タスク
- [x] タスク1の説明
- [x] タスク2の説明

### 作成/変更ファイル
- `path/to/file.rs`: [変更内容]
- `path/to/file.tsx`: [変更内容]

### テスト状況
- [ ] Unit tests passing
- [ ] Build successful
- [ ] No new warnings

### 他セッションへの通知
[他セッションが知るべき情報。なければ "None"]
```

### 6.3 問題報告（ブロック時）

```markdown
## Issue Report

**Session**: SESSION_XXX
**Severity**: Critical | High | Medium | Low

### Problem
[問題の簡潔な説明]

### Context
- File: `path/to/file`
- Function: `function_name`
- Error: [エラーメッセージ]

### Attempted Solutions
1. [試したこと1] → [結果]
2. [試したこと2] → [結果]

### Needed
[解決に必要なもの: 情報/判断/別セッションの作業完了]
```

### 6.4 統合リクエスト

```markdown
## Integration Request

**From**: SESSION_XXX
**To**: SESSION_YYY (or "All")
**Priority**: Urgent | Normal | Low

### Request Type
- [ ] Interface change proposal
- [ ] Dependency on other session's work
- [ ] Conflict resolution needed

### Details
[具体的な内容]

### Proposed Solution
[提案がある場合]
```

---

## 7. クイックリファレンス

### 7.1 Tauri Command Template (Rust)

```rust
// src-tauri/src/commands/example.rs

use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExampleInput {
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct ExampleOutput {
    pub result: String,
}

#[tauri::command]
pub async fn example_command(input: ExampleInput) -> Result<ExampleOutput, AppError> {
    Ok(ExampleOutput {
        result: format!("Processed: {}", input.value),
    })
}
```

### 7.2 Tauri Invoke Template (TypeScript)

```typescript
// src/hooks/useExample.ts

import { invoke } from '@tauri-apps/api/core';
import { useState, useCallback } from 'react';

interface ExampleOutput {
  result: string;
}

export function useExample() {
  const [data, setData] = useState<ExampleOutput | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const execute = useCallback(async (value: string) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<ExampleOutput>('example_command', {
        input: { value }
      });
      setData(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  return { data, error, loading, execute };
}
```

### 7.3 Zustand Store Template

```typescript
// src/stores/exampleStore.ts

import { create } from 'zustand';

interface ExampleState {
  value: string;
  setValue: (value: string) => void;
  reset: () => void;
}

export const useExampleStore = create<ExampleState>((set) => ({
  value: '',
  setValue: (value) => set({ value }),
  reset: () => set({ value: '' }),
}));
```

---

## 8. ディレクトリ構造

```
obs_optimizer/
├── CLAUDE.md                    # このファイル
├── contracts/                   # セッション間契約
│   ├── api.md                   # Tauriコマンド定義
│   ├── events.md                # イベント定義
│   ├── types.md                 # 共通型定義
│   └── obs-protocol.md          # OBS連携仕様
├── .claude-locks/               # セッションロック
│   └── .gitkeep
├── src-tauri/                   # Rustバックエンド [SESSION_CORE]
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── lib.rs               # エントリポイント
│       ├── error.rs             # エラー型定義
│       ├── commands/            # Tauriコマンド
│       │   ├── mod.rs
│       │   ├── system.rs
│       │   └── obs.rs
│       ├── obs/                 # OBS連携 [SESSION_OBS]
│       │   ├── mod.rs
│       │   ├── client.rs
│       │   └── optimizer.rs
│       └── monitor/             # 監視機能 [SESSION_MONITOR]
│           ├── mod.rs
│           ├── cpu.rs
│           ├── gpu.rs
│           └── network.rs
├── src/                         # Reactフロントエンド [SESSION_UI]
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/              # 共通コンポーネント
│   │   ├── common/
│   │   └── layout/
│   ├── features/                # 機能別モジュール
│   │   ├── obs/                 # [SESSION_OBS]
│   │   └── monitor/             # [SESSION_MONITOR]
│   ├── hooks/
│   ├── stores/
│   ├── types/
│   │   ├── index.ts
│   │   └── commands.ts          # 契約の型定義
│   └── styles/
├── package.json
├── tsconfig.json
└── vite.config.ts
```

---

## 9. Docker並列開発

### 9.1 セッション起動コマンド

```bash
# 全セッションを並列起動（4セッション）
docker compose --profile parallel up -d

# 個別セッション起動
docker compose --profile core up -d      # SESSION_CORE
docker compose --profile ui up -d        # SESSION_UI
docker compose --profile obs up -d       # SESSION_OBS
docker compose --profile monitor up -d   # SESSION_MONITOR

# Phase 1: 基盤構築（2セッション）
docker compose --profile core --profile ui up -d
```

### 9.2 セッション接続

```bash
# 各セッションにattach（別ターミナルで実行）
docker attach claude-session-core
docker attach claude-session-ui
docker attach claude-session-obs
docker attach claude-session-monitor

# または exec で新規シェル
docker exec -it claude-session-core bash
```

### 9.3 セッション管理

```bash
# 状態確認
docker compose ps

# ログ確認
docker compose logs session-core
docker compose logs -f session-ui  # フォロー

# セッション停止
docker compose --profile parallel down

# 個別停止
docker compose stop session-core
```

### 9.4 開発サーバー（別コンテナ）

```bash
# 開発環境起動（Tauri dev server用）
docker compose up -d obs-optimizer-dev

# コンテナ内でビルド
docker exec -it obs-optimizer-dev bash
> pnpm install
> pnpm tauri dev
```

---

## 10. ビルドコマンド

```bash
# 開発サーバー起動
pnpm tauri dev

# プロダクションビルド
pnpm tauri build

# Rustのみビルド確認
cd src-tauri && cargo build

# フロントエンドのみ確認
pnpm build

# Lintチェック
pnpm lint
cd src-tauri && cargo clippy

# テスト
pnpm test
cd src-tauri && cargo test
```

---

## 11. 開発フェーズ

### Phase 1: 基盤構築（並列度: 2）

```
SESSION_CORE: Tauriプロジェクト初期化、基本コマンド、エラー型
SESSION_UI:   Reactプロジェクト初期化、レイアウト、型定義
```

### Phase 2: 機能実装（並列度: 4）

```
SESSION_CORE:    コア機能強化
SESSION_UI:      コンポーネント実装、状態管理
SESSION_OBS:     OBS WebSocket連携
SESSION_MONITOR: システム監視機能
```

### Phase 3: 統合・テスト（並列度: 2）

```
SESSION_INTEGRATION: E2Eテスト、統合テスト
SESSION_POLISH:      UI/UX改善、パフォーマンス最適化
```

---

## 12. チェックリスト

### セッション開始時

- [ ] 最新の `contracts/` を確認
- [ ] `.claude-locks/` で他セッションの状況確認
- [ ] 担当ディレクトリ外のファイルに触らない
- [ ] セッション開始宣言を出力

### セッション終了時

- [ ] 進捗報告を作成
- [ ] 変更した `contracts/` ファイルを更新
- [ ] ビルド成功を確認
- [ ] ロックファイルを削除

---

## 13. 司令塔セッション（SESSION_COMMANDER）

### 13.1 概要

SESSION_COMMANDER は依存関係リクエストを一元管理する特別なセッション。
**ホスト側で起動し、Dockerの管理を担当する**。

| 項目 | 内容 |
|------|------|
| 起動場所 | **ホスト（Docker外）** |
| 起動コマンド | `claude --session SESSION_COMMANDER` |
| 責務 | 依存関係管理、Dockerfile編集、ビルド、コンテナ再起動 |
| 監視対象 | `.claude/dependency-requests.md` |
| 編集権限 | `Dockerfile`, `docker-compose.yml` |
| 実行権限 | `docker compose build`, `docker compose up/down` |

### 13.2 依存関係リクエスト方法

作業中にパッケージが不足した場合:

1. **直接編集禁止**: `Dockerfile`, `Cargo.toml`, `package.json` を直接編集しない
2. **リクエスト追加**: `.claude/dependency-requests.md` に以下を追記

```markdown
### REQ-XXX
- **ID**: REQ-XXX（既存の最大ID + 1）
- **From**: [自セッション名]
- **Priority**: critical | normal
- **Type**: apt-package | cargo-crate | npm-package
- **Requested**: [現在時刻 ISO8601]
- **Package**: [パッケージ名]
- **Reason**: [必要な理由]
- **Status**: pending
```

3. **critical の場合**: 作業を中断して待機
4. **normal の場合**: 可能なら代替手段で作業継続

### 13.3 リクエストタイプ一覧

| Type | 対象ファイル | 例 |
|------|-------------|-----|
| `apt-package` | Dockerfile | `libobs-dev`, `libssl-dev` |
| `cargo-crate` | Cargo.toml | `serde = "1.0"` |
| `npm-package` | package.json | `zustand@^4` |
| `dockerfile-config` | Dockerfile | ENV, EXPOSE 追加 |
| `env-var` | docker-compose.yml | 環境変数追加 |

### 13.4 リクエスト完了の確認

- `.claude/dependency-requests.md` から自分のリクエストが消える
- `.claude/dependency-history.md` に移動されている
- `.claude/commander-status.md` に処理結果が記録

### 13.5 緊急時の対応

司令塔が応答しない場合:
1. `.claude/commander-status.md` の `Last Check` を確認
2. 10分以上更新がない場合、人間に報告

### 13.6 禁止事項（全セッション共通）

- `Dockerfile` の直接編集（SESSION_COMMANDER のみ許可）
- `docker-compose.yml` の直接編集（SESSION_COMMANDER のみ許可）
- 他セッションのリクエストの変更・削除

---

## 14. セッション起動順序

### 14.1 推奨起動順序

```bash
# ターミナル1: 司令塔をホスト側で起動
claude --session SESSION_COMMANDER

# ターミナル2: Dockerコンテナをビルド・起動（司令塔が実行）
docker compose build
docker compose --profile core --profile ui up -d

# ターミナル3: SESSION_CORE に接続
docker attach claude-session-core

# ターミナル4: SESSION_UI に接続
docker attach claude-session-ui
```

### 14.2 全作業セッション起動

```bash
# Phase 1: 基盤構築（2セッション）
docker compose --profile parallel up -d

# Phase 2: 全セッション（4セッション）
docker compose --profile all up -d
```

### 14.3 セッション接続

```bash
# 各作業セッションに接続（別ターミナルで）
docker attach claude-session-core
docker attach claude-session-ui
docker attach claude-session-obs
docker attach claude-session-monitor
```

### 14.4 司令塔の責務フロー

```
司令塔 (ホスト)                    作業セッション (Docker内)
      │                                    │
      │  1. docker compose build           │
      │  2. docker compose up -d           │
      │──────────────────────────────────→ │ 起動
      │                                    │
      │                                    │ 作業中...
      │                                    │
      │  ← 依存リクエスト発生              │
      │    (.claude/dependency-requests.md)│
      │                                    │
      │  3. Dockerfile 更新                │
      │  4. docker compose build           │
      │  5. docker compose up -d --force   │
      │──────────────────────────────────→ │ 再起動
      │                                    │
      │  6. 完了通知                       │
      │    (.claude/dependency-history.md) │
      │                                    │
```

---

## 15. セッション別推奨エージェント

### SESSION_COMMANDER（司令塔）
```
@agent-context-manager     → セッション間調整、進捗管理
@agent-fullstack-developer → Dockerfile更新、依存関係追加
@agent-task-decomposition-expert → ブロッカー発生時のタスク再分解
```

### SESSION_CORE（Rustバックエンド）
```
@agent-backend-architect   → エラー型設計、状態管理アーキテクチャ
@agent-fullstack-developer → 実装作業（commands/, state.rs等）
@agent-debugger            → ビルドエラー、ランタイムエラー調査
@agent-code-reviewer       → コード品質確認（Phase 3）
```

### SESSION_UI（Reactフロントエンド）
```
@agent-frontend-developer  → React実装、Zustand、Tailwind
@agent-ui-ux-designer      → ワイヤーフレーム準拠確認、アクセシビリティ
@agent-fullstack-developer → Tauriブリッジ連携
@agent-code-reviewer       → コンポーネント品質確認（Phase 3）
```

### SESSION_OBS（OBS連携）
```
@agent-backend-architect   → WebSocket接続設計、再接続ロジック
@agent-fullstack-developer → obwsクレート統合、コマンド実装
@agent-debugger            → OBS接続問題、イベント配信デバッグ
```

### SESSION_MONITOR（システム監視）
```
@agent-backend-architect   → メトリクス収集アーキテクチャ
@agent-fullstack-developer → sysinfo/NVML統合、ポーリング実装
@agent-debugger            → GPU検出問題、パフォーマンス調査
```

---

## 16. 依存関係ファイル構成

```
.claude/
├── dependency-requests.md    # リクエストキュー（各セッションが追記）
├── dependency-history.md     # 処理済みログ（司令塔が記録）
├── commander-status.md       # 司令塔の状態
└── settings.local.json       # Claude Code 設定
```

---

*Version: 4.0.0*
*Last Updated: 2024-12-20*
