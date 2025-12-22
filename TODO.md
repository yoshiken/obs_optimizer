# OBS配信最適化ツール - 並列開発タスクリスト

> 各セッションはこのファイルを参照し、自分の担当タスクを実行する。
> タスク完了時は `[ ]` を `[x]` に変更し、コミットすること。

---

## Phase 0: プロジェクト初期化【BLOCKER】

**担当**: SESSION_COMMANDER（ホスト側）
**並列度**: 1
**所要時間**: 30分

```
[ ] INIT-001: Tauriプロジェクト生成 (S)
    コマンド: pnpm create tauri-app obs_optimizer -- --template react-ts
    成果物: src-tauri/, src/, package.json

[ ] INIT-002: ディレクトリ構造作成 (S)
    作成:
    - src-tauri/src/commands/
    - src-tauri/src/obs/
    - src-tauri/src/monitor/
    - src/components/common/
    - src/components/layout/
    - src/features/dashboard/
    - src/features/obs/
    - src/features/monitor/
    - src/hooks/
    - src/stores/
    - src/types/

[ ] INIT-003: 依存関係インストール (M)
    Rust: obws, sysinfo, tokio, serde, thiserror, tracing
    npm: zustand, @tauri-apps/api, recharts, tailwindcss

[ ] INIT-004: Docker build & 作業セッション起動 (S)
    コマンド:
    - docker compose build
    - docker compose --profile parallel up -d
```

**待ち合わせ**: 全セッションは INIT-004 完了を確認してから Phase 1 開始

---

## Phase 1: 基盤構築

**並列度**: 2（SESSION_CORE + SESSION_UI）
**所要時間**: 2-3時間

### SESSION_CORE タスク

```
[ ] CORE-101: エラー型定義 (M)
    ファイル: src-tauri/src/error.rs
    内容: AppError構造体、thiserror統合
    依存: INIT-004

[ ] CORE-102: 設定ファイル管理 (M)
    ファイル: src-tauri/src/config.rs
    内容: serde + JSON、アプリ設定の永続化
    依存: CORE-101

[ ] CORE-103: アプリ状態管理 (M)
    ファイル: src-tauri/src/state.rs
    内容: Arc<Mutex<AppState>>、Tauri State
    依存: CORE-101

[ ] CORE-104: 基本Tauriコマンド (M)
    ファイル: src-tauri/src/commands/mod.rs
    内容: get_app_info, get_config, save_config
    依存: CORE-102, CORE-103

[ ] CORE-105: ログ基盤 (S)
    ファイル: src-tauri/src/logging.rs
    内容: tracing設定
    依存: CORE-101
```

### SESSION_UI タスク

```
[ ] UI-101: TypeScript型定義 (M)
    ファイル: src/types/
    内容:
    - commands.ts（contracts/api.mdから生成）
    - obs.ts（ObsStatus, ObsSettings）
    - metrics.ts（SystemMetrics, CpuMetrics, GpuMetrics）
    - error.ts（AppError）
    依存: INIT-004

[ ] UI-102: Tailwind CSS設定 (S)
    ファイル: tailwind.config.js, src/styles/globals.css
    内容: カラーパレット、ダークモード対応
    依存: INIT-004

[ ] UI-103: Zustand基本ストア (M)
    ファイル: src/stores/appStore.ts
    内容: 接続状態、設定、メトリクス
    依存: UI-101

[ ] UI-104: Tauriブリッジフック (M)
    ファイル:
    - src/hooks/useTauriCommand.ts
    - src/hooks/useTauriEvent.ts
    内容: invoke wrapper、event listener
    依存: UI-101

[ ] UI-105: レイアウトコンポーネント (M)
    ファイル: src/components/layout/
    内容: AppShell, Sidebar, Header
    依存: UI-102

[ ] UI-106: 共通UIコンポーネント (M)
    ファイル: src/components/common/
    内容: Button, Card, StatusIndicator, MetricsCard
    依存: UI-102
```

**待ち合わせ**: 統合ポイント #1（IPC通信基盤確立）

```
[ ] INT-001: TypeScript型とRust型の整合性確認 (S)
    担当: SESSION_UI

[ ] INT-002: 基本コマンドの疎通テスト (S)
    担当: SESSION_CORE
    コマンド: pnpm tauri dev で動作確認
```

