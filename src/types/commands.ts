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
// 設定関連の型（Rust AppConfigに完全対応）
// ========================================

/** 配信スタイル（レガシー型） */
export type StreamStyle = 'talk' | 'game' | 'music' | 'art';

/** 配信プラットフォーム（レガシー型） */
export type StreamPlatform = 'youtube' | 'twitch' | 'niconico' | 'other';

/** 最適化プリセット */
export type OptimizationPreset = 'low' | 'medium' | 'high' | 'ultra' | 'custom';

/** OBS接続設定 */
export interface ConnectionConfig {
  /** 最後に接続したホスト */
  lastHost: string;
  /** 最後に接続したポート */
  lastPort: number;
  /** パスワードを保存するか */
  savePassword: boolean;
  /** 起動時に自動接続するか */
  autoConnectOnStartup: boolean;
  /** 接続タイムアウト（秒） */
  connectionTimeoutSecs: number;
}

/** 監視設定 */
export interface MonitoringConfig {
  /** メトリクス更新間隔（ミリ秒） */
  updateIntervalMs: number;
  /** システムメトリクスを収集するか */
  collectSystemMetrics: boolean;
  /** GPUメトリクスを収集するか */
  collectGpuMetrics: boolean;
  /** OBSプロセスメトリクスを収集するか */
  collectProcessMetrics: boolean;
  /** メトリクス履歴を保存するか */
  saveMetricsHistory: boolean;
}

/** アラート設定 */
export interface AlertConfig {
  /** アラートを有効にするか */
  enabled: boolean;
  /** CPU使用率警告閾値（%） */
  cpuWarningThreshold: number;
  /** CPU使用率クリティカル閾値（%） */
  cpuCriticalThreshold: number;
  /** GPU使用率警告閾値（%） */
  gpuWarningThreshold: number;
  /** GPU使用率クリティカル閾値（%） */
  gpuCriticalThreshold: number;
  /** フレームドロップ率警告閾値（%） */
  frameDropWarningThreshold: number;
  /** フレームドロップ率クリティカル閾値（%） */
  frameDropCriticalThreshold: number;
  /** アラート判定に必要な継続時間（秒） */
  alertDurationSecs: number;
  /** アラート音を鳴らすか */
  playSound: boolean;
  /** デスクトップ通知を表示するか */
  showNotification: boolean;
}

/** 表示設定 */
export interface DisplayConfig {
  /** ダークモードを使用するか */
  darkMode: boolean;
  /** メトリクスグラフの履歴表示時間（秒） */
  graphHistoryDurationSecs: number;
  /** コンパクト表示モード */
  compactMode: boolean;
  /** 常に最前面に表示 */
  alwaysOnTop: boolean;
}

/** 配信モード設定 */
export interface StreamingModeConfig {
  /** 配信プラットフォーム */
  platform: StreamingPlatform;
  /** 配信スタイル */
  style: StreamingStyle;
  /** ネットワーク速度（Mbps） */
  networkSpeedMbps: number;
  /** 画質優先モード */
  qualityPriority: boolean;
}

/** アプリケーション設定（Rust AppConfigに対応） */
export interface AppConfig {
  /** 設定ファイルバージョン */
  version: string;
  /** OBS接続設定 */
  connection: ConnectionConfig;
  /** 監視設定 */
  monitoring: MonitoringConfig;
  /** アラート設定 */
  alerts: AlertConfig;
  /** 表示設定 */
  display: DisplayConfig;
  /** 配信モード設定 */
  streamingMode: StreamingModeConfig;
}

/** フロントエンド用簡易設定（オンボーディング等で使用） */
export interface SimpleAppConfig {
  /** OBS接続設定を保存するか */
  saveConnection: boolean;
  /** アプリケーション起動時に自動接続するか */
  autoConnect: boolean;
  /** 配信スタイル */
  streamStyle: StreamStyle | null;
  /** 配信プラットフォーム */
  platform: StreamPlatform | null;
  /** オンボーディング完了フラグ */
  onboardingCompleted: boolean;
  /** ストリーミングモード有効 */
  streamingModeEnabled: boolean;
}

// ========================================
// 診断・最適化関連の型
// ========================================

