# OBS配信最適化ツール: 並列セッション開発コンテキスト設計

## 1. セッション間依存関係マップ

### 1.1 セッション概要

| Session ID | 役割 | 主な責務 | 技術領域 |
|------------|------|----------|----------|
| SESSION_COMMANDER | オーケストレーション | Docker管理、セッション起動/停止、コンテキスト配布 | Docker, Shell |
| SESSION_CORE | バックエンド | Tauri/Rust、IPC、サービス層、データ永続化 | Rust, Tauri 2.x |
| SESSION_UI | フロントエンド | React/TypeScript、コンポーネント、状態管理 | React 18, TypeScript |
| SESSION_OBS | OBS連携 | obws経由のWebSocket、メトリクス取得、設定操作 | Rust, obws 0.14.0 |
| SESSION_MONITOR | システム監視 | sysinfo/NVML、CPU/GPU/メモリ監視 | Rust, sysinfo, nvml |

---

### 1.2 依存関係ダイアグラム

```
                    SESSION_COMMANDER
                          |
            +-------------+-------------+
            |             |             |
            v             v             v
     SESSION_CORE   SESSION_OBS   SESSION_MONITOR
            |             |             |
            +------+------+------+------+
                   |
                   v
              SESSION_UI
```

**依存関係の詳細:**

```
SESSION_UI ─depends on─> SESSION_CORE (IPC Commands)
SESSION_UI ─depends on─> contracts/types.ts (共有型定義)

SESSION_CORE ─depends on─> SESSION_OBS (OBS連携モジュール)
SESSION_CORE ─depends on─> SESSION_MONITOR (システム監視モジュール)
SESSION_CORE ─depends on─> contracts/ipc.rs (IPCコマンド定義)
SESSION_CORE ─depends on─> contracts/types.rs (共有型定義)

SESSION_OBS ─depends on─> contracts/obs_types.rs (OBS関連型定義)
SESSION_OBS ─depends on─> contracts/metrics.rs (メトリクス型定義)

SESSION_MONITOR ─depends on─> contracts/metrics.rs (メトリクス型定義)
SESSION_MONITOR ─depends on─> contracts/system_types.rs (システム情報型定義)
```

---

### 1.3 依存関係マトリクス

| From \ To | COMMANDER | CORE | UI | OBS | MONITOR |
|-----------|:---------:|:----:|:--:|:---:|:-------:|
| COMMANDER | - | 起動 | 起動 | 起動 | 起動 |
| CORE | なし | - | IPC提供 | 統合 | 統合 |
| UI | なし | IPC消費 | - | なし | なし |
| OBS | なし | モジュール | なし | - | なし |
| MONITOR | なし | モジュール | なし | なし | - |

---

## 2. タスク依存関係と並列実行可能性

### 2.1 Phase 1a: 基盤構築（並列度2）

```
                    [Phase 1a開始]
                          |
          +---------------+---------------+
          |                               |
          v                               v
   [T1: プロジェクト初期化]        [T2: 型定義作成]
   SESSION_CORE (BLOCKER)         SESSION_CORE + SESSION_UI
          |                               |
          +---------------+---------------+
                          |
          +---------------+---------------+
          |                               |
          v                               v
   [T3: OBS接続実装]              [T4: システム監視実装]
   SESSION_OBS                    SESSION_MONITOR
          |                               |
          +---------------+---------------+
                          |
                          v
                   [T5: 基本UI実装]
                   SESSION_UI
                          |
                          v
                   [Phase 1a完了]
```

#### ブロッカータスク識別

| タスク | Blocker? | 理由 | 待機セッション |
|--------|:--------:|------|----------------|
| T1: プロジェクト初期化 | **YES** | Cargo.toml, package.json, Tauri設定が必要 | 全セッション |
| T2: 型定義作成 | **YES** | IPC/APIの型が決まらないと実装開始不可 | CORE, UI, OBS, MONITOR |
| T3: OBS接続実装 | NO | 独立して開発可能 | なし |
| T4: システム監視実装 | NO | 独立して開発可能 | なし |
| T5: 基本UI実装 | NO | T2完了後に着手可能 | なし |

---

### 2.2 Phase 1b: 機能実装（並列度4）

```
[Phase 1b開始] ──────────────────────────────────────────────────────────────┐
      |                                                                       |
      +─────────────+─────────────+─────────────+─────────────+               |
      |             |             |             |             |               |
      v             v             v             v             v               |
[診断サービス] [アラート実装] [データ永続化] [ダッシュボード] [オンボーディング]  |
SESSION_CORE  SESSION_CORE  SESSION_CORE  SESSION_UI     SESSION_UI         |
      |             |             |             |             |               |
      +─────────────+─────────────+             +─────────────+               |
                    |                                   |                     |
                    v                                   v                     |
             [通知連携]                          [画面統合]                   |
             SESSION_CORE                        SESSION_UI                   |
                    |                                   |                     |
                    +───────────────────────────────────+                     |
                                      |                                       |
                                      v                                       |
                               [Phase 1b完了] ────────────────────────────────┘
```

