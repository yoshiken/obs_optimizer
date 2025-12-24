// テストデータビルダー
//
// ビルダーパターンを使用して、柔軟なテストデータ構築を支援する。
// 各ビルダーはデフォルト値を持ち、必要なフィールドのみをカスタマイズ可能。

use crate::obs::settings::{
    AudioSettings, ObsSettings, OutputSettings, VideoSettings,
};
use crate::obs::types::{ConnectionConfig, ObsStatus};
use crate::services::optimizer::HardwareInfo;
use crate::storage::metrics_history::SystemMetricsSnapshot;

// =============================================================================
// SystemMetricsSnapshot ビルダー
// =============================================================================

/// システムメトリクスのビルダー
#[derive(Debug, Clone)]
pub struct SystemMetricsBuilder {
    cpu_usage: f32,
    memory_used: u64,
    memory_total: u64,
    gpu_usage: Option<f32>,
    gpu_memory_used: Option<u64>,
    network_upload: u64,
    network_download: u64,
}

impl Default for SystemMetricsBuilder {
    fn default() -> Self {
        Self {
            cpu_usage: 50.0,
            memory_used: 16_000_000_000,
            memory_total: 32_000_000_000,
            gpu_usage: Some(50.0),
            gpu_memory_used: Some(4_000_000_000),
            network_upload: 1_000_000,
            network_download: 500_000,
        }
    }
}

impl SystemMetricsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cpu_usage(mut self, usage: f32) -> Self {
        self.cpu_usage = usage;
        self
    }

    pub fn memory(mut self, used: u64, total: u64) -> Self {
        self.memory_used = used;
        self.memory_total = total;
        self
    }

    pub fn memory_percent(mut self, percent: f32) -> Self {
        self.memory_used = (self.memory_total as f32 * percent / 100.0) as u64;
        self
    }

    pub fn gpu_usage(mut self, usage: Option<f32>) -> Self {
        self.gpu_usage = usage;
        self
    }

    pub fn gpu_memory(mut self, used: Option<u64>) -> Self {
        self.gpu_memory_used = used;
        self
    }

    pub fn no_gpu(mut self) -> Self {
        self.gpu_usage = None;
        self.gpu_memory_used = None;
        self
    }

    pub fn network(mut self, upload: u64, download: u64) -> Self {
        self.network_upload = upload;
        self.network_download = download;
        self
    }

    pub fn build(self) -> SystemMetricsSnapshot {
        SystemMetricsSnapshot {
            cpu_usage: self.cpu_usage,
            memory_used: self.memory_used,
            memory_total: self.memory_total,
            gpu_usage: self.gpu_usage,
            gpu_memory_used: self.gpu_memory_used,
            network_upload: self.network_upload,
            network_download: self.network_download,
        }
    }
}

// =============================================================================
// HardwareInfo ビルダー
// =============================================================================

/// ハードウェア情報のビルダー
#[derive(Debug, Clone)]
pub struct HardwareInfoBuilder {
    cpu_name: String,
    cpu_cores: usize,
    total_memory_gb: f64,
    gpu_name: Option<String>,
}

impl Default for HardwareInfoBuilder {
    fn default() -> Self {
        Self {
            cpu_name: "Test CPU".to_string(),
            cpu_cores: 8,
            total_memory_gb: 16.0,
            gpu_name: Some("NVIDIA GeForce RTX 3060".to_string()),
        }
    }
}

impl HardwareInfoBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cpu(mut self, name: &str, cores: usize) -> Self {
        self.cpu_name = name.to_string();
        self.cpu_cores = cores;
        self
    }

    pub fn cores(mut self, cores: usize) -> Self {
        self.cpu_cores = cores;
        self
    }

    pub fn memory_gb(mut self, gb: f64) -> Self {
        self.total_memory_gb = gb;
        self
    }

    pub fn gpu(mut self, name: &str) -> Self {
        self.gpu_name = Some(name.to_string());
        self
    }

    pub fn nvidia_gpu(mut self) -> Self {
        self.gpu_name = Some("NVIDIA GeForce RTX 3060".to_string());
        self
    }

    pub fn amd_gpu(mut self) -> Self {
        self.gpu_name = Some("AMD Radeon RX 6800 XT".to_string());
        self
    }

    pub fn intel_gpu(mut self) -> Self {
        self.gpu_name = Some("Intel Arc A770".to_string());
        self
    }

    pub fn no_gpu(mut self) -> Self {
        self.gpu_name = None;
        self
    }

    pub fn build(self) -> HardwareInfo {
        use crate::monitor::gpu::GpuInfo;

        let gpu = self.gpu_name.map(|name| GpuInfo { name });

        HardwareInfo {
            cpu_name: self.cpu_name,
            cpu_cores: self.cpu_cores,
            total_memory_gb: self.total_memory_gb,
            gpu,
        }
    }
}

