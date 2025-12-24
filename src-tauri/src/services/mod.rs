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
pub mod optimizer;
pub mod alerts;
pub mod streaming_mode;
pub mod analyzer;
pub mod exporter;
pub mod gpu_detection;
pub mod encoder_selector;

// 公開エクスポート
// 将来的な拡張や外部クレートからの利用を想定した再エクスポート
#[allow(unused_imports)]
pub use obs::obs_service;
#[allow(unused_imports)]
pub use system::system_monitor_service;
#[allow(unused_imports)]
pub use optimizer::{RecommendationEngine, HardwareInfo, RecommendedSettings};
#[allow(unused_imports)]
pub use alerts::{AlertEngine, Alert, AlertSeverity, MetricType, initialize_alert_engine, get_alert_engine};
#[allow(unused_imports)]
pub use streaming_mode::{StreamingModeService, get_streaming_mode_service};
#[allow(unused_imports)]
pub use analyzer::{ProblemAnalyzer, ProblemReport, ProblemCategory};
#[allow(unused_imports)]
pub use exporter::{ReportExporter, DiagnosticReport, PerformanceEvaluation};
#[allow(unused_imports)]
pub use gpu_detection::{GpuGeneration, CpuTier, detect_gpu_generation, get_encoder_capability, determine_cpu_tier};
#[allow(unused_imports)]
pub use encoder_selector::{RecommendedEncoder, EncoderSelectionContext, EncoderSelector};