/** OBS設定項目 */
export interface ObsSetting {
  /** 設定項目名（内部キー） */
  key: string;
  /** ユーザー向け表示名 */
  displayName: string;
  /** 現在の値 */
  currentValue: string | number | boolean;
  /** 推奨値 */
  recommendedValue: string | number | boolean;
  /** 変更が推奨される理由 */
  reason: string;
  /** 重要度（critical=必須、recommended=推奨、optional=任意） */
  priority: 'critical' | 'recommended' | 'optional';
}

/** システム環境情報 */
export interface SystemInfo {
  /** CPUモデル名 */
  cpuModel: string;
  /** GPUモデル名 */
  gpuModel: string | null;
  /** 総メモリ容量（MB） */
  totalMemoryMb: number;
  /** 利用可能メモリ（MB） */
  availableMemoryMb: number;
}

/** 設定分析リクエスト */
export interface AnalyzeSettingsRequest {
  /** 配信プラットフォーム（省略時は設定ファイルから取得） */
  platform?: StreamingPlatform;
  /** 配信スタイル（省略時は設定ファイルから取得） */
  style?: StreamingStyle;
  /** ネットワーク速度（Mbps、省略時は設定ファイルから取得） */
  networkSpeedMbps?: number;
}

/** 診断結果 */
export interface AnalysisResult {
  /** 全体の品質スコア（0-100） */
  qualityScore: number;
  /** 検出された問題の数 */
  issueCount: number;
  /** 推奨される設定変更リスト */
  recommendations: ObsSetting[];
  /** システム環境情報 */
  systemInfo: SystemInfo;
  /** 分析日時 */
  analyzedAt: number;
  /** 初心者向けサマリー */
  summary: AnalysisSummary;
}

/** 分析サマリー（初心者向け） */
export interface AnalysisSummary {
  /** 初心者向けの一言説明 */
  headline: string;
  /** 推奨プリセット（low/medium/high/ultra） */
  recommendedPreset: 'low' | 'medium' | 'high' | 'ultra';
  /** 主要な推奨値（キー項目のみ） */
  keyRecommendations: KeyRecommendation[];
}

/** 主要な推奨項目（初心者向け） */
export interface KeyRecommendation {
  /** 項目ラベル */
  label: string;
  /** 推奨値 */
  value: string;
  /** 初心者向けの簡潔な理由 */
  reasonSimple: string;
}

/** 最適化適用結果 */
export interface OptimizationResult {
  /** 適用された設定の数 */
  appliedCount: number;
  /** 適用に失敗した設定の数 */
  failedCount: number;
  /** エラーメッセージ（失敗時） */
  errors: string[];
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

  // 設定管理
  get_config: () => Promise<AppConfig>;
  save_app_config: (config: AppConfig) => Promise<void>;

  // 診断・最適化
  analyze_settings: (request?: AnalyzeSettingsRequest) => Promise<AnalysisResult>;
  apply_optimization: (params: {
    preset: OptimizationPreset;
    selectedKeys?: string[];
  }) => Promise<OptimizationResult>;

  // Phase 1b: OBS設定取得
  get_obs_settings_command: () => Promise<ObsSettings>;

  // Phase 1b: 推奨設定算出
  calculate_recommendations: () => Promise<RecommendedSettings>;
  calculate_custom_recommendations: (params: {
    platform: StreamingPlatform;
    style: StreamingStyle;
    networkSpeedMbps: number;
  }) => Promise<RecommendedSettings>;

  // Phase 1b: アラート管理
  get_active_alerts: () => Promise<Alert[]>;
  clear_all_alerts: () => Promise<void>;

  // Phase 2a: プロファイル管理
  get_profiles: () => Promise<ProfileSummary[]>;
  get_profile: (profileId: string) => Promise<SettingsProfile>;
  save_profile: (profile: SettingsProfile) => Promise<void>;
  delete_profile: (profileId: string) => Promise<void>;
  apply_profile: (profileId: string) => Promise<void>;
  save_current_settings_as_profile: (params: {
    name: string;
    description: string;
    platform: StreamingPlatform;
    style: StreamingStyle;
  }) => Promise<string>;

