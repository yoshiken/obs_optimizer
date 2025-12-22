// Tauriコマンドの型定義 - バックエンドと同期必須
// contracts/api.md に準拠

// ========================================
// OBS関連の型
// ========================================

export interface ObsConnectionParams {
  host: string;
  port: number;
  password?: string;
}

export interface ObsStatus {
  connected: boolean;
  streaming: boolean;
  recording: boolean;
  virtualCamActive: boolean;
  currentScene: string | null;
  obsVersion: string | null;
  websocketVersion: string | null;
  streamTimecode: number | null;
  recordTimecode: number | null;
  streamBitrate: number | null;
  recordBitrate: number | null;
  fps: number | null;
  renderDroppedFrames: number | null;
  outputDroppedFrames: number | null;
}

export type ConnectionState =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | 'reconnecting'
  | 'error';

// OBSイベントペイロード
export interface ConnectionChangedPayload {
  previousState: ConnectionState;
  currentState: ConnectionState;
  host: string | null;
  port: number | null;
}

export interface StreamingChangedPayload {
  isStreaming: boolean;
  startedAt: number | null;
}

export interface RecordingChangedPayload {
  isRecording: boolean;
  startedAt: number | null;
}

export interface SceneChangedPayload {
  previousScene: string | null;
  currentScene: string;
}

export interface ObsErrorPayload {
  code: string;
  message: string;
  recoverable: boolean;
}

// OBSイベント名
export const OBS_EVENTS = {
  CONNECTION_CHANGED: 'obs:connection-changed',
  STREAMING_CHANGED: 'obs:streaming-changed',
  RECORDING_CHANGED: 'obs:recording-changed',
  STATUS_UPDATE: 'obs:status-update',
  SCENE_CHANGED: 'obs:scene-changed',
  ERROR: 'obs:error',
} as const;

// ========================================
// システムメトリクス関連の型（契約準拠）
// ========================================

/** CPU使用状況のメトリクス */
export interface CpuMetrics {
  /** 平均CPU使用率（0-100%） */
  usagePercent: number;
  /** CPUコア数 */
  coreCount: number;
  /** 各コアの使用率 */
  perCoreUsage: number[];
}

/** メモリ使用状況のメトリクス */
export interface MemoryMetrics {
  /** 総メモリ容量（バイト） */
  totalBytes: number;
  /** 使用中のメモリ（バイト） */
  usedBytes: number;
  /** 利用可能なメモリ（バイト） */
  availableBytes: number;
  /** メモリ使用率（0-100%） */
  usagePercent: number;
}

/** GPU使用状況のメトリクス */
export interface GpuMetrics {
  /** GPU名称 */
  name: string;
  /** GPU使用率（0-100%） */
  usagePercent: number;
  /** 使用中のVRAM（バイト） */
  memoryUsedBytes: number;
  /** 総VRAM容量（バイト） */
  memoryTotalBytes: number;
  /** エンコーダー使用率（0-100%） */
  encoderUsage: number;
}

/** ネットワーク使用状況のメトリクス */
export interface NetworkMetrics {
  /** アップロード速度（バイト/秒） */
  uploadBytesPerSec: number;
  /** ダウンロード速度（バイト/秒） */
  downloadBytesPerSec: number;
}

/** システム全体のメトリクス（契約準拠） */
export interface SystemMetrics {
  /** CPU情報 */
  cpu: CpuMetrics;
  /** メモリ情報 */
  memory: MemoryMetrics;
  /** GPU情報（取得できない場合はnull） */
  gpu: GpuMetrics | null;
  /** ネットワーク情報 */
  network: NetworkMetrics;
}

// ========================================
// プロセスメトリクス関連の型
// ========================================

/** プロセスのリソース使用状況 */
export interface ProcessMetrics {
  /** プロセス名 */
  name: string;
  /** プロセスID */
  pid: number;
  /** CPU使用率 */
  cpuUsage: number;
  /** メモリ使用量（バイト） */
  memoryBytes: number;
  /** プロセスが存在するかどうか */
  isRunning: boolean;
}

/** OBSプロセス固有のメトリクス */
export interface ObsProcessMetrics {
  /** OBSメインプロセスのメトリクス */
  mainProcess: ProcessMetrics | null;
  /** 合計CPU使用率 */
  totalCpuUsage: number;
  /** 合計メモリ使用量（バイト） */
  totalMemoryBytes: number;
}

// ========================================
// レガシー型（後方互換性用）
// ========================================

/** レガシー形式のシステムメトリクス */
export interface LegacySystemMetrics {
  cpuUsage: number;
  memoryUsed: number;
  memoryTotal: number;
}

// ========================================
// コマンド名と戻り値の型マッピング
// ========================================

export interface Commands {
  // システムメトリクス
  get_system_metrics: () => Promise<SystemMetrics>;
  get_process_metrics: () => Promise<ObsProcessMetrics>;
  get_legacy_system_metrics: () => Promise<LegacySystemMetrics>;

  // OBS接続
  connect_obs: (params: ObsConnectionParams) => Promise<void>;
  disconnect_obs: () => Promise<void>;
  get_obs_status: () => Promise<ObsStatus>;

  // OBSシーン操作
  get_scene_list: () => Promise<string[]>;
  set_current_scene: (sceneName: string) => Promise<void>;

  // OBS配信・録画
  start_streaming: () => Promise<void>;
  stop_streaming: () => Promise<void>;
  start_recording: () => Promise<void>;
  stop_recording: () => Promise<string>;
}