---

## Phase 2: 機能実装

**並列度**: 4（SESSION_CORE + SESSION_UI + SESSION_OBS + SESSION_MONITOR）
**所要時間**: 4-6時間

### SESSION_OBS タスク

```
[ ] OBS-201: OBS接続モジュール (L)
    ファイル: src-tauri/src/obs/connection.rs
    内容:
    - obwsクレート統合
    - connect/disconnect/reconnect
    - 接続状態管理
    - 指数バックオフ再接続
    依存: CORE-103

[ ] OBS-202: OBS接続コマンド (M)
    ファイル: src-tauri/src/commands/obs.rs
    内容: connect_obs, disconnect_obs, get_obs_status
    依存: OBS-201

[ ] OBS-203: OBSメトリクス取得 (M)
    ファイル: src-tauri/src/obs/metrics.rs
    内容:
    - Stats取得（CPU, FPS, フレームドロップ）
    - ビットレート計算（差分）
    依存: OBS-201

[ ] OBS-204: OBS設定読み取り (M)
    ファイル: src-tauri/src/obs/settings.rs
    内容: Video/Audio/Output設定取得
    依存: OBS-201

[ ] OBS-205: OBSイベント配信 (M)
    ファイル: src-tauri/src/obs/events.rs
    内容:
    - obs_connection_changed
    - obs_streaming_changed
    依存: OBS-201
```

### SESSION_MONITOR タスク

```
[ ] MON-201: CPU/メモリ監視 (M)
    ファイル: src-tauri/src/monitor/cpu.rs, memory.rs
    内容:
    - sysinfo統合
    - CPU使用率（全体・コア別）
    - メモリ使用量
    依存: CORE-103

[ ] MON-202: GPU監視 (L)
    ファイル: src-tauri/src/monitor/gpu.rs
    内容:
    - nvml-wrapper統合（※依存リクエスト必要）
    - GPU使用率、VRAM
    - エンコーダー使用率
    - フォールバック処理（GPU未検出時）
    依存: MON-201

[ ] MON-203: ネットワーク監視 (M)
    ファイル: src-tauri/src/monitor/network.rs
    内容: 帯域使用量測定
    依存: MON-201

[ ] MON-204: メトリクスポーリング (M)
    ファイル: src-tauri/src/monitor/collector.rs
    内容:
    - 1秒間隔のTokioタスク
    - MetricsSnapshot構造体
    依存: MON-201, MON-202, MON-203

[ ] MON-205: メトリクスイベント配信 (M)
    ファイル: src-tauri/src/monitor/events.rs
    内容: metrics_update イベント
    依存: MON-204

[ ] MON-206: 監視コマンド (S)
    ファイル: src-tauri/src/commands/monitor.rs
    内容: get_system_metrics, start/stop_metrics_stream
    依存: MON-204
```

### SESSION_CORE タスク（Phase 2）

```
[ ] CORE-201: OBSモジュール統合 (M)
    ファイル: src-tauri/src/lib.rs
    内容: OBSコマンドをTauriに登録
    依存: OBS-202

[ ] CORE-202: Monitorモジュール統合 (M)
    ファイル: src-tauri/src/lib.rs
    内容: 監視コマンドをTauriに登録
    依存: MON-206

[ ] CORE-203: アラート判定ロジック (M)
    ファイル: src-tauri/src/alerts.rs
    内容:
    - 閾値ベースルール
    - alert_triggeredイベント
    依存: MON-205
```

### SESSION_UI タスク（Phase 2）

