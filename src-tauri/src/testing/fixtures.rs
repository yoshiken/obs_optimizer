// テストフィクスチャ
//
// テスト用の標準的なデータセットを提供する。
// 各フィクスチャは現実的なデータを模倣し、テストの意図を明確にする。

use crate::obs::settings::{
    AudioSettings, ObsSettings, OutputSettings, VideoSettings,
};
use crate::obs::types::ObsStatus;
use crate::services::optimizer::HardwareInfo;
pub use crate::storage::metrics_history::SystemMetricsSnapshot;

// =============================================================================
// システムメトリクス フィクスチャ
// =============================================================================

/// 正常なシステム状態（低負荷）
pub fn healthy_system_metrics() -> SystemMetricsSnapshot {
    SystemMetricsSnapshot {
        cpu_usage: 35.0,
        memory_used: 8_000_000_000,      // 8GB
        memory_total: 32_000_000_000,     // 32GB
        gpu_usage: Some(40.0),
        gpu_memory_used: Some(4_000_000_000), // 4GB
        network_upload: 1_000_000,        // 1MB/s
        network_download: 500_000,        // 500KB/s
    }
}

/// 高負荷状態（配信中の典型的な状態）
pub fn high_load_system_metrics() -> SystemMetricsSnapshot {
    SystemMetricsSnapshot {
        cpu_usage: 85.0,
        memory_used: 28_000_000_000,      // 28GB
        memory_total: 32_000_000_000,     // 32GB
        gpu_usage: Some(92.0),
        gpu_memory_used: Some(10_000_000_000), // 10GB
        network_upload: 800_000,
        network_download: 200_000,
    }
}

/// クリティカル状態（リソース枯渇寸前）
pub fn critical_system_metrics() -> SystemMetricsSnapshot {
    SystemMetricsSnapshot {
        cpu_usage: 98.0,
        memory_used: 31_500_000_000,      // 31.5GB
        memory_total: 32_000_000_000,     // 32GB
        gpu_usage: Some(99.0),
        gpu_memory_used: Some(11_500_000_000), // 11.5GB
        network_upload: 100_000,          // 帯域制限状態
        network_download: 50_000,
    }
}

/// GPU非搭載システム
pub fn no_gpu_system_metrics() -> SystemMetricsSnapshot {
    SystemMetricsSnapshot {
        cpu_usage: 50.0,
        memory_used: 8_000_000_000,
        memory_total: 16_000_000_000,
        gpu_usage: None,
        gpu_memory_used: None,
        network_upload: 500_000,
        network_download: 250_000,
    }
}

// =============================================================================
// ハードウェア情報 フィクスチャ
// =============================================================================

/// ハイエンドPC（NVIDIA GPU搭載）
pub fn high_end_hardware() -> HardwareInfo {
    use crate::monitor::gpu::GpuInfo;

    HardwareInfo {
        cpu_name: "AMD Ryzen 9 7950X".to_string(),
        cpu_cores: 16,
        total_memory_gb: 64.0,
        gpu: Some(GpuInfo {
            name: "NVIDIA GeForce RTX 4090".to_string(),
        }),
    }
}

/// ミドルレンジPC（NVIDIA GPU搭載）
pub fn mid_range_hardware() -> HardwareInfo {
    use crate::monitor::gpu::GpuInfo;

    HardwareInfo {
        cpu_name: "Intel Core i7-12700".to_string(),
        cpu_cores: 8,
        total_memory_gb: 32.0,
        gpu: Some(GpuInfo {
            name: "NVIDIA GeForce RTX 3060".to_string(),
        }),
    }
}

/// ローエンドPC（GPU非搭載）
pub fn low_end_hardware() -> HardwareInfo {
    HardwareInfo {
        cpu_name: "Intel Core i3-10100".to_string(),
        cpu_cores: 4,
        total_memory_gb: 8.0,
        gpu: None,
    }
}

// =============================================================================
// OBS設定 フィクスチャ
// =============================================================================

/// 標準的な1080p60fps設定
pub fn standard_obs_settings() -> ObsSettings {
    ObsSettings {
        video: VideoSettings {
            base_width: 1920,
            base_height: 1080,
            output_width: 1920,
            output_height: 1080,
            fps_numerator: 60,
            fps_denominator: 1,
        },
        audio: AudioSettings {
            sample_rate: 48000,
            channels: 2,
        },
        output: OutputSettings {
            encoder: "ffmpeg_nvenc".to_string(),
            bitrate_kbps: 6000,
            keyframe_interval_secs: 2,
            preset: Some("p5".to_string()),
            rate_control: Some("CBR".to_string()),
        },
    }
}

