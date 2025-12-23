// テストユーティリティモジュール
//
// テスト用の共通ヘルパー、フィクスチャ、モックを提供する。
// このモジュールは #[cfg(test)] でのみコンパイルされる。
//
// 設計原則:
// - シンプルに保つ: 過度な抽象化を避ける
// - ビルダーパターン: 複雑なテストデータ構築を支援
// - トレイト分離: モック実装を明確に区別

pub mod fixtures;
pub mod builders;
pub mod assertions;

// 主要な型を再エクスポート
pub use fixtures::*;
pub use builders::*;
pub use assertions::*;