  // Phase 2a: ワンクリック適用・バックアップ
  apply_recommended_settings: () => Promise<void>;
  apply_custom_settings: (params: {
    platform: StreamingPlatform;
    style: StreamingStyle;
    networkSpeedMbps: number;
  }) => Promise<void>;
  backup_current_settings: () => Promise<string>;
  restore_backup: (backupId: string) => Promise<void>;
  get_backups: () => Promise<BackupInfo[]>;

  // Phase 2a: 配信中モード
  set_streaming_mode: (enabled: boolean) => Promise<void>;
  get_streaming_mode: () => Promise<boolean>;

  // Phase 2b: 問題分析
  analyze_problems: (params: AnalyzeProblemsRequest) => Promise<AnalyzeProblemsResponse>;
  get_problem_history: (limit: number) => Promise<ProblemReport[]>;

  // Phase 2b: セッション履歴
  get_sessions: () => Promise<SessionSummary[]>;
  get_metrics_range: (params: {
    sessionId: string;
    from: number;
    to: number;
  }) => Promise<HistoricalMetrics[]>;

  // Phase 2b: エクスポート
  export_session_json: (request: ExportSessionRequest) => Promise<ExportJsonResponse>;
  export_session_csv: (request: ExportSessionRequest) => Promise<ExportCsvResponse>;
  generate_diagnostic_report: () => Promise<DiagnosticReport>;
}

// ========================================
// Phase 1b追加型定義
// ========================================

export type StreamingPlatform = 'youTube' | 'twitch' | 'nicoNico' | 'twitCasting' | 'other';
export type StreamingStyle = 'talk' | 'gaming' | 'music' | 'art' | 'other';

// GPU世代の分類
export type GpuGeneration =
  | 'nvidiaPascal'
  | 'nvidiaTuring'
  | 'nvidiaAmpere'
  | 'nvidiaAda'
  | 'amdVcn3'
  | 'amdVcn4'
  | 'intelArc'
  | 'intelQuickSync'
  | 'unknown'
  | 'none';

// CPUティアの分類
export type CpuTier = 'entry' | 'middle' | 'upperMiddle' | 'highEnd';

// GPU世代ごとのエンコーダー能力
export interface GpuEncoderCapability {
  generation: GpuGeneration;
  h264: boolean;
  hevc: boolean;
  av1: boolean;
  bFrames: boolean;
  qualityEquivalent: string;
  recommendedPreset: string;
}

// 推奨エンコーダー情報
export interface RecommendedEncoder {
  encoderId: string;
  displayName: string;
  preset: string;
  rateControl: string;
  bFrames: number | null;
  lookAhead: boolean;
  psychoVisualTuning: boolean;
  /** マルチパスモード（NVENC: "disabled", "quarter_res", "full_res"） */
  multipassMode: string;
  /** チューニング設定（NVENC: "hq", "ll", "ull" / x264: "film", "animation"等） */
  tuning: string | null;
  /** H.264プロファイル（"baseline", "main", "high"） */
  profile: string;
  reason: string;
}

// ユーザー設定（推奨設定算出用）
export interface UserPreferences {
  platform: StreamingPlatform;
  style: StreamingStyle;
  networkUploadMbps: number;
  networkType: 'fiber' | 'cable' | 'wifi' | 'other';
}

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
  bitrateKbps: number;
  keyframeIntervalSecs: number;
  preset: string | null;
  rateControl: string | null;
}

export type EncoderType = 'nvencH264' | 'quickSync' | 'amdVce' | 'x264' | 'x265' | 'other';

export interface RecommendedSettings {
  video: RecommendedVideoSettings;
  audio: RecommendedAudioSettings;
  output: RecommendedOutputSettings;
  reasons: string[];
  overallScore: number;
}

export interface RecommendedVideoSettings {
  outputWidth: number;
  outputHeight: number;
  fps: number;
  downscaleFilter: string;
}

export interface RecommendedAudioSettings {
  sampleRate: number;
  bitrateKbps: number;
}

export interface RecommendedOutputSettings {
  encoder: string;
  bitrateKbps: number;
  keyframeIntervalSecs: number;
  preset: string | null;
  rateControl: string;
}

