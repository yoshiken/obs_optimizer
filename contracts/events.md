# Tauri Events Contract

> このファイルはTauriのイベント（Backend→Frontend）を定義する。
> リアルタイム更新にはイベントを使用し、一度きりの取得にはコマンドを使用する。

---

## Metrics Events

### metrics_update

リアルタイムメトリクス更新（1秒間隔）

```rust
// Backend emission
app_handle.emit("metrics_update", MetricsSnapshot { ... })?;
```

```typescript
// Frontend subscription
import { listen } from '@tauri-apps/api/event';

interface MetricsUpdatePayload {
  timestamp: number;
  obs: ObsMetrics | null;
  system: SystemMetrics;
}

const unlisten = await listen<MetricsUpdatePayload>('metrics_update', (event) => {
  console.log('Metrics:', event.payload);
});

// Cleanup
unlisten();
```

---

## OBS Events

### obs_connection_changed

OBS接続状態の変更

```rust
#[derive(Debug, Serialize)]
pub struct ObsConnectionEvent {
    pub connected: bool,
    pub host: Option<String>,
    pub error: Option<String>,
}

app_handle.emit("obs_connection_changed", ObsConnectionEvent { ... })?;
```

```typescript
interface ObsConnectionPayload {
  connected: boolean;
  host?: string;
  error?: string;
}

const unlisten = await listen<ObsConnectionPayload>('obs_connection_changed', (event) => {
  if (event.payload.connected) {
    console.log('Connected to:', event.payload.host);
  } else {
    console.log('Disconnected:', event.payload.error);
  }
});
```

---

### obs_streaming_changed

配信状態の変更

```rust
#[derive(Debug, Serialize)]
pub struct ObsStreamingEvent {
    pub streaming: bool,
    pub recording: bool,
}

app_handle.emit("obs_streaming_changed", ObsStreamingEvent { ... })?;
```

```typescript
interface ObsStreamingPayload {
  streaming: boolean;
  recording: boolean;
}

const unlisten = await listen<ObsStreamingPayload>('obs_streaming_changed', (event) => {
  console.log('Streaming:', event.payload.streaming);
  console.log('Recording:', event.payload.recording);
});
```

---

## Alert Events

### alert_triggered

閾値超過時のアラート

```rust
#[derive(Debug, Serialize)]
pub struct AlertEvent {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
}

#[derive(Debug, Serialize)]
pub enum AlertType {
    CpuHigh,
    GpuHigh,
    FrameDrop,
    NetworkUnstable,
}

#[derive(Debug, Serialize)]
pub enum AlertSeverity {
    Warning,
    Critical,
}

app_handle.emit("alert_triggered", AlertEvent { ... })?;
```

```typescript
type AlertType = 'cpu_high' | 'gpu_high' | 'frame_drop' | 'network_unstable';
type AlertSeverity = 'warning' | 'critical';

interface AlertPayload {
  alertType: AlertType;
  severity: AlertSeverity;
  message: string;
  value: number;
  threshold: number;
}

const unlisten = await listen<AlertPayload>('alert_triggered', (event) => {
  const { alertType, severity, message } = event.payload;
  // Show notification to user
});
```

---

## Event Subscription Hook Example

```typescript
// src/hooks/useTauriEvents.ts

import { useEffect } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export function useTauriEvent<T>(
  eventName: string,
  handler: (payload: T) => void
) {
  useEffect(() => {
    let unlisten: UnlistenFn;

    listen<T>(eventName, (event) => {
      handler(event.payload);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [eventName, handler]);
}

// Usage
function MetricsPanel() {
  const [metrics, setMetrics] = useState<MetricsSnapshot | null>(null);

  useTauriEvent<MetricsSnapshot>('metrics_update', setMetrics);

  return <div>{metrics?.system.cpu.usagePercent}%</div>;
}
```