```
[ ] UI-201: 接続設定画面 (M)
    ファイル: src/features/settings/ConnectionSettings.tsx
    内容: OBS接続フォーム
    依存: UI-104

[ ] UI-202: グランスビュー (M)
    ファイル: src/features/dashboard/GlanceView.tsx
    内容: 総合ステータス表示（緑/黄/赤）
    依存: UI-104, UI-106

[ ] UI-203: 概要ビュー (L)
    ファイル: src/features/dashboard/OverviewView.tsx
    内容: CPU/GPU/FPS/ビットレートカード
    依存: UI-106, MON-205

[ ] UI-204: 詳細ビュー (L)
    ファイル: src/features/dashboard/DetailView.tsx
    内容: Rechartsグラフ、時系列表示
    依存: UI-203

[ ] UI-205: アラート通知 (M)
    ファイル: src/features/alerts/AlertNotification.tsx
    内容: トースト通知、アラート履歴
    依存: CORE-203

[ ] UI-206: OBSステータスパネル (M)
    ファイル: src/features/obs/StatusPanel.tsx
    内容: 接続状態、配信状態表示
    依存: OBS-205
```

**待ち合わせ**: 統合ポイント #2（メトリクス配信基盤）

```
[ ] INT-010: イベント配信テスト (M)
    担当: SESSION_CORE
    内容: metrics_update, obs_connection_changed が正常に配信されるか

[ ] INT-011: フロントエンド受信テスト (M)
    担当: SESSION_UI
    内容: useTauriEvent でイベント受信、状態更新
```

---

## Phase 3: 統合・テスト

**並列度**: 2（SESSION_CORE + SESSION_UI）
**所要時間**: 2-3時間

### SESSION_CORE タスク

```
[ ] CORE-301: Rustユニットテスト (L)
    ファイル: src-tauri/src/*/tests.rs
    内容: エラー処理、設定読み書き
    依存: Phase 2完了

[ ] CORE-302: パフォーマンス確認 (M)
    内容:
    - CPU使用率 < 1%
    - メモリリークなし
    依存: CORE-301
```

### SESSION_UI タスク

```
[ ] UI-301: コンポーネントテスト (M)
    ファイル: src/**/*.test.tsx
    内容: Vitest + React Testing Library
    依存: Phase 2完了

[ ] UI-302: E2Eテスト (M)
    内容: 接続→監視→切断の一連フロー
    依存: UI-301

[ ] UI-303: アクセシビリティ確認 (S)
    内容: キーボード操作、ARIA属性
    依存: UI-301
```

### 全体統合テスト

```
[ ] INT-020: OBS実機テスト (L)
    担当: 全セッション
    内容:
    - OBS Studio起動
    - 接続→設定読み取り→監視開始
    - 配信開始/停止検出

[ ] INT-021: 長時間稼働テスト (M)
    担当: SESSION_CORE
    内容: 30分間連続稼働、リソース監視

[ ] INT-022: バグ修正 (可変)
    担当: 各セッション
```

---

## 依存関係サマリー

```
Phase 0 (COMMANDER)
    │
    ├──→ Phase 1 並列
    │    ├── SESSION_CORE: CORE-101 → 102 → 103 → 104
    │    └── SESSION_UI:   UI-101 → 103 → 104
    │                      UI-102 → 105 → 106
    │
    ├──→ 統合ポイント #1
    │
    ├──→ Phase 2 並列
    │    ├── SESSION_OBS:     OBS-201 → 202/203/204/205
    │    ├── SESSION_MONITOR: MON-201 → 202 → 204 → 205
    │    ├── SESSION_CORE:    CORE-201, 202, 203
    │    └── SESSION_UI:      UI-201 → 206
    │
    ├──→ 統合ポイント #2
    │
    └──→ Phase 3 テスト
```

---

## タスクサイズ目安

- **S (Small)**: 15-30分
- **M (Medium)**: 30-60分
- **L (Large)**: 1-2時間

---

## セッション別クイックリファレンス

### SESSION_COMMANDER（ホスト）
```bash
# Phase 0 実行
pnpm create tauri-app obs_optimizer -- --template react-ts
docker compose build
docker compose --profile parallel up -d
```

### SESSION_CORE（Docker内）
```bash
# 作業開始
cd /workspace/src-tauri
cargo build  # ビルド確認
```

### SESSION_UI（Docker内）
```bash
# 作業開始
cd /workspace
pnpm install
pnpm dev  # フロントエンド開発
```

### SESSION_OBS / SESSION_MONITOR（Docker内）
```bash
# 依存追加が必要な場合
# .claude/dependency-requests.md にリクエストを追記
```

---

*Version: 1.0.0*
*Created: 2024-12-20*
