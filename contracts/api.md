# Tauri Commands API Contract

> このファイルはBackend/Frontend間のインターフェース契約を定義する。
> 新規コマンド追加時は必ずここに先に記述すること。

---

## OBS Connection

### connect_obs

```rust
#[tauri::command]
async fn connect_obs(host: String, port: u16, password: Option<String>) -> Result<(), AppError>
```

```typescript
invoke('connect_obs', { host: string, port: number, password?: string }): Promise<void>
```

**Status**: [ ] Rust実装 | [ ] TypeScript型 | [ ] 統合テスト

---

### disconnect_obs

```rust
#[tauri::command]
async fn disconnect_obs() -> Result<(), AppError>
```

```typescript
invoke('disconnect_obs'): Promise<void>
```

**Status**: [ ] Rust実装 | [ ] TypeScript型 | [ ] 統合テスト

---

### get_obs_status

```rust
#[tauri::command]
async fn get_obs_status() -> Result<ObsStatus, AppError>
```

```typescript
interface ObsStatus {
  connected: boolean;
  streaming: boolean;
  recording: boolean;
  cpuUsage: number;
  memoryUsage: number;
  activeFps: number;
  renderSkippedFrames: number;
  outputSkippedFrames: number;
}

invoke<ObsStatus>('get_obs_status'): Promise<ObsStatus>
```

**Status**: [ ] Rust実装 | [ ] TypeScript型 | [ ] 統合テスト

---

## System Metrics

### get_system_metrics

```rust
#[tauri::command]
async fn get_system_metrics() -> Result<SystemMetrics, AppError>
```

```typescript
interface SystemMetrics {
  cpu: CpuMetrics;
  memory: MemoryMetrics;
  gpu: GpuMetrics | null;
  network: NetworkMetrics;
}

interface CpuMetrics {
  usagePercent: number;
  coreCount: number;
  perCoreUsage: number[];
}

interface MemoryMetrics {
  totalBytes: number;
  usedBytes: number;
  availableBytes: number;
  usagePercent: number;
}

interface GpuMetrics {
  name: string;
  index: number;
  usagePercent: number;
  memoryUsedBytes: number;
  memoryTotalBytes: number;
  temperature: number | null;
  encoderUsage: number | null;
}

interface NetworkMetrics {
  uploadBytesPerSec: number;
  downloadBytesPerSec: number;
}

invoke<SystemMetrics>('get_system_metrics'): Promise<SystemMetrics>
```

**Status**: [ ] Rust実装 | [ ] TypeScript型 | [ ] 統合テスト

---

## Optimizer

### analyze_settings

```rust
#[tauri::command]
async fn analyze_settings() -> Result<AnalysisResult, AppError>
```

```typescript
interface AnalysisResult {
  currentSettings: ObsSettings;
  recommendedSettings: ObsSettings;
  issues: Issue[];
  overallScore: number; // 0-100
}

interface Issue {
  severity: 'critical' | 'warning' | 'info';
  category: 'video' | 'audio' | 'network' | 'encoder';
  message: string;
  recommendation: string;
}

invoke<AnalysisResult>('analyze_settings'): Promise<AnalysisResult>
```

**Status**: [ ] Rust実装 | [ ] TypeScript型 | [ ] 統合テスト

---

### apply_optimization

```rust
#[tauri::command]
async fn apply_optimization(preset: OptimizationPreset) -> Result<(), AppError>
```

```typescript
type OptimizationPreset = 'quality' | 'balanced' | 'performance';

invoke('apply_optimization', { preset: OptimizationPreset }): Promise<void>
```

**Status**: [ ] Rust実装 | [ ] TypeScript型 | [ ] 統合テスト

---

## Configuration

### get_config

```rust
#[tauri::command]
async fn get_config() -> Result<AppConfig, AppError>
```

```typescript
invoke<AppConfig>('get_config'): Promise<AppConfig>
```

**Status**: [x] Rust実装 | [x] TypeScript型 | [ ] 統合テスト

---

### save_app_config

```rust
#[tauri::command]
async fn save_app_config(config: AppConfig) -> Result<(), AppError>
```

```typescript
invoke('save_app_config', { config: AppConfig }): Promise<void>
```

**Status**: [x] Rust実装 | [x] TypeScript型 | [ ] 統合テスト

---

## OBS Settings

### get_obs_settings_command

```rust
#[tauri::command]
async fn get_obs_settings_command() -> Result<ObsSettings, AppError>
```

```typescript
interface ObsSettings {
  video: VideoSettings;
  audio: AudioSettings;
  output: OutputSettings;
}

invoke<ObsSettings>('get_obs_settings_command'): Promise<ObsSettings>
```

**Status**: [x] Rust実装 | [x] TypeScript型 | [ ] 統合テスト

---

## Recommendations

### calculate_recommendations

```rust
#[tauri::command]
async fn calculate_recommendations() -> Result<RecommendedSettings, AppError>
```

```typescript
interface RecommendedSettings {
  video: RecommendedVideoSettings;
  audio: RecommendedAudioSettings;
  output: RecommendedOutputSettings;
  reasons: string[];
  overallScore: number;
}

invoke<RecommendedSettings>('calculate_recommendations'): Promise<RecommendedSettings>
```

**Status**: [x] Rust実装 | [x] TypeScript型 | [ ] 統合テスト

---

### calculate_custom_recommendations

```rust
#[tauri::command]
async fn calculate_custom_recommendations(
    platform: StreamingPlatform,
    style: StreamingStyle,
    network_speed_mbps: f64,
) -> Result<RecommendedSettings, AppError>
```

```typescript
invoke<RecommendedSettings>('calculate_custom_recommendations', {
  platform: StreamingPlatform,
  style: StreamingStyle,
  networkSpeedMbps: number
}): Promise<RecommendedSettings>
```

**Status**: [x] Rust実装 | [x] TypeScript型 | [ ] 統合テスト

---

## Alerts

### get_active_alerts

```rust
#[tauri::command]
async fn get_active_alerts() -> Result<Vec<Alert>, AppError>
```

```typescript
interface Alert {
  id: string;
  metric: MetricType;
  currentValue: number;
  threshold: number;
  severity: AlertSeverity;
  message: string;
  timestamp: number;
  active: boolean;
}

invoke<Alert[]>('get_active_alerts'): Promise<Alert[]>
```

**Status**: [x] Rust実装 | [x] TypeScript型 | [ ] 統合テスト

---

### clear_all_alerts

```rust
#[tauri::command]
async fn clear_all_alerts() -> Result<(), AppError>
```

```typescript
invoke('clear_all_alerts'): Promise<void>
```

**Status**: [x] Rust実装 | [x] TypeScript型 | [ ] 統合テスト