/// 720p30fps設定（低スペック向け）
pub fn low_spec_obs_settings() -> ObsSettings {
    ObsSettings {
        video: VideoSettings {
            base_width: 1920,
            base_height: 1080,
            output_width: 1280,
            output_height: 720,
            fps_numerator: 30,
            fps_denominator: 1,
        },
        audio: AudioSettings {
            sample_rate: 44100,
            channels: 2,
        },
        output: OutputSettings {
            encoder: "obs_x264".to_string(),
            bitrate_kbps: 3000,
            keyframe_interval_secs: 2,
            preset: Some("veryfast".to_string()),
            rate_control: Some("CBR".to_string()),
        },
    }
}

/// 4K60fps設定（ハイエンド向け）
pub fn high_end_obs_settings() -> ObsSettings {
    ObsSettings {
        video: VideoSettings {
            base_width: 3840,
            base_height: 2160,
            output_width: 3840,
            output_height: 2160,
            fps_numerator: 60,
            fps_denominator: 1,
        },
        audio: AudioSettings {
            sample_rate: 48000,
            channels: 2,
        },
        output: OutputSettings {
            encoder: "ffmpeg_nvenc".to_string(),
            bitrate_kbps: 20000,
            keyframe_interval_secs: 2,
            preset: Some("p6".to_string()),
            rate_control: Some("CBR".to_string()),
        },
    }
}

// =============================================================================
// OBSステータス フィクスチャ
// =============================================================================

/// 配信中のステータス
pub fn streaming_obs_status() -> ObsStatus {
    ObsStatus {
        connected: true,
        streaming: true,
        recording: false,
        virtual_cam_active: false,
        current_scene: Some("Main Scene".to_string()),
        obs_version: Some("30.0.0".to_string()),
        websocket_version: Some("5.0.0".to_string()),
        stream_timecode: Some(3600),      // 1時間
        record_timecode: None,
        stream_bitrate: Some(6000),
        record_bitrate: None,
        fps: Some(60.0),
        render_dropped_frames: Some(5),
        output_dropped_frames: Some(2),
    }
}

/// 録画中のステータス
pub fn recording_obs_status() -> ObsStatus {
    ObsStatus {
        connected: true,
        streaming: false,
        recording: true,
        virtual_cam_active: false,
        current_scene: Some("Recording Scene".to_string()),
        obs_version: Some("30.0.0".to_string()),
        websocket_version: Some("5.0.0".to_string()),
        stream_timecode: None,
        record_timecode: Some(1800),      // 30分
        stream_bitrate: None,
        record_bitrate: Some(20000),
        fps: Some(60.0),
        render_dropped_frames: Some(0),
        output_dropped_frames: Some(0),
    }
}

/// アイドル状態（接続済み、配信・録画なし）
pub fn idle_obs_status() -> ObsStatus {
    ObsStatus {
        connected: true,
        streaming: false,
        recording: false,
        virtual_cam_active: false,
        current_scene: Some("Default Scene".to_string()),
        obs_version: Some("30.0.0".to_string()),
        websocket_version: Some("5.0.0".to_string()),
        stream_timecode: None,
        record_timecode: None,
        stream_bitrate: None,
        record_bitrate: None,
        fps: Some(60.0),
        render_dropped_frames: None,
        output_dropped_frames: None,
    }
}

// =============================================================================
// ビットレート履歴 フィクスチャ
// =============================================================================

/// 安定したビットレート履歴
pub fn stable_bitrate_history() -> Vec<u64> {
    vec![6000, 6010, 5990, 6005, 5995, 6000, 6008, 5992, 6000, 6000]
}

/// 不安定なビットレート履歴
pub fn unstable_bitrate_history() -> Vec<u64> {
    vec![6000, 4500, 6200, 3500, 5800, 4000, 6500, 3000, 5500, 4200]
}

/// 帯域不足のビットレート履歴
pub fn insufficient_bitrate_history() -> Vec<u64> {
    vec![4000, 3800, 3500, 4200, 3600, 3900, 3700, 4100, 3400, 3800]
}

// =============================================================================
// テスト用ユーティリティ
// =============================================================================

/// メトリクス履歴を生成（指定した数だけ）
pub fn generate_metrics_history(count: usize, base: SystemMetricsSnapshot) -> Vec<SystemMetricsSnapshot> {
    (0..count)
        .map(|i| {
            let variance = (i as f32 % 10.0) - 5.0;
            SystemMetricsSnapshot {
                cpu_usage: (base.cpu_usage + variance).clamp(0.0, 100.0),
                gpu_usage: base.gpu_usage.map(|g| (g + variance).clamp(0.0, 100.0)),
                ..base.clone()
            }
        })
        .collect()
}
