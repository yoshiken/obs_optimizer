// OBS WebSocket連携モジュール
//
// OBS Studioとの通信を担当するモジュール群
// obwsクレートを使用してOBS WebSocket 5.x プロトコルに対応

pub mod client;
pub mod error;
pub mod events;
pub mod reconnect;
pub mod state;
pub mod types;
pub mod settings;

// 主要な型の再エクスポート
pub use client::ObsClient;
pub use events::{
    ConnectionChangedPayload,
    ObsEventEmitter,
    RecordingChangedPayload,
    StreamingChangedPayload,
};
pub use state::get_obs_client;
pub use types::{
    ConnectionConfig,
    ConnectionState,
    ObsStatus,
};
// 設定関連の型をエクスポート（公開API用）
// 将来のAPI拡張のために定義を維持
#[allow(unused_imports)]
pub use settings::{
    get_obs_settings,
    ObsSettings,
    VideoSettings,
    AudioSettings,
    OutputSettings,
    EncoderType,
};