// =============================================================================
// ObsSettings ビルダー
// =============================================================================

/// OBS設定のビルダー
#[derive(Debug, Clone)]
pub struct ObsSettingsBuilder {
    base_width: u32,
    base_height: u32,
    output_width: u32,
    output_height: u32,
    fps: u32,
    sample_rate: u32,
    channels: u32,
    encoder: String,
    bitrate_kbps: u32,
    keyframe_interval_secs: u32,
    preset: Option<String>,
    rate_control: Option<String>,
}

impl Default for ObsSettingsBuilder {
    fn default() -> Self {
        Self {
            base_width: 1920,
            base_height: 1080,
            output_width: 1920,
            output_height: 1080,
            fps: 60,
            sample_rate: 48000,
            channels: 2,
            encoder: "ffmpeg_nvenc".to_string(),
            bitrate_kbps: 6000,
            keyframe_interval_secs: 2,
            preset: Some("p5".to_string()),
            rate_control: Some("CBR".to_string()),
        }
    }
}

impl ObsSettingsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolution(mut self, width: u32, height: u32) -> Self {
        self.base_width = width;
        self.base_height = height;
        self.output_width = width;
        self.output_height = height;
        self
    }

    pub fn output_resolution(mut self, width: u32, height: u32) -> Self {
        self.output_width = width;
        self.output_height = height;
        self
    }

    pub fn fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }

    pub fn preset_1080p60(self) -> Self {
        self.resolution(1920, 1080).fps(60)
    }

    pub fn preset_720p30(self) -> Self {
        self.resolution(1920, 1080)
            .output_resolution(1280, 720)
            .fps(30)
            .bitrate(4500)  // 720p30に適したビットレート
    }

    pub fn preset_4k60(self) -> Self {
        self.resolution(3840, 2160)
            .fps(60)
            .bitrate(20000)  // 4K60に適したビットレート
    }

    pub fn encoder(mut self, encoder: &str) -> Self {
        self.encoder = encoder.to_string();
        self
    }

    pub fn nvenc(mut self) -> Self {
        self.encoder = "ffmpeg_nvenc".to_string();
        self.preset = Some("p5".to_string());
        self
    }

    pub fn x264(mut self) -> Self {
        self.encoder = "obs_x264".to_string();
        self.preset = Some("veryfast".to_string());
        self
    }

    pub fn bitrate(mut self, kbps: u32) -> Self {
        self.bitrate_kbps = kbps;
        self
    }

    pub fn preset(mut self, preset: &str) -> Self {
        self.preset = Some(preset.to_string());
        self
    }

    pub fn rate_control(mut self, mode: &str) -> Self {
        self.rate_control = Some(mode.to_string());
        self
    }

    pub fn audio(mut self, sample_rate: u32, channels: u32) -> Self {
        self.sample_rate = sample_rate;
        self.channels = channels;
        self
    }

    pub fn build(self) -> ObsSettings {
        ObsSettings {
            video: VideoSettings {
                base_width: self.base_width,
                base_height: self.base_height,
                output_width: self.output_width,
                output_height: self.output_height,
                fps_numerator: self.fps,
                fps_denominator: 1,
            },
            audio: AudioSettings {
                sample_rate: self.sample_rate,
                channels: self.channels,
            },
            output: OutputSettings {
                encoder: self.encoder,
                bitrate_kbps: self.bitrate_kbps,
                keyframe_interval_secs: self.keyframe_interval_secs,
                preset: self.preset,
                rate_control: self.rate_control,
            },
        }
    }
}

// =============================================================================
// ObsStatus ビルダー
// =============================================================================

/// OBSステータスのビルダー
#[derive(Debug, Clone, Default)]
pub struct ObsStatusBuilder {
    connected: bool,
    streaming: bool,
    recording: bool,
    virtual_cam_active: bool,
    current_scene: Option<String>,
    obs_version: Option<String>,
    websocket_version: Option<String>,
    stream_timecode: Option<u64>,
    record_timecode: Option<u64>,
    stream_bitrate: Option<u32>,
    record_bitrate: Option<u32>,
    fps: Option<f64>,
    render_dropped_frames: Option<u32>,
    output_dropped_frames: Option<u32>,
}

