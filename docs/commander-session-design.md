# 並列開発司令塔セッション設計

> 複数のClaude Codeセッションが並列動作する環境での依存関係管理と調整を行う司令塔セッションの設計ドキュメント

---

## 1. 概要

### 1.1 背景と課題

複数のClaude Codeセッション（SESSION_CORE, SESSION_UI, SESSION_OBS, SESSION_MONITOR）がDocker内で並列動作している際、以下の課題が発生する:

1. **依存関係の不足**: 作業中に「このパッケージが足りない」と判明
2. **Dockerfileの競合**: 複数セッションが同時にDockerfileを編集するとコンフリクト
3. **依頼の順序管理**: 緊急度に応じた処理が必要

### 1.2 解決策

**SESSION_COMMANDER（司令塔セッション）** を導入し、以下を実現:

- 依存関係リクエストの一元管理
- Dockerfile/設定ファイルの単一編集者ルール
- 優先度付きキュー処理

---

## 2. 依存関係リクエストファイル設計

### 2.1 ファイル構成

```
.claude/
├── dependency-requests.md      # リクエストキュー（司令塔が監視）
├── dependency-history.md       # 処理済みリクエストのログ
└── commander-status.md         # 司令塔の状態
```

### 2.2 `.claude/dependency-requests.md` フォーマット

```markdown
# Dependency Requests Queue

> このファイルは SESSION_COMMANDER が監視する。
> 各セッションは新規リクエストを末尾に追加する。
> 司令塔が処理後、リクエストを history に移動する。

---

## Pending Requests

### REQ-001
- **ID**: REQ-001
- **From**: SESSION_OBS
- **Priority**: critical
- **Type**: apt-package
- **Requested**: 2024-12-20T21:30:00+09:00
- **Package**: `libobs-dev`
- **Reason**: OBS WebSocket ライブラリのビルドに必要
- **Status**: pending

---

### REQ-002
- **ID**: REQ-002
- **From**: SESSION_MONITOR
- **Priority**: normal
- **Type**: cargo-crate
- **Requested**: 2024-12-20T21:35:00+09:00
- **Package**: `nvml-wrapper = "0.9"`
- **Reason**: NVIDIA GPU監視機能の実装に必要
- **Status**: pending

---

### REQ-003
- **ID**: REQ-003
- **From**: SESSION_UI
- **Priority**: normal
- **Type**: npm-package
- **Requested**: 2024-12-20T21:40:00+09:00
- **Package**: `@tanstack/react-query@^5`
- **Reason**: サーバー状態管理の改善
- **Status**: pending

---

## Request Template

新規リクエスト追加時は以下をコピーして末尾に追加:

\```markdown
### REQ-XXX
- **ID**: REQ-XXX
- **From**: SESSION_NAME
- **Priority**: critical | normal
- **Type**: apt-package | cargo-crate | npm-package | dockerfile-config | env-var
- **Requested**: [ISO8601 timestamp]
- **Package**: [パッケージ名とバージョン]
- **Reason**: [なぜ必要か、どの機能で使うか]
- **Status**: pending
\```
```

### 2.3 リクエストタイプ定義

| Type | 対象 | 編集ファイル | 備考 |
|------|------|-------------|------|
| `apt-package` | Ubuntuパッケージ | `Dockerfile` | RUN apt-get に追加 |
| `cargo-crate` | Rustクレート | `src-tauri/Cargo.toml` | dependencies に追加 |
| `npm-package` | npmパッケージ | `package.json` | dependencies/devDependencies |
| `dockerfile-config` | Dockerfile設定 | `Dockerfile` | ENV, EXPOSE など |
| `env-var` | 環境変数 | `docker-compose.yml` | environment セクション |

### 2.4 優先度定義

| Priority | 意味 | 処理目標 |
|----------|------|---------|
| `critical` | 作業がブロックされている | 即時処理（他の作業を中断） |
| `normal` | 必要だが代替手段がある | キュー順に処理 |

---

## 3. SESSION_COMMANDER 責務定義

### 3.1 責務一覧

```
SESSION_COMMANDER の責務
├── 1. リクエスト監視
│   ├── .claude/dependency-requests.md を定期的に確認
│   └── 新規リクエストの検出
├── 2. 優先度判定
│   ├── critical リクエストを優先
│   └── 同一優先度は FIFO
├── 3. 実装指示
│   ├── Dockerfile 変更が必要な場合 → 自ら実装
│   ├── Cargo.toml 変更 → SESSION_CORE に指示
│   └── package.json 変更 → SESSION_UI に指示
├── 4. 検証
│   ├── docker build の成功確認
│   └── 変更後の動作確認
└── 5. 通知
    ├── リクエスト元セッションに完了通知
    └── 全セッションに影響する変更は全体通知
```

