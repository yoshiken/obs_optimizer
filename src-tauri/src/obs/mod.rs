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