export type AlertSeverity = 'critical' | 'warning' | 'info' | 'tips';
export type MetricType = 'cpuUsage' | 'gpuUsage' | 'memoryUsage' | 'frameDropRate' | 'networkBandwidth';

export interface Alert {
  id: string;
  metric: MetricType;
  currentValue: number;
  threshold: number;
  severity: AlertSeverity;
  message: string;
  timestamp: number;
  active: boolean;
}

// ========================================
// Phase 2a追加型定義
// ========================================

/** プロファイル設定 */
export interface ProfileSettings {
  video: ProfileVideoSettings;
  audio: ProfileAudioSettings;
  output: ProfileOutputSettings;
}

/** プロファイル用ビデオ設定 */
export interface ProfileVideoSettings {
  outputWidth: number;
  outputHeight: number;
  fps: number;
  downscaleFilter: string;
}

/** プロファイル用音声設定 */
export interface ProfileAudioSettings {
  sampleRate: number;
  bitrateKbps: number;
}

/** プロファイル用出力設定 */
export interface ProfileOutputSettings {
  encoder: string;
  bitrateKbps: number;
  keyframeIntervalSecs: number;
  preset: string | null;
  rateControl: string;
}

/** 設定プロファイル */
export interface SettingsProfile {
  id: string;
  name: string;
  description: string;
  platform: StreamingPlatform;
  style: StreamingStyle;
  settings: ProfileSettings;
  createdAt: number;
  updatedAt: number;
}

/** プロファイル概要（一覧表示用） */
export interface ProfileSummary {
  id: string;
  name: string;
  description: string;
  platform: StreamingPlatform;
  style: StreamingStyle;
  createdAt: number;
  updatedAt: number;
}

/** バックアップ情報 */
export interface BackupInfo {
  id: string;
  createdAt: number;
  description: string;
  settings: ProfileSettings;
}

// ========================================
// Phase 2b: 問題分析関連の型
// ========================================

export type ProblemCategory = 'encoding' | 'network' | 'resource' | 'settings';

export interface ProblemReport {
  id: string;
  category: ProblemCategory;
  severity: AlertSeverity;
  title: string;
  description: string;
  suggestedActions: string[];
  affectedMetric: MetricType;
  detectedAt: number;
}

// ========================================
// Phase 2b: セッション履歴関連の型
// ========================================

export interface SessionSummary {
  sessionId: string;
  startTime: number;
  endTime: number;
  avgCpu: number;
  avgGpu: number;
  totalDroppedFrames: number;
  peakBitrate: number;
  qualityScore: number;
}

export interface ObsStatusSnapshot {
  streaming: boolean;
  recording: boolean;
  fps: number | null;
  renderDroppedFrames: number | null;
  outputDroppedFrames: number | null;
  streamBitrate: number | null;
}

export interface HistoricalMetrics {
  timestamp: number;
  sessionId: string;
  system: SystemMetrics;
  obs: ObsStatusSnapshot;
}

// ========================================
// Phase 2b: エクスポート関連の型
// ========================================

export interface AnalyzeProblemsRequest {
  encoderType: string;
  targetBitrate: number;
}

export interface AnalyzeProblemsResponse {
  problems: ProblemReport[];
  overallScore: number;
}

export interface ExportSessionRequest {
  sessionId: string;
}

export interface ExportJsonResponse {
  data: string;
  filename: string;
}

export interface ExportCsvResponse {
  data: string;
  filename: string;
}

export interface SessionInfo {
  sessionId: string;
  durationSecs: number;
  startedAt: number;
  endedAt: number;
}

/** 診断レポート用のシステム情報（OSを含む） */
export interface DiagnosticSystemInfo {
  os: string;
  cpuModel: string;
  totalMemoryMb: number;
  gpuModel: string | null;
}

export interface PerformanceEvaluation {
  overallScore: number;
  cpuScore: number;
  gpuScore: number;
  networkScore: number;
  stabilityScore: number;
}

export interface DiagnosticReport {
  generatedAt: number;
  session: SessionInfo;
  systemInfo: DiagnosticSystemInfo;
  problems: ProblemReport[];
  performance: PerformanceEvaluation;
  recommendationsSummary: string;
}