#### 並列実行可能なタスクグループ

| グループ | タスク | セッション | 前提条件 |
|----------|--------|------------|----------|
| G1 | 診断サービス, アラート実装, データ永続化 | CORE | Phase 1a完了 |
| G2 | ダッシュボード, オンボーディング | UI | Phase 1a完了 + IPC定義 |
| G3 | 通知連携 | CORE | アラート実装完了 |
| G4 | 画面統合 | UI | ダッシュボード + オンボーディング完了 |

---

### 2.3 Phase 2: 自動化・高度分析（並列度4）

| タスク | セッション | 依存 | 並列可否 |
|--------|------------|------|:--------:|
| 設定ワンクリック適用 | CORE + OBS | Phase 1b | 可 |
| プロファイル管理 | CORE + UI | Phase 1b | 可 |
| 配信中モード | CORE + UI | Phase 1b | 可 |
| 自動ビットレート調整 | OBS | 設定適用完了 | 可 |
| 問題箇所特定 | CORE + MONITOR | データ永続化完了 | 可 |
| 原因分析 | CORE | 問題特定完了 | 不可 |
| 改善提案 | CORE + UI | 原因分析完了 | 不可 |
| 履歴比較 | UI | データ永続化完了 | 可 |

---

## 3. 共有コンテキスト初期化 (contracts/)

### 3.1 ディレクトリ構造

```
contracts/
├── rust/
│   ├── mod.rs                 # モジュール公開
│   ├── types.rs               # 共通型定義
│   ├── ipc.rs                 # IPCコマンド定義
│   ├── obs_types.rs           # OBS関連型
│   ├── metrics.rs             # メトリクス型
│   ├── system_types.rs        # システム情報型
│   ├── settings.rs            # 設定関連型
│   └── errors.rs              # 共通エラー型
│
├── typescript/
│   ├── index.ts               # エクスポート
│   ├── types.ts               # 共通型定義
│   ├── ipc.ts                 # IPCコマンド型
│   ├── obs.ts                 # OBS関連型
│   ├── metrics.ts             # メトリクス型
│   ├── settings.ts            # 設定関連型
│   └── errors.ts              # エラー型
│
└── schemas/
    ├── settings.schema.json   # 設定ファイルスキーマ
    ├── session.schema.json    # セッションデータスキーマ
    └── metrics.schema.json    # メトリクスデータスキーマ
```

---

### 3.2 最初に定義すべき内容（優先順位順）

#### Priority 1: 基盤型定義（Phase 1a開始前に必須）

```rust
// contracts/rust/types.rs

/// アプリケーション全体のステータス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Good,
    Warning,
    Error,
    Unknown,
}

/// 接続状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Reconnecting,
    Error,
}

/// 配信状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StreamingStatus {
    Idle,
    Starting,
    Streaming,
    Stopping,
    Recording,
}
```

#### Priority 2: メトリクス型定義

```rust
// contracts/rust/metrics.rs

/// 統合メトリクスデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: i64,  // Unix timestamp (ms)
    pub cpu: CpuMetrics,
    pub gpu: Option<GpuMetrics>,
    pub memory: MemoryMetrics,
    pub obs: Option<ObsMetrics>,
    pub network: Option<NetworkMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage_percent: f64,
    pub per_core: Vec<f64>,
    pub process_usage: f64,  // OBSプロセスのCPU使用率
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMetrics {
    pub vendor: String,      // "NVIDIA", "AMD", "Intel"
    pub name: String,
    pub usage_percent: f64,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub encoder_usage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub used_mb: u64,
    pub total_mb: u64,
    pub process_used_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsMetrics {
    pub active_fps: f64,
    pub render_skipped_frames: u64,
    pub render_total_frames: u64,
    pub output_skipped_frames: u64,
    pub output_total_frames: u64,
    pub average_frame_time_ms: f64,
    pub bitrate_kbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub bitrate_kbps: f64,
    pub target_bitrate_kbps: f64,
    pub congestion: f64,
    pub rtt_ms: Option<f64>,
}
```

#### Priority 3: IPCコマンド定義

