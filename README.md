# OBS配信最適化ツール (OBS Optimizer)

<div align="center">

![OBS Optimizer](https://via.placeholder.com/800x400/1e293b/64748b?text=OBS+Optimizer+Screenshot)

**OBS Studioの設定を自動最適化し、リアルタイムでシステムを監視するデスクトップアプリケーション**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.x-24C8D8.svg)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-61DAFB.svg)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.x-3178C6.svg)](https://www.typescriptlang.org/)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)

[機能](#機能一覧) •
[インストール](#インストール方法) •
[開発環境](#開発環境セットアップ) •
[ビルド](#ビルド方法) •
[テスト](#テスト実行) •
[ドキュメント](#関連ドキュメント)

</div>

---

## プロジェクト概要

OBS Optimizer は、配信者がOBS Studioの複雑な設定を手動で調整する手間を省き、システムリソースをリアルタイムで監視しながら、最適なストリーミング品質を実現するためのデスクトップアプリケーションです。

### 特徴

- **自動最適化**: システムスペックと配信プラットフォームに基づいた設定の自動推奨
- **リアルタイム監視**: CPU、GPU、メモリ、ネットワーク帯域の継続的なモニタリング
- **OBS WebSocket連携**: OBS Studio 28.0以降との双方向通信
- **プロファイル管理**: 配信スタイル別の設定保存・切り替え
- **アラート機能**: リソース使用率の閾値超過時の通知
- **詳細分析**: 現在の設定の問題点を診断し、改善案を提示

---

## 機能一覧

### コア機能

| 機能 | 説明 | ステータス |
|------|------|-----------|
| **OBS接続管理** | WebSocketによるOBS Studio連携 | 実装済み |
| **システム監視** | CPU/GPU/メモリ/ネットワークのリアルタイム監視 | 実装済み |
| **設定分析** | 現在のOBS設定の診断と問題点の検出 | 実装済み |
| **自動最適化** | プリセット・カスタム推奨設定の生成 | 実装済み |
| **プロファイル管理** | 設定の保存・復元・切り替え | 実装済み |
| **アラート機能** | メトリクス閾値超過時の警告 | 実装済み |
| **設定エクスポート** | 推奨設定のJSON/テキスト出力 | 実装済み |
| **履歴管理** | 過去の最適化履歴の閲覧 | 実装済み |

### 監視メトリクス

- **CPU**: 全体使用率、コア別使用率
- **GPU**: 使用率、メモリ使用量、温度、エンコーダ使用率 (NVIDIA)
- **メモリ**: 使用量、空き容量、使用率
- **ネットワーク**: アップロード/ダウンロード速度
- **OBSステータス**: 配信/録画状態、フレームドロップ、CPU/メモリ使用率

### 最適化プリセット

- **品質重視 (Quality)**: 高ビットレート、低速エンコーダプリセット
- **バランス (Balanced)**: 品質とパフォーマンスの均衡
- **パフォーマンス重視 (Performance)**: 低負荷、高速エンコーダプリセット

### カスタム推奨

配信プラットフォーム（YouTube、Twitch、その他）、配信スタイル（ゲーム、雑談、クリエイティブ、音楽）、ネットワーク速度に基づいたカスタマイズ設定。

---

## 必要環境

### 実行環境

| 項目 | 要件 |
|------|------|
| **OS** | Windows 10 / Windows 11 (64-bit) |
| **OBS Studio** | バージョン 28.0 以降 |
| **WebSocket** | OBS WebSocket プラグイン有効化 |
| **メモリ** | 最低 4GB RAM (推奨 8GB 以上) |
| **ディスプレイ** | 解像度 1280x720 以上 |

### 開発環境

| ツール | バージョン |
|--------|-----------|
| **Node.js** | 18.x 以降 |
| **pnpm** | 8.x 以降 |
| **Rust** | 1.70 以降 (Edition 2021) |
| **Cargo** | 最新版 |
| **Docker** | 20.x 以降 (オプション) |

---

## インストール方法

### エンドユーザー向け

1. [Releases](https://github.com/yourusername/obs_optimizer/releases) ページから最新版のインストーラーをダウンロード
2. `obs-optimizer-setup-x.x.x.exe` を実行
3. インストーラーの指示に従ってセットアップ
4. OBS Studio で WebSocket プラグインを有効化
   - OBS Studio → ツール → WebSocket Server Settings
   - "Enable WebSocket server" にチェック
5. OBS Optimizer を起動し、接続設定を入力

---

## 開発環境セットアップ

### 1. リポジトリのクローン

```bash
git clone https://github.com/yourusername/obs_optimizer.git
cd obs_optimizer
```

### 2. 依存関係のインストール

#### フロントエンド (Node.js / pnpm)

```bash
# pnpmのインストール（未導入の場合）
npm install -g pnpm

# 依存パッケージのインストール
pnpm install
```

#### バックエンド (Rust / Cargo)

```bash
cd src-tauri
cargo build
cd ..
```

### 3. 環境設定（オプション）

プロジェクトルートに `.env` ファイルを作成（`.env.example` を参考）:

```bash
cp .env.example .env
```

### 4. 開発サーバーの起動

```bash
pnpm tauri dev
```

---

## ビルド方法

### 開発ビルド

```bash
pnpm tauri dev
```

自動的に以下が実行されます:
- Vite開発サーバー起動 (ホットリロード有効)
- Rustバックエンドのビルド
- Tauriアプリケーションの起動

### プロダクションビルド

```bash
pnpm tauri build
```

ビルド成果物は `src-tauri/target/release/` に生成されます:
- Windows: `obs_optimizer_app.exe`
- インストーラー: `src-tauri/target/release/bundle/nsis/`

### フロントエンドのみビルド

```bash
pnpm build
```

---

## テスト実行

### クイックスタート

```bash
# Dockerを使用した全テスト実行（推奨）
docker compose -f docker-compose.test.yml run --rm test-all
```

### ローカル環境でのテスト

#### フロントエンドテスト

```bash
# 全テスト実行
pnpm test

# ウォッチモード
pnpm test

# カバレッジ付き
pnpm test:coverage

# UIモード
pnpm test:ui
```

#### バックエンド (Rust) テスト

```bash
cd src-tauri

# 全テスト実行
cargo test

# 特定モジュールのテスト
cargo test services::alerts

# 統合テスト
cargo test --test '*'
```

### Docker環境でのテスト

詳細は [TESTING.md](/home/yskn/git/obs_optimizer/TESTING.md) を参照してください。

| コマンド | 説明 |
|---------|------|
| `docker compose -f docker-compose.test.yml run --rm test-all` | 全テスト実行 |
| `docker compose -f docker-compose.test.yml run --rm test-rust` | Rustテストのみ |
| `docker compose -f docker-compose.test.yml run --rm test-frontend` | フロントエンドテストのみ |
| `docker compose -f docker-compose.test.yml run --rm test-shell` | 対話シェル (デバッグ用) |

#### テスト統計 (2025-12-23時点)

- **Rustバックエンド**: 242テスト
- **Reactフロントエンド**: 272テスト
- **合計**: 514テスト

---

## コード品質チェック

### Lint

```bash
# フロントエンド (ESLint)
pnpm lint
pnpm lint:fix

# バックエンド (Clippy)
cd src-tauri
cargo clippy -- -D warnings
cargo clippy --fix
```

### フォーマット

```bash
# フロントエンド (Prettier)
pnpm format
pnpm format:check

# バックエンド (rustfmt)
cd src-tauri
cargo fmt
cargo fmt -- --check
```

### 型チェック

```bash
pnpm typecheck
```

---

## プロジェクト構造

```
obs_optimizer/
├── src/                          # Reactフロントエンド
│   ├── components/               # 共通UIコンポーネント
│   │   ├── common/               # ボタン、入力、モーダル等
│   │   └── layout/               # ヘッダー、サイドバー、レイアウト
│   ├── features/                 # 機能別モジュール
│   │   ├── alerts/               # アラート管理
│   │   ├── analysis/             # 設定分析
│   │   ├── diagnostics/          # システム診断
│   │   ├── export/               # 設定エクスポート
│   │   ├── history/              # 履歴管理
│   │   ├── monitor/              # メトリクス監視
│   │   ├── obs/                  # OBS接続管理
│   │   ├── onboarding/           # 初回セットアップ
│   │   ├── optimization/         # 最適化エンジン
│   │   ├── profiles/             # プロファイル管理
│   │   └── streaming/            # ストリーミングモード
│   ├── hooks/                    # カスタムReactフック
│   ├── stores/                   # Zustand状態管理
│   ├── types/                    # TypeScript型定義
│   ├── utils/                    # ユーティリティ関数
│   └── tests/                    # テスト用モック・ユーティリティ
├── src-tauri/                    # Rustバックエンド
│   ├── src/
│   │   ├── commands/             # Tauriコマンド定義
│   │   │   ├── alerts.rs         # アラート関連コマンド
│   │   │   ├── analyzer.rs       # 設定分析コマンド
│   │   │   ├── config.rs         # 設定管理コマンド
│   │   │   ├── export.rs         # エクスポートコマンド
│   │   │   ├── optimization.rs   # 最適化コマンド
│   │   │   ├── optimizer.rs      # 推奨計算コマンド
│   │   │   ├── profiles.rs       # プロファイルコマンド
│   │   │   └── streaming_mode.rs # ストリーミングモードコマンド
│   │   ├── monitor/              # システム監視
│   │   │   ├── cpu.rs            # CPU監視
│   │   │   ├── gpu.rs            # GPU監視 (NVIDIA)
│   │   │   ├── memory.rs         # メモリ監視
│   │   │   ├── network.rs        # ネットワーク監視
│   │   │   └── process.rs        # プロセス監視
│   │   ├── obs/                  # OBS WebSocket連携
│   │   │   ├── client.rs         # OBSクライアント
│   │   │   ├── error.rs          # OBSエラー処理
│   │   │   ├── events.rs         # イベントハンドリング
│   │   │   └── settings.rs       # 設定取得・適用
│   │   ├── services/             # ビジネスロジック
│   │   │   ├── alerts.rs         # アラートサービス
│   │   │   ├── analyzer.rs       # 分析サービス
│   │   │   ├── exporter.rs       # エクスポートサービス
│   │   │   ├── obs.rs            # OBSサービス
│   │   │   ├── optimizer.rs      # 最適化サービス
│   │   │   ├── streaming_mode.rs # ストリーミングモードサービス
│   │   │   └── system.rs         # システムサービス
│   │   ├── storage/              # データ永続化 (SQLite)
│   │   ├── testing/              # テストユーティリティ
│   │   ├── error.rs              # エラー定義
│   │   └── lib.rs                # エントリーポイント
│   ├── Cargo.toml                # Rust依存関係
│   └── tauri.conf.json           # Tauri設定
├── contracts/                    # Backend/Frontend契約
│   ├── api.md                    # Tauriコマンド定義
│   ├── events.md                 # Tauriイベント定義
│   └── types.md                  # 共通型定義
├── tests/                        # 統合テスト
├── docs/                         # 設計ドキュメント
├── .claude/                      # Claude Code設定
├── docker-compose.test.yml       # テスト用Docker設定
├── CLAUDE.md                     # 開発規約
├── TESTING.md                    # テスト実行ガイド
└── README.md                     # このファイル
```

---

## 技術スタック

### フロントエンド

| 技術 | バージョン | 用途 |
|------|-----------|------|
| **React** | 19.1.0 | UIフレームワーク |
| **TypeScript** | 5.8.3 | 型安全性 |
| **Zustand** | 4.4.0 | 状態管理 |
| **Tailwind CSS** | 最新 | スタイリング |
| **Vite** | 7.0.4 | ビルドツール |
| **Vitest** | 4.0.16 | テストランナー |
| **@tauri-apps/api** | 2.x | Tauri連携 |

### バックエンド

| 技術 | バージョン | 用途 |
|------|-----------|------|
| **Rust** | Edition 2021 | システムプログラミング |
| **Tauri** | 2.x | デスクトップアプリフレームワーク |
| **obws** | 0.14 | OBS WebSocketクライアント |
| **sysinfo** | 0.30 | システム情報取得 |
| **nvml-wrapper** | 0.10 | NVIDIA GPU監視 |
| **tokio** | 1.x | 非同期ランタイム |
| **rusqlite** | 0.31 | SQLiteデータベース |
| **serde** | 1.x | シリアライズ/デシリアライズ |

### 開発ツール

- **ESLint** - JavaScriptコード品質
- **Prettier** - コードフォーマット
- **Clippy** - Rustコード品質
- **rustfmt** - Rustコードフォーマット
- **Docker** - テスト環境の標準化

---

## アーキテクチャ

### レイヤー構造

```
┌─────────────────────────────────────┐
│   React UI Layer (src/)             │
│   - Components, Features, Stores    │
└──────────────┬──────────────────────┘
               │ invoke()
┌──────────────▼──────────────────────┐
│   Tauri Commands (src-tauri/commands/) │
│   - API Entry Points                │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│   Services Layer (src-tauri/services/) │
│   - Business Logic                  │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│   Infrastructure Layer              │
│   - OBS Client, Monitor, Storage    │
└─────────────────────────────────────┘
```

### 状態管理

- **Zustand**: クライアントサイド状態（UI状態、キャッシュ）
- **Tauri Events**: Backend → Frontend イベント通知
- **SQLite**: 永続化データ（プロファイル、履歴、設定）

---

## 開発規約

プロジェクト固有の開発ルールは [CLAUDE.md](/home/yskn/git/obs_optimizer/CLAUDE.md) を参照してください。

### 重要な原則

- **型安全性**: すべての型を明示的に定義
- **エラーハンドリング**: 非同期処理には必ずエラー処理
- **契約優先**: `contracts/api.md` が信頼の源泉
- **テストファースト**: 新機能には必ずテストを追加

### 禁止事項

- `unwrap()` / `expect()` の本番コード使用 (Rust)
- `any` / `@ts-ignore` の使用 (TypeScript)
- `tauri.conf.json` の編集（人間のみが管理）

---

## 関連ドキュメント

| ドキュメント | 説明 |
|-------------|------|
| [CLAUDE.md](/home/yskn/git/obs_optimizer/CLAUDE.md) | 開発規約・プロジェクト全体のルール |
| [TESTING.md](/home/yskn/git/obs_optimizer/TESTING.md) | テスト実行方法・Docker環境 |
| [contracts/api.md](/home/yskn/git/obs_optimizer/contracts/api.md) | Tauriコマンド定義 |
| [contracts/events.md](/home/yskn/git/obs_optimizer/contracts/events.md) | Tauriイベント定義 |
| [contracts/types.md](/home/yskn/git/obs_optimizer/contracts/types.md) | 共通型定義 |
| [LINT_QUICK_REFERENCE.md](/home/yskn/git/obs_optimizer/LINT_QUICK_REFERENCE.md) | Lint設定リファレンス |

---

## トラブルシューティング

### OBSに接続できない

1. OBS Studio でWebSocketサーバーが有効か確認
2. ポート番号が正しいか確認（デフォルト: 4455）
3. パスワード設定がある場合は正しく入力されているか確認

### GPU情報が取得できない

- NVIDIA GPU以外の場合、GPU監視機能は制限されます
- NVIDIA GPU Driversが最新か確認してください

### ビルドエラー

```bash
# キャッシュをクリア
pnpm store prune
rm -rf node_modules
pnpm install

# Rustキャッシュクリア
cd src-tauri
cargo clean
cargo build
```

### テストが失敗する

```bash
# Dockerキャッシュをクリアして再実行
docker compose -f docker-compose.test.yml down -v
docker compose -f docker-compose.test.yml build --no-cache
docker compose -f docker-compose.test.yml run --rm test-all
```

---

## コントリビューション

コントリビューションは歓迎します。以下のガイドラインに従ってください:

1. このリポジトリをフォーク
2. フィーチャーブランチを作成 (`git checkout -b feature/amazing-feature`)
3. コミット (`git commit -m 'Add amazing feature'`)
4. プッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

### コミットメッセージ規約

```
<type>: <subject>

<body>
```

**Type:**
- `feat`: 新機能
- `fix`: バグ修正
- `docs`: ドキュメント変更
- `refactor`: リファクタリング
- `test`: テスト追加・修正
- `chore`: ビルド・設定変更

---

## ライセンス

このプロジェクトは MIT ライセンスの下で公開されています。詳細は [LICENSE](LICENSE) ファイルを参照してください。

---

## クレジット

### 使用ライブラリ

- [Tauri](https://tauri.app/) - デスクトップアプリフレームワーク
- [React](https://react.dev/) - UIライブラリ
- [obws](https://github.com/dnaka91/obws) - OBS WebSocketクライアント
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - システム情報取得
- [Zustand](https://github.com/pmndrs/zustand) - 状態管理

### 作者

開発者: [yskn](https://github.com/yskn)

---

## サポート

問題が発生した場合や機能リクエストがある場合は、[Issues](https://github.com/yourusername/obs_optimizer/issues) ページで報告してください。

---

<div align="center">

**OBS Optimizer で快適な配信ライフを！**

Made with ❤️ by the OBS Optimizer Team

</div>
