# OBS Optimizer - Makefile
# Docker を使用した開発・テスト・ビルドコマンド
#
# 使用方法:
#   make test        # 全テスト実行（Rust + Frontend）
#   make test-rust   # Rustテストのみ
#   make test-front  # フロントエンドテストのみ
#   make lint        # 静的解析（Clippy + ESLint）
#   make build       # ビルド
#   make clean       # キャッシュクリア
#   make shell       # 開発用シェルに入る

.PHONY: test test-rust test-front test-integration lint lint-rust lint-front build clean shell help

# デフォルトターゲット
.DEFAULT_GOAL := help

# Docker Compose ファイル
DC_TEST := docker compose -f docker-compose.test.yml

#------------------------------------------------------------------------------
# テスト
#------------------------------------------------------------------------------

## 全テスト実行（Rust + Frontend）
test:
	$(DC_TEST) run --rm test-all

## Rustユニットテスト
test-rust:
	$(DC_TEST) run --rm test-rust

## Rust統合テスト
test-integration:
	$(DC_TEST) run --rm test-rust-integration

## フロントエンドテスト
test-front:
	$(DC_TEST) run --rm test-frontend

## フロントエンドカバレッジ
test-coverage:
	$(DC_TEST) run --rm test-frontend-coverage

#------------------------------------------------------------------------------
# 静的解析
#------------------------------------------------------------------------------

## 全静的解析（Clippy + ESLint + TypeScript）
lint: lint-rust lint-front

## Rust静的解析（Clippy）
lint-rust:
	$(DC_TEST) run --rm test-rust bash -c "cargo clippy -- -D warnings"

## フロントエンド静的解析（ESLint + TypeScript）
lint-front:
	$(DC_TEST) run --rm test-frontend bash -c "pnpm install --frozen-lockfile && pnpm lint && pnpm tsc --noEmit"

#------------------------------------------------------------------------------
# ビルド
#------------------------------------------------------------------------------

## プロダクションビルド
build:
	$(DC_TEST) run --rm test-all bash -c "pnpm install --frozen-lockfile && pnpm tauri build"

## 開発ビルド確認（ビルド可能かチェック）
build-check:
	$(DC_TEST) run --rm test-rust bash -c "cargo check"
	$(DC_TEST) run --rm test-frontend bash -c "pnpm install --frozen-lockfile && pnpm tsc --noEmit"

#------------------------------------------------------------------------------
# ユーティリティ
#------------------------------------------------------------------------------

## 開発用シェル（デバッグ用）
shell:
	$(DC_TEST) run --rm test-shell

## Dockerイメージのリビルド
rebuild:
	$(DC_TEST) build --no-cache

## キャッシュクリア
clean:
	docker volume rm obs_optimizer_cargo-cache obs_optimizer_cargo-git obs_optimizer_pnpm-cache 2>/dev/null || true
	$(DC_TEST) down --volumes --remove-orphans

## フォーマット
fmt:
	$(DC_TEST) run --rm test-rust bash -c "cargo fmt"
	$(DC_TEST) run --rm test-frontend bash -c "pnpm install --frozen-lockfile && pnpm exec prettier --write src/"

## フォーマットチェック
fmt-check:
	$(DC_TEST) run --rm test-rust bash -c "cargo fmt -- --check"
	$(DC_TEST) run --rm test-frontend bash -c "pnpm install --frozen-lockfile && pnpm exec prettier --check src/"

#------------------------------------------------------------------------------
# CI/CD用
#------------------------------------------------------------------------------

## CI用：全チェック
ci: lint test
	@echo "All CI checks passed!"

## CI用：クイックチェック（ビルド確認 + lint）
ci-quick: build-check lint
	@echo "Quick CI checks passed!"

#------------------------------------------------------------------------------
# ヘルプ
#------------------------------------------------------------------------------

## ヘルプ表示
help:
	@echo "OBS Optimizer - 開発コマンド"
	@echo ""
	@echo "使用方法: make [target]"
	@echo ""
	@echo "テスト:"
	@echo "  test            全テスト実行（Rust + Frontend）"
	@echo "  test-rust       Rustユニットテスト"
	@echo "  test-integration Rust統合テスト"
	@echo "  test-front      フロントエンドテスト"
	@echo "  test-coverage   フロントエンドカバレッジ"
	@echo ""
	@echo "静的解析:"
	@echo "  lint            全静的解析（Clippy + ESLint）"
	@echo "  lint-rust       Rust静的解析（Clippy）"
	@echo "  lint-front      フロントエンド静的解析"
	@echo ""
	@echo "ビルド:"
	@echo "  build           プロダクションビルド"
	@echo "  build-check     ビルド確認（高速）"
	@echo ""
	@echo "フォーマット:"
	@echo "  fmt             コード整形"
	@echo "  fmt-check       フォーマットチェック"
	@echo ""
	@echo "ユーティリティ:"
	@echo "  shell           開発用シェル"
	@echo "  clean           キャッシュクリア"
	@echo "  rebuild         Dockerイメージ再ビルド"
	@echo ""
	@echo "CI:"
	@echo "  ci              全CIチェック"
	@echo "  ci-quick        クイックチェック"
