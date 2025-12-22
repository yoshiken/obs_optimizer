# Shared Types Contract

> このファイルはRust/TypeScript間で共有される型定義を管理する。
> 型の追加・変更時は必ずここに先に記述し、両方の実装を同期すること。

---

## Error Types

### AppError

```rust
// src-tauri/src/error.rs

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn obs_connection(message: impl Into<String>) -> Self {
        Self::new("OBS_CONNECTION_ERROR", message)
    }

    pub fn obs_timeout(message: impl Into<String>) -> Self {
        Self::new("OBS_TIMEOUT", message)
    }

    pub fn system_metrics(message: impl Into<String>) -> Self {
        Self::new("SYSTEM_METRICS_ERROR", message)
    }
}
```

```typescript
// src/types/error.ts

export interface AppError {
  code: string;
  message: string;
  details?: string;
}

export const ErrorCodes = {
  OBS_CONNECTION_ERROR: 'OBS_CONNECTION_ERROR',
  OBS_TIMEOUT: 'OBS_TIMEOUT',
  SYSTEM_METRICS_ERROR: 'SYSTEM_METRICS_ERROR',
} as const;
```

---

## OBS Types

### ObsSettings

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ObsSettings {
    pub video: VideoSettings,
    pub audio: AudioSettings,
    pub output: OutputSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoSettings {
    pub base_width: u32,
    pub base_height: u32,
    pub output_width: u32,
    pub output_height: u32,
    pub fps_numerator: u32,
    pub fps_denominator: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioSettings {
    pub sample_rate: u32,
    pub channels: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputSettings {
    pub encoder: String,
    pub video_bitrate: u32,
    pub audio_bitrate: u32,
    pub keyframe_interval: u32,
}
```

```typescript
// src/types/obs.ts

export interface ObsSettings {
  video: VideoSettings;
  audio: AudioSettings;
  output: OutputSettings;
}

export interface VideoSettings {
  baseWidth: number;
  baseHeight: number;
  outputWidth: number;
  outputHeight: number;
  fpsNumerator: number;
  fpsDenominator: number;
}

export interface AudioSettings {
  sampleRate: number;
  channels: number;
}

export interface OutputSettings {
  encoder: string;
  videoBitrate: number;
  audioBitrate: number;
  keyframeInterval: number;
}
```

**Note**: Rust uses `snake_case`, TypeScript uses `camelCase`. Serde handles conversion.

---

## Platform Types

### StreamingPlatform

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum StreamingPlatform {
    YouTube,
    Twitch,
    Niconico,
    Custom(String),
}
```

```typescript
export type StreamingPlatform = 'youtube' | 'twitch' | 'niconico' | { custom: string };
```

---

## Metrics Types

### MetricsSnapshot

```rust
#[derive(Debug, Serialize)]
pub struct MetricsSnapshot {
    pub timestamp: i64,
    pub obs: Option<ObsMetrics>,
    pub system: SystemMetrics,
}
```

```typescript
export interface MetricsSnapshot {
  timestamp: number;
  obs: ObsMetrics | null;
  system: SystemMetrics;
}
```

---

### GpuMetrics

GPU使用状況のメトリクス（NVIDIA GPU用）

```rust
// src-tauri/src/monitor/gpu.rs

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GpuMetrics {
    /// GPU名称
    pub name: String,
    /// GPUインデックス（マルチGPU環境用）
    pub index: u32,
    /// GPU使用率（0-100%）
    pub usage_percent: f32,
    /// 使用中のVRAM（バイト）
    pub memory_used_bytes: u64,
    /// 総VRAM容量（バイト）
    pub memory_total_bytes: u64,
    /// GPU温度（摂氏）
    pub temperature: Option<u32>,
    /// エンコーダー使用率（0-100%）
    /// OBS配信時のNVENC負荷
    pub encoder_usage: Option<f32>,
}
```

```typescript
// src/types/monitor.ts

export interface GpuMetrics {
  name: string;
  index: number;
  usagePercent: number;
  memoryUsedBytes: number;
  memoryTotalBytes: number;
  temperature: number | null;
  encoderUsage: number | null;
}
```

**Notes**:
- `temperature` と `encoder_usage` は `Option<T>` - GPUがサポートしていない場合は `None`/`null`
- NVIDIA GPUのみサポート（nvml-wrapperを使用）
- NVMLドライバがインストールされていない環境では `get_gpu_metrics()` は `Ok(None)` を返す
- マルチGPU環境では `get_all_gpu_metrics()` で全GPUを取得可能
