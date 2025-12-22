// サービスレイヤーモジュール
//
// ビジネスロジックを集約し、Tauriコマンドとドメイン層を仲介する。
//
// 責務:
// - ドメインロジックの統一的なAPIを提供
// - エラーハンドリングとバリデーションの一元化
// - 将来的なロギング、メトリクス、キャッシングのフックポイント
//
// アーキテクチャ:
// ```
// Tauriコマンド (commands/)
//       ↓
// サービス層 (services/)  ← このモジュール
//       ↓
// ドメイン層 (obs/, monitor/)
// ```

pub mod obs;
pub mod system;

// 公開エクスポート
pub use obs::obs_service;
pub use system::system_monitor_service;