impl ObsStatusBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connected(mut self) -> Self {
        self.connected = true;
        self.obs_version = Some("30.0.0".to_string());
        self.websocket_version = Some("5.0.0".to_string());
        self.current_scene = Some("Default Scene".to_string());
        self.fps = Some(60.0);
        self
    }

    pub fn disconnected(mut self) -> Self {
        self.connected = false;
        self
    }

    pub fn streaming(mut self) -> Self {
        self.connected = true;
        self.streaming = true;
        self
    }

    pub fn recording(mut self) -> Self {
        self.connected = true;
        self.recording = true;
        self
    }

    pub fn scene(mut self, name: &str) -> Self {
        self.current_scene = Some(name.to_string());
        self
    }

    pub fn stream_time(mut self, seconds: u64) -> Self {
        self.stream_timecode = Some(seconds);
        self
    }

    pub fn record_time(mut self, seconds: u64) -> Self {
        self.record_timecode = Some(seconds);
        self
    }

    pub fn stream_bitrate(mut self, kbps: u32) -> Self {
        self.stream_bitrate = Some(kbps);
        self
    }

    pub fn dropped_frames(mut self, render: u32, output: u32) -> Self {
        self.render_dropped_frames = Some(render);
        self.output_dropped_frames = Some(output);
        self
    }

    pub fn fps(mut self, fps: f64) -> Self {
        self.fps = Some(fps);
        self
    }

    pub fn build(self) -> ObsStatus {
        ObsStatus {
            connected: self.connected,
            streaming: self.streaming,
            recording: self.recording,
            virtual_cam_active: self.virtual_cam_active,
            current_scene: self.current_scene,
            obs_version: self.obs_version,
            websocket_version: self.websocket_version,
            stream_timecode: self.stream_timecode,
            record_timecode: self.record_timecode,
            stream_bitrate: self.stream_bitrate,
            record_bitrate: self.record_bitrate,
            fps: self.fps,
            render_dropped_frames: self.render_dropped_frames,
            output_dropped_frames: self.output_dropped_frames,
        }
    }
}

// =============================================================================
// ConnectionConfig ビルダー
// =============================================================================

/// OBS接続設定のビルダー
#[derive(Debug, Clone)]
pub struct ConnectionConfigBuilder {
    host: String,
    port: u16,
    password: Option<String>,
}

impl Default for ConnectionConfigBuilder {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 4455,
            password: None,
        }
    }
}

impl ConnectionConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }

    pub fn no_password(mut self) -> Self {
        self.password = None;
        self
    }

    /// 無効な設定（空ホスト）
    pub fn invalid_empty_host(mut self) -> Self {
        self.host = String::new();
        self
    }

    /// 無効な設定（低ポート番号）
    pub fn invalid_low_port(mut self) -> Self {
        self.port = 80;
        self
    }

    pub fn build(self) -> ConnectionConfig {
        ConnectionConfig {
            host: self.host,
            port: self.port,
            password: self.password,
        }
    }
}

// =============================================================================
// テスト用ユーティリティ関数
// =============================================================================

/// 複数のメトリクススナップショットを生成
pub fn build_metrics_sequence<F>(count: usize, modifier: F) -> Vec<SystemMetricsSnapshot>
where
    F: Fn(usize, SystemMetricsBuilder) -> SystemMetricsBuilder,
{
    (0..count)
        .map(|i| modifier(i, SystemMetricsBuilder::new()).build())
        .collect()
}

/// CPU使用率が増加するメトリクス履歴を生成
pub fn build_increasing_cpu_metrics(count: usize, start: f32, end: f32) -> Vec<SystemMetricsSnapshot> {
    // count - 1 で割ることで、最初の値が start、最後の値が end になる
    let step = if count > 1 {
        (end - start) / (count - 1) as f32
    } else {
        0.0
    };
    build_metrics_sequence(count, move |i, builder| {
        builder.cpu_usage(start + step * i as f32)
    })
}

/// GPU使用率が増加するメトリクス履歴を生成
pub fn build_increasing_gpu_metrics(count: usize, start: f32, end: f32) -> Vec<SystemMetricsSnapshot> {
    // count - 1 で割ることで、最初の値が start、最後の値が end になる
    let step = if count > 1 {
        (end - start) / (count - 1) as f32
    } else {
        0.0
    };
    build_metrics_sequence(count, move |i, builder| {
        builder.gpu_usage(Some(start + step * i as f32))
    })
}