### 3.2 処理フロー

```
[リクエスト検出]
      │
      v
[優先度チェック] ─── critical ──> [即時処理開始]
      │                              │
   normal                            │
      │                              │
      v                              v
[キュー順序決定]            [対象ファイル特定]
      │                              │
      v                              │
[処理待機]                           │
      │                              │
      v                              v
[対象ファイルロック取得] <───────────┘
      │
      v
[変更実装]
      │
      ├── Dockerfile → 直接編集
      ├── Cargo.toml → SESSION_CORE に Integration Request
      └── package.json → SESSION_UI に Integration Request
      │
      v
[ビルド検証]
      │
      ├── 成功 → [リクエストを history に移動]
      │              │
      │              v
      │         [完了通知]
      │
      └── 失敗 → [ロールバック]
                     │
                     v
                [エラー報告]
```

### 3.3 ロックファイル追加

`.claude-locks/` に以下を追加:

```
.claude-locks/
├── dockerfile.lock          # Dockerfile 編集中
├── docker-compose.lock      # docker-compose.yml 編集中
├── cargo-toml.lock          # Cargo.toml 編集中（SESSION_CORE専用）
└── package-json.lock        # package.json 編集中（SESSION_UI専用）
```

### 3.4 司令塔ステータスファイル

`.claude/commander-status.md`:

```markdown
# Commander Status

## Current State
- **Status**: active | processing | idle
- **Last Check**: [ISO8601 timestamp]
- **Processing**: REQ-XXX | none

## Queue Summary
- **Pending Critical**: 0
- **Pending Normal**: 2
- **Processed Today**: 5

## Recent Actions
| Time | Request | Action | Result |
|------|---------|--------|--------|
| 21:45 | REQ-001 | Added libobs-dev to Dockerfile | Success |
| 21:30 | REQ-000 | Initial setup | Success |
```

---

## 4. docker-compose.yml 追加サービス

```yaml
  # ============================================
  # SESSION_COMMANDER: Orchestration & Dependencies
  # ============================================

  session-commander:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: claude-session-commander

    volumes:
      - .:/workspace
      - cargo-cache:/usr/local/cargo/registry
      - cargo-git:/usr/local/cargo/git
      - pnpm-cache:/home/developer/.local/share/pnpm
      - /var/run/docker.sock:/var/run/docker.sock  # Docker操作用

    environment:
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY:-}
      - CLAUDE_CODE_API_KEY=${CLAUDE_CODE_API_KEY:-}
      - SESSION_NAME=SESSION_COMMANDER
      - COMMANDER_MODE=true

    stdin_open: true
    tty: true
    working_dir: /workspace

    command: ["claude"]

    # 他セッションより先に起動
    profiles:
      - commander
      - parallel

    # 特権: Dockerビルド実行用
    privileged: true
```

### 4.1 profiles 更新

```yaml
# 起動パターン

# 1. 司令塔のみ
docker compose --profile commander up -d

# 2. 司令塔 + 全並列セッション
docker compose --profile parallel up -d

# 3. 段階的起動（推奨）
docker compose --profile commander up -d
sleep 5
docker compose --profile core --profile ui up -d
```

---

## 5. CLAUDE.md 追記内容

以下を既存のCLAUDE.mdに追加:

```markdown
---

## 13. 司令塔セッション（SESSION_COMMANDER）

### 13.1 概要

SESSION_COMMANDER は依存関係リクエストを一元管理する特別なセッション。
Dockerfile、docker-compose.yml の編集権限を持つ唯一のセッション。

### 13.2 依存関係リクエスト方法

作業中にパッケージが不足した場合:

1. **直接編集禁止**: Dockerfile, Cargo.toml, package.json を直接編集しない
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

### 13.3 リクエスト完了の確認

- `.claude/dependency-requests.md` から自分のリクエストが消える
- `.claude/dependency-history.md` に移動されている
- `.claude/commander-status.md` に処理結果が記録

### 13.4 緊急時の対応

司令塔が応答しない場合:
1. `.claude/commander-status.md` の Last Check を確認
2. 10分以上更新がない場合、人間に報告

### 13.5 禁止事項（全セッション共通）

- `Dockerfile` の直接編集（SESSION_COMMANDER のみ許可）
- `docker-compose.yml` の直接編集（SESSION_COMMANDER のみ許可）
- 他セッションのリクエストの変更・削除

---

## 14. セッション起動順序

### 14.1 推奨起動順序

```bash
# Step 1: 司令塔を先に起動
docker compose --profile commander up -d

