// OBS接続統合テスト
//
// OBS WebSocket接続関連の機能を統合的にテストする。
// 実際のOBS接続は必要とせず、接続設定とステータスの操作をテストする。

mod common;

use obs_optimizer_app_lib::testing::builders::{ConnectionConfigBuilder, ObsStatusBuilder};
use obs_optimizer_app_lib::testing::fixtures::{
    idle_obs_status, recording_obs_status, streaming_obs_status,
};

// =============================================================================
// 接続設定ビルダーテスト
// =============================================================================

#[test]
fn test_connection_config_builder_default() {
    let config = ConnectionConfigBuilder::new().build();

    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 4455);
    assert!(config.password.is_none());
}

#[test]
fn test_connection_config_builder_custom() {
    let config = ConnectionConfigBuilder::new()
        .host("192.168.1.100")
        .port(4444)
        .password("secret123")
        .build();

    assert_eq!(config.host, "192.168.1.100");
    assert_eq!(config.port, 4444);
    assert_eq!(config.password.as_deref(), Some("secret123"));
}

#[test]
fn test_connection_config_validation_valid() {
    let config = ConnectionConfigBuilder::new().build();
    assert!(config.validate().is_ok());
}

#[test]
fn test_connection_config_validation_empty_host() {
    let config = ConnectionConfigBuilder::new()
        .invalid_empty_host()
        .build();
    assert!(config.validate().is_err());
}

#[test]
fn test_connection_config_validation_low_port() {
    let config = ConnectionConfigBuilder::new()
        .invalid_low_port()
        .build();
    assert!(config.validate().is_err());
}

// =============================================================================
// OBSステータスビルダーテスト
// =============================================================================

#[test]
fn test_obs_status_builder_disconnected() {
    let status = ObsStatusBuilder::new()
        .disconnected()
        .build();

    assert!(!status.connected);
    assert!(!status.streaming);
    assert!(!status.recording);
}

#[test]
fn test_obs_status_builder_connected() {
    let status = ObsStatusBuilder::new()
        .connected()
        .build();

    assert!(status.connected);
    assert!(!status.streaming);
    assert!(!status.recording);
    assert!(status.obs_version.is_some());
    assert!(status.current_scene.is_some());
}

#[test]
fn test_obs_status_builder_streaming() {
    let status = ObsStatusBuilder::new()
        .connected()
        .streaming()
        .scene("Gaming Scene")
        .stream_time(3600)
        .stream_bitrate(6000)
        .build();

    assert!(status.connected);
    assert!(status.streaming);
    assert!(!status.recording);
    assert_eq!(status.current_scene.as_deref(), Some("Gaming Scene"));
    assert_eq!(status.stream_timecode, Some(3600));
    assert_eq!(status.stream_bitrate, Some(6000));
}

#[test]
fn test_obs_status_builder_recording() {
    let status = ObsStatusBuilder::new()
        .connected()
        .recording()
        .scene("Recording Scene")
        .record_time(1800)
        .build();

    assert!(status.connected);
    assert!(!status.streaming);
    assert!(status.recording);
    assert_eq!(status.record_timecode, Some(1800));
}

#[test]
fn test_obs_status_builder_dropped_frames() {
    let status = ObsStatusBuilder::new()
        .connected()
        .streaming()
        .dropped_frames(10, 5)
        .build();

    assert_eq!(status.render_dropped_frames, Some(10));
    assert_eq!(status.output_dropped_frames, Some(5));
}

// =============================================================================
// フィクスチャテスト
// =============================================================================

#[test]
fn test_streaming_obs_status_fixture() {
    let status = streaming_obs_status();

    assert!(status.connected);
    assert!(status.streaming);
    assert!(!status.recording);
    assert!(status.stream_timecode.is_some());
    assert!(status.stream_bitrate.is_some());
    assert!(status.fps.is_some());
}

#[test]
fn test_recording_obs_status_fixture() {
    let status = recording_obs_status();

    assert!(status.connected);
    assert!(!status.streaming);
    assert!(status.recording);
    assert!(status.record_timecode.is_some());
    assert!(status.record_bitrate.is_some());
}