```rust
// contracts/rust/ipc.rs

/// フロントエンドからのコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum IpcCommand {
    // 接続管理
    Connect { host: String, port: u16, password: Option<String> },
    Disconnect,

    // メトリクス
    GetMetrics,
    StartMonitoring { interval_ms: u32 },
    StopMonitoring,

    // 診断
    RunDiagnostics,
    GetRecommendations,

    // 設定
    GetSettings,
    UpdateSettings { settings: AppSettings },
    ApplyObsSettings { changes: Vec<SettingChange> },

    // セッション
    GetSessionHistory { limit: u32 },
    GetSessionDetail { session_id: i64 },
}

/// バックエンドからのイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum IpcEvent {
    // 接続
    ConnectionChanged { status: ConnectionStatus },

    // メトリクス
    MetricsUpdated { metrics: SystemMetrics },

    // アラート
    Alert { level: AlertLevel, message: String, action: Option<AlertAction> },

    // 配信
    StreamingStatusChanged { status: StreamingStatus },

    // 診断
    DiagnosticsCompleted { result: DiagnosticsResult },
}
```

#### Priority 4: 設定型定義

```rust
// contracts/rust/settings.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub version: String,
    pub connection: ConnectionSettings,
    pub monitoring: MonitoringSettings,
    pub alerts: AlertSettings,
    pub display: DisplaySettings,
    pub streaming_mode: StreamingModeSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSettings {
    pub host: String,
    pub port: u16,
    pub auto_connect: bool,
    pub reconnect_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSettings {
    pub update_interval_ms: u32,
    pub streaming_mode_interval_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSettings {
    pub enabled: bool,
    pub levels: AlertLevelSettings,
    pub thresholds: AlertThresholds,
    pub sound_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub cpu_warning: u8,      // default: 80
    pub cpu_critical: u8,     // default: 95
    pub gpu_warning: u8,      // default: 85
    pub gpu_critical: u8,     // default: 95
    pub frame_drop_warning: f64,   // default: 1.0
    pub frame_drop_critical: f64,  // default: 3.0
}
```

---

### 3.3 各セッションが参照すべきファイル

| Session | 必須ファイル | 参照目的 |
|---------|-------------|----------|
| SESSION_CORE | `contracts/rust/*` | 型定義、IPC実装 |
| SESSION_UI | `contracts/typescript/*`, `wireframe_spec.md` | 型定義、UI仕様 |
| SESSION_OBS | `contracts/rust/obs_types.rs`, `contracts/rust/metrics.rs` | OBS関連型 |
| SESSION_MONITOR | `contracts/rust/metrics.rs`, `contracts/rust/system_types.rs` | 監視関連型 |
| ALL | `requirements_v2.md` | 機能要件、非機能要件 |

---

## 4. Phase別セッション割り当て表

### 4.1 Phase 1a: プロトタイプ（MVP最小構成）

**期間目標:** 基本的な接続と監視機能の実証
**並列度:** 2（最初は1から開始し、段階的に2へ）