# Step 2: 司令塔の起動確認
docker logs claude-session-commander

# Step 3: 作業セッションを起動
docker compose --profile core --profile ui up -d

# Step 4: 必要に応じて追加セッション
docker compose --profile obs --profile monitor up -d
```

### 14.2 全セッション起動（一括）

```bash
docker compose --profile parallel up -d
```

注意: parallel プロファイルには commander も含まれる
```

---

## 6. 初期化ファイル

### 6.1 `.claude/dependency-requests.md` 初期状態

```markdown
# Dependency Requests Queue

> このファイルは SESSION_COMMANDER が監視する。
> 各セッションは新規リクエストを末尾に追加する。
> 司令塔が処理後、リクエストを history に移動する。

---

## Pending Requests

（現在リクエストなし）

---

## Request Template

新規リクエスト追加時は以下をコピーして末尾に追加:

\```markdown
### REQ-XXX
- **ID**: REQ-XXX
- **From**: SESSION_NAME
- **Priority**: critical | normal
- **Type**: apt-package | cargo-crate | npm-package | dockerfile-config | env-var
- **Requested**: [ISO8601 timestamp]
- **Package**: [パッケージ名とバージョン]
- **Reason**: [なぜ必要か、どの機能で使うか]
- **Status**: pending
\```
```

### 6.2 `.claude/dependency-history.md` 初期状態

```markdown
# Dependency Request History

> 処理済みリクエストのログ

---

## Processed Requests

（履歴なし）

---

## Log Format

\```markdown
### REQ-XXX [COMPLETED/REJECTED]
- **From**: SESSION_NAME
- **Type**: type
- **Package**: package-name
- **Processed**: [ISO8601 timestamp]
- **By**: SESSION_COMMANDER
- **Result**: success | failed | rejected
- **Notes**: [追加情報]
\```
```

### 6.3 `.claude/commander-status.md` 初期状態

```markdown
# Commander Status

## Current State
- **Status**: idle
- **Last Check**: （起動時に設定）
- **Processing**: none

## Queue Summary
- **Pending Critical**: 0
- **Pending Normal**: 0
- **Processed Today**: 0

## Recent Actions

| Time | Request | Action | Result |
|------|---------|--------|--------|
| - | - | - | - |
```

---

## 7. 運用シナリオ

### 7.1 通常フロー例

```
1. SESSION_OBS が OBS連携機能を実装中
2. obws クレートのビルドに libobs-dev が必要と判明
3. SESSION_OBS が dependency-requests.md にリクエスト追加:
   - ID: REQ-001
   - Priority: critical（作業ブロック）
   - Type: apt-package
   - Package: libobs-dev

4. SESSION_COMMANDER が検出
5. SESSION_COMMANDER が Dockerfile を編集:
   - libobs-dev を apt-get install に追加
6. docker build で検証
7. 成功 → リクエストを history に移動
8. SESSION_OBS に完了通知
9. SESSION_OBS が作業再開
```

### 7.2 Cargo.toml 変更フロー

```
1. SESSION_MONITOR が nvml-wrapper クレート追加をリクエスト
2. SESSION_COMMANDER が検出
3. SESSION_COMMANDER → SESSION_CORE に Integration Request:
   「Cargo.toml に nvml-wrapper = "0.9" を追加してください」
4. SESSION_CORE が Cargo.toml を編集
5. SESSION_CORE が cargo build で確認
6. SESSION_CORE → SESSION_COMMANDER に完了報告
7. SESSION_COMMANDER がリクエストを history に移動
8. SESSION_MONITOR に完了通知
```

---

## 8. 実装チェックリスト

### 8.1 ファイル作成

- [ ] `.claude/dependency-requests.md`
- [ ] `.claude/dependency-history.md`
- [ ] `.claude/commander-status.md`

### 8.2 設定変更

- [ ] `docker-compose.yml` に session-commander 追加
- [ ] `CLAUDE.md` にセクション13, 14 追加

### 8.3 ロックファイル追加

- [ ] `.claude-locks/dockerfile.lock` のルール追加
- [ ] `.claude-locks/docker-compose.lock` のルール追加

---

## 9. 拡張案（将来）

### 9.1 自動化

- ファイル監視スクリプト（inotifywait）で変更検知
- Slack/Discord 通知連携
- GitHub Issues との連携

### 9.2 依存関係解析

- パッケージの依存関係チェック
- バージョン競合の自動検出
- セキュリティ脆弱性スキャン

---

*Version: 1.0.0*
*Created: 2024-12-20*