#[test]
fn test_idle_obs_status_fixture() {
    let status = idle_obs_status();

    assert!(status.connected);
    assert!(!status.streaming);
    assert!(!status.recording);
    assert!(status.stream_timecode.is_none());
    assert!(status.record_timecode.is_none());
}

// =============================================================================
// 複合シナリオテスト
// =============================================================================

#[test]
fn test_scenario_streaming_session_lifecycle() {
    // 配信セッションのライフサイクルをシミュレート

    // 1. 接続前
    let disconnected = ObsStatusBuilder::new()
        .disconnected()
        .build();
    assert!(!disconnected.connected);

    // 2. 接続後（アイドル）
    let idle = ObsStatusBuilder::new()
        .connected()
        .scene("Waiting Scene")
        .build();
    assert!(idle.connected);
    assert!(!idle.streaming);

    // 3. 配信開始
    let streaming_start = ObsStatusBuilder::new()
        .connected()
        .streaming()
        .scene("Main Scene")
        .stream_time(0)
        .stream_bitrate(6000)
        .dropped_frames(0, 0)
        .build();
    assert!(streaming_start.streaming);
    assert_eq!(streaming_start.stream_timecode, Some(0));

    // 4. 配信中（1時間後）
    let streaming_ongoing = ObsStatusBuilder::new()
        .connected()
        .streaming()
        .scene("Main Scene")
        .stream_time(3600)
        .stream_bitrate(5800)
        .dropped_frames(15, 8)
        .build();
    assert_eq!(streaming_ongoing.stream_timecode, Some(3600));
    assert!(streaming_ongoing.render_dropped_frames.unwrap_or(0) > 0);

    // 5. 配信終了
    let ended = ObsStatusBuilder::new()
        .connected()
        .scene("Ending Scene")
        .build();
    assert!(!ended.streaming);
    assert!(ended.stream_timecode.is_none());
}

#[test]
fn test_scenario_simultaneous_streaming_and_recording() {
    // 配信と録画を同時に行うシナリオ
    let status = ObsStatusBuilder::new()
        .connected()
        .streaming()
        .recording()
        .scene("Live Scene")
        .stream_time(1800)
        .record_time(1800)
        .stream_bitrate(6000)
        .fps(60.0)
        .build();

    assert!(status.streaming);
    assert!(status.recording);
    assert_eq!(status.stream_timecode, status.record_timecode);
}

#[test]
fn test_scenario_connection_retry() {
    // 接続リトライのシナリオ
    // 各状態遷移を確認

    // 初回接続失敗
    let failed = ObsStatusBuilder::new()
        .disconnected()
        .build();
    assert!(!failed.connected);

    // リトライ後接続成功
    let connected = ObsStatusBuilder::new()
        .connected()
        .build();
    assert!(connected.connected);
}

// =============================================================================
// エッジケーステスト
// =============================================================================

#[test]
fn test_edge_case_very_long_stream() {
    // 非常に長い配信時間
    let status = ObsStatusBuilder::new()
        .connected()
        .streaming()
        .stream_time(86400) // 24時間
        .build();

    assert_eq!(status.stream_timecode, Some(86400));
}

#[test]
fn test_edge_case_zero_bitrate() {
    // ビットレート0（一時的なネットワーク断）
    let status = ObsStatusBuilder::new()
        .connected()
        .streaming()
        .stream_bitrate(0)
        .build();

    assert_eq!(status.stream_bitrate, Some(0));
}

#[test]
fn test_edge_case_high_dropped_frames() {
    // 大量のドロップフレーム
    let status = ObsStatusBuilder::new()
        .connected()
        .streaming()
        .dropped_frames(10000, 5000)
        .build();

    assert_eq!(status.render_dropped_frames, Some(10000));
    assert_eq!(status.output_dropped_frames, Some(5000));
}

#[test]
fn test_edge_case_very_high_fps() {
    // 高FPS設定
    let status = ObsStatusBuilder::new()
        .connected()
        .fps(144.0)
        .build();

    assert_eq!(status.fps, Some(144.0));
}
