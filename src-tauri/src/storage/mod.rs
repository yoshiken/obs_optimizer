// ストレージレイヤー - 永続化処理を担当
//
// 責務:
// - 設定ファイルの読み書き (JSON)
// - セッション履歴のデータベース操作 (SQLite)
// - アプリケーションデータディレクトリの管理

pub mod config;
pub mod profiles;
pub mod metrics_history;

// 将来的な拡張や外部クレートからの利用を想定した再エクスポート
#[allow(unused_imports)]
pub use config::{AppConfig, load_config, save_config};
#[allow(unused_imports)]
pub use profiles::{
    SettingsProfile, ProfileSettings, ProfileSummary,
    get_profiles, get_profile, save_profile, delete_profile,
};
#[allow(unused_imports)]
pub use metrics_history::{
    MetricsHistoryStore, HistoricalMetrics, SessionSummary,
    SystemMetricsSnapshot, ObsStatusSnapshot,
};
