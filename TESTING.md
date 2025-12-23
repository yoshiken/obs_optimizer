# OBS Optimizer テスト実行ガイド

> ホスト環境を汚さずにDockerでテストを実行する方法

## クイックスタート

```bash
# 全テスト実行（推奨）
docker compose -f docker-compose.test.yml run --rm test-all

# Rustバックエンドのみ
docker compose -f docker-compose.test.yml run --rm test-rust

# フロントエンドのみ
docker compose -f docker-compose.test.yml run --rm test-frontend
```

---

## 利用可能なテストサービス

| サービス名 | 説明 | 実行時間目安 |
|-----------|------|-------------|
| `test-all` | Rust + Frontend 全テスト | 2-5分 |
| `test-rust` | Rustユニットテスト | 1-2分 |
| `test-rust-integration` | Rust統合テスト | 1-2分 |
| `test-frontend` | Vitest フロントエンドテスト | 30秒-1分 |
| `test-frontend-coverage` | フロントエンドカバレッジ | 1分 |
| `test-shell` | 対話シェル（デバッグ用） | - |

---

## 詳細な使用方法

### 1. 全テスト実行

```bash
docker compose -f docker-compose.test.yml run --rm test-all
```

**出力例:**
```
========================================
  OBS Optimizer Test Runner
========================================

[1/3] Installing frontend dependencies...
[2/3] Running Rust tests...
   Compiling obs_optimizer_app_lib v0.1.0
   Running unittests src/lib.rs
running 243 tests
...
test result: ok. 243 passed; 0 failed

[3/3] Running frontend tests...
 ✓ src/stores/alertStore.test.ts (23 tests)
 ✓ src/stores/obsStore.test.ts (22 tests)
...
Tests: 272 passed

========================================
  All tests completed successfully!
========================================
```

### 2. Rustテストのみ

```bash
# ユニットテスト
docker compose -f docker-compose.test.yml run --rm test-rust

# 統合テスト
docker compose -f docker-compose.test.yml run --rm test-rust-integration
```

**特定のテストを実行:**
```bash
docker compose -f docker-compose.test.yml run --rm test-shell
# コンテナ内で:
cd src-tauri
cargo test test_alert_triggered  # 特定のテスト
cargo test services::alerts      # 特定のモジュール
```

### 3. フロントエンドテスト

```bash
# 通常実行
docker compose -f docker-compose.test.yml run --rm test-frontend

# カバレッジ付き
docker compose -f docker-compose.test.yml run --rm test-frontend-coverage
```

**カバレッジレポート:**
カバレッジは `coverage/` ディレクトリに生成されます:
- `coverage/index.html` - HTMLレポート
- `coverage/coverage-final.json` - JSON形式

### 4. 対話シェル（デバッグ用）

```bash
docker compose -f docker-compose.test.yml run --rm test-shell
```

コンテナ内で自由にコマンドを実行できます:
```bash
# Rustテスト
cd src-tauri && cargo test

# フロントエンドテスト
pnpm test

# Lint
pnpm lint
cargo clippy

# 特定のテストファイル
pnpm test -- src/stores/alertStore.test.ts
```

---

## 初回ビルド

初回実行時はDockerイメージのビルドが必要です（約5-10分）:

```bash
# イメージをビルド
docker compose -f docker-compose.test.yml build

# または、run時に自動ビルド
docker compose -f docker-compose.test.yml run --rm test-all
```

### キャッシュについて

以下のボリュームでキャッシュを共有し、2回目以降の実行を高速化:
- `cargo-cache` - Cargo registryキャッシュ
- `cargo-git` - Cargo gitキャッシュ
- `pnpm-cache` - pnpmパッケージキャッシュ

**キャッシュをクリアする場合:**
```bash
docker compose -f docker-compose.test.yml down -v
```

---

## CI/CD での使用

### GitHub Actions

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run all tests
        run: docker compose -f docker-compose.test.yml run --rm test-all
```

### ローカルでのCI再現

```bash
# CI環境と同じ条件でテスト
docker compose -f docker-compose.test.yml run --rm -e CI=true test-all
```

---

## トラブルシューティング

### ビルドエラー

```bash
# キャッシュをクリアして再ビルド
docker compose -f docker-compose.test.yml build --no-cache
```

### 権限エラー

```bash
# ファイル権限を修正
sudo chown -R $(id -u):$(id -g) .
```

### メモリ不足

Dockerのメモリ制限を増やしてください（推奨: 4GB以上）

### cargo testがハングする

```bash
# シングルスレッドで実行
docker compose -f docker-compose.test.yml run --rm test-shell
cd src-tauri && cargo test -- --test-threads=1
```

---

## テスト統計

### 現在のテスト数（2025-12-23時点）

| カテゴリ | テスト数 |
|---------|---------|
| Rustバックエンド | 242 |
| Reactフロントエンド | 272 |
| **合計** | **514** |

### テスト実行時間目安

| 環境 | test-all | test-rust | test-frontend |
|-----|----------|-----------|---------------|
| M1 Mac | ~2分 | ~1分 | ~30秒 |
| Intel Mac | ~3分 | ~2分 | ~45秒 |
| Linux (GitHub Actions) | ~5分 | ~3分 | ~1分 |

---

## 関連ドキュメント

- [TEST_PROGRESS.md](./TEST_PROGRESS.md) - テスト進捗と詳細
- [TEST_COVERAGE_SUMMARY.md](./TEST_COVERAGE_SUMMARY.md) - カバレッジサマリー
- [CLAUDE.md](./CLAUDE.md) - 開発規約

---

*Last Updated: 2025-12-23*