| Week | タスク | 担当Session | 並列可否 | 成果物 |
|------|--------|-------------|:--------:|--------|
| W1-D1 | プロジェクト初期化 | CORE | BLOCKER | Tauri 2.x + React 18 雛形 |
| W1-D1 | 型定義作成 | CORE | BLOCKER | contracts/rust/*, contracts/typescript/* |
| W1-D2-3 | OBS WebSocket接続 | OBS | 可 | F-101~104 実装 |
| W1-D2-3 | システム監視基盤 | MONITOR | 可 | F-114~115 実装 |
| W1-D4-5 | 基本UI（接続設定） | UI | - | F-121~122 実装 |
| W2-D1-2 | メトリクス取得統合 | CORE | - | F-111~113 統合 |
| W2-D3-5 | ダッシュボード | UI | - | グランスビュー + 概要ビュー |

**Phase 1a完了条件:**
- [ ] OBSに接続してメトリクスを取得できる
- [ ] CPU/GPU使用率をリアルタイム表示できる
- [ ] フレームドロップを検出・表示できる
- [ ] システムトレイに常駐できる

---

### 4.2 Phase 1b: MVP拡張

**期間目標:** 診断・分析機能の実装
**並列度:** 4（フルパラレル）

| タスク群 | 担当Session | 依存関係 | 成果物 |
|----------|-------------|----------|--------|
| 診断サービス | CORE | Phase 1a | F-201~206 |
| アラートシステム | CORE | Phase 1a | F-211~214 |
| データ永続化 | CORE | Phase 1a | F-221~223 (SQLite) |
| オンボーディング | UI | Phase 1a + IPC | F-231~235 |
| ダッシュボード詳細 | UI | Phase 1a + IPC | 詳細ビュー |
| 設定画面 | UI | Phase 1a + IPC | 接続/通知/表示設定 |

**並列作業マトリクス:**

| 時期 | CORE | UI | OBS | MONITOR |
|------|------|-----|-----|---------|
| 前半 | 診断サービス | オンボーディング | 設定読み取り拡張 | GPU詳細監視 |
| 前半 | アラートシステム | ダッシュボード詳細 | - | - |
| 後半 | データ永続化 | 設定画面 | - | - |
| 統合 | IPC統合 | 画面統合 | - | - |

**Phase 1b完了条件:**
- [ ] 初回起動時にオンボーディングが表示される
- [ ] OBS設定の読み取りと推奨値提案ができる
- [ ] 閾値超過時にアラートが表示される
- [ ] セッションログがSQLiteに保存される

---

### 4.3 Phase 2a: 自動化

**期間目標:** 設定の自動適用と配信中モード
**並列度:** 4

| タスク | 担当Session | 優先度 | 成果物 |
|--------|-------------|--------|--------|
| ワンクリック適用 | CORE + OBS | Must | F-301 |
| プロファイル管理 | CORE + UI | Must | F-302 |
| 配信中モード | CORE + UI | Must | F-303 |
| 自動ビットレート調整 | OBS | Should | F-304 |
| 競合プロセス検出 | MONITOR | Should | F-305 |

---

### 4.4 Phase 2b: 高度な分析

**期間目標:** 詳細分析とレポート機能
**並列度:** 3

| タスク | 担当Session | 優先度 | 成果物 |
|--------|-------------|--------|--------|
| 問題箇所特定 | CORE + MONITOR | Must | F-401 |
| 原因分析 | CORE | Must | F-402 |
| 改善提案 | CORE + UI | Must | F-403 |
| 履歴比較 | UI | Should | F-404 |
| エクスポート | CORE + UI | Could | F-405 |

---

### 4.5 Phase 3: 統合テスト

**並列度:** 2（テスト実行は並列、修正は逐次）

| テスト種別 | 担当Session | 対象 |
|------------|-------------|------|
| ユニットテスト | CORE + OBS + MONITOR | Rustモジュール |
| コンポーネントテスト | UI | Reactコンポーネント |
| 統合テスト | CORE | IPC、サービス間連携 |
| E2Eテスト | COMMANDER | 全体フロー |

---

## 5. コンテキスト伝播ルール

### 5.1 セッション間の情報共有

```
[SESSION_COMMANDER]
    │
    ├── contracts/ ディレクトリを初期生成
    │
    ├── 各セッションに配布するコンテキスト:
    │   ├── requirements_v2.md (readonly)
    │   ├── wireframe_spec.md (readonly)
    │   ├── session_context_design.md (readonly)
    │   └── contracts/* (共有、変更時は全セッションに通知)
    │
    └── 作業ログの収集:
        ├── CORE: src-tauri/ の変更
        ├── UI: src/ の変更
        ├── OBS: src-tauri/src/obs/ の変更
        └── MONITOR: src-tauri/src/monitor/ の変更
```

### 5.2 コンテキスト更新プロトコル

1. **型定義の変更** (`contracts/` 配下)
   - 変更者: 変更内容をコミットメッセージに明記
   - COMMANDER: 全セッションに変更を通知
   - 影響セッション: 30分以内に対応を確認

2. **IPC仕様の変更**
   - 変更者: CORE または UIのいずれか
   - 影響範囲: CORE ⟷ UI間のすべての通信
   - 対応: 両セッションで同期的に更新

3. **ブロッカー発生時**
   - 発見者: 即座にCOMMANDERに報告
   - COMMANDER: 依存セッションに通知、タスク再割り当て
   - 待機セッション: 並列可能な別タスクにスイッチ

---

## 6. Quick Reference

### 6.1 セッション起動コマンド

```bash
# SESSION_COMMANDER
docker-compose up -d commander

# SESSION_CORE
docker exec -it obs-optimizer-core bash
cd /app && cargo build

# SESSION_UI
docker exec -it obs-optimizer-ui bash
cd /app && npm run dev

# SESSION_OBS
docker exec -it obs-optimizer-core bash
cargo test --package obs_connector

# SESSION_MONITOR
docker exec -it obs-optimizer-core bash
cargo test --package system_monitor
```

### 6.2 ヘルスチェック

| Session | ヘルスチェック方法 | 成功条件 |
|---------|-------------------|----------|
| CORE | `cargo check` | コンパイルエラーなし |
| UI | `npm run type-check` | TypeScriptエラーなし |
| OBS | `cargo test obs::` | テスト全パス |
| MONITOR | `cargo test monitor::` | テスト全パス |

### 6.3 エスカレーションパス

```
問題発生
    │
    ├─ 自己解決可能 → セッション内で解決
    │
    ├─ 他セッションに影響 → COMMANDER経由で調整
    │
    └─ 仕様不明 → requirements_v2.md参照 → なければCOMMANDER経由で確認
```

---

## 7. 成功指標（セッション別）

| Session | KPI | 目標値 |
|---------|-----|--------|
| CORE | IPCレスポンス時間 | < 100ms |
| UI | レンダリング時間 | < 16ms (60fps) |
| OBS | メトリクス取得間隔 | 1秒 |
| MONITOR | CPU使用率（監視処理） | < 1% |
| 全体 | 統合テストパス率 | 100% |
