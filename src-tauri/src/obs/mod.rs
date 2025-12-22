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
pub use error::{error_codes, ObsResult};
pub use events::{
    current_timestamp,
    event_names,
    ConnectionChangedPayload,
    ErrorPayload,
    ObsEventEmitter,
    RecordingChangedPayload,
    SceneChangedPayload,
    StreamingChangedPayload,
};
pub use reconnect::{ReconnectHandle, ReconnectManager, ReconnectTaskState};
pub use state::get_obs_client;
pub use types::{
    ConnectionConfig,
    ConnectionState,
    ObsStatus,
    ReconnectConfig,
    SceneInfo,
    SourceInfo,
};
