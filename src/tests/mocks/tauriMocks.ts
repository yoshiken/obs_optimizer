import type {
  ObsProcessMetrics,
  ObsStatus,
  SystemMetrics,
} from '../../types/commands';

// Tauriコマンド用のモック型定義
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type InvokeMockType = { mockImplementation: (fn: (...args: any[]) => any) => void; mockRejectedValue: (error: Error) => void };
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type ListenMockType = { mockImplementation: (fn: (...args: any[]) => any) => void };

/**
 * Tauriコマンドのモックデータとヘルパー関数
 */

// モックデータ: OBSステータス
export const mockObsStatus: ObsStatus = {
  connected: true,
  streaming: false,
  recording: false,
  virtualCamActive: false,
  currentScene: 'テストシーン',
  obsVersion: '30.0.0',
  websocketVersion: '5.0.0',
  streamTimecode: null,
  recordTimecode: null,
  streamBitrate: null,
  recordBitrate: null,
  fps: 60.0,
  renderDroppedFrames: 0,
  outputDroppedFrames: 0,
};

// モックデータ: システムメトリクス
export const mockSystemMetrics: SystemMetrics = {
  cpu: {
    usagePercent: 45.5,
    coreCount: 8,
    perCoreUsage: [40, 50, 45, 42, 48, 43, 47, 44],
    cpuName: 'Intel(R) Core(TM) i7-9700K CPU @ 3.60GHz',
  },
  memory: {
    totalBytes: 16000000000,
    usedBytes: 8000000000,
    availableBytes: 8000000000,
    usagePercent: 50.0,
  },
  gpu: {
    name: 'NVIDIA GeForce RTX 3060',
    usagePercent: 30.0,
    memoryUsedBytes: 2000000000,
    memoryTotalBytes: 6000000000,
    encoderUsage: 15.0,
  },
  network: {
    uploadBytesPerSec: 5000000,
    downloadBytesPerSec: 10000000,
  },
};

// モックデータ: OBSプロセスメトリクス
export const mockObsProcessMetrics: ObsProcessMetrics = {
  mainProcess: {
    name: 'obs64.exe',
    pid: 12345,
    cpuUsage: 10.5,
    memoryBytes: 500000000,
    isRunning: true,
  },
  totalCpuUsage: 10.5,
  totalMemoryBytes: 500000000,
};

// モックデータ: シーンリスト
export const mockSceneList = ['シーン1', 'シーン2', 'ゲームシーン', 'トークシーン'];

/**
 * Tauri invokeコマンドのモックを設定するヘルパー関数
 */
export function setupInvokeMock(invoke: InvokeMockType) {
  invoke.mockImplementation((command: string): Promise<unknown> => {
    switch (command) {
      case 'get_obs_status':
        return Promise.resolve(mockObsStatus);
      case 'get_system_metrics':
        return Promise.resolve(mockSystemMetrics);
      case 'get_process_metrics':
        return Promise.resolve(mockObsProcessMetrics);
      case 'get_scene_list':
        return Promise.resolve(mockSceneList);
      case 'connect_obs':
        return Promise.resolve(undefined);
      case 'disconnect_obs':
        return Promise.resolve(undefined);
      case 'start_streaming':
        return Promise.resolve(undefined);
      case 'stop_streaming':
        return Promise.resolve(undefined);
      case 'start_recording':
        return Promise.resolve(undefined);
      case 'stop_recording':
        return Promise.resolve('C:\\recordings\\video.mp4');
      case 'set_current_scene':
        return Promise.resolve(undefined);
      default:
        return Promise.reject(new Error(`Unknown command: ${command}`));
    }
  });
}

/**
 * Tauri listenイベントのモックを設定するヘルパー関数
 */
export function setupListenMock(listen: ListenMockType) {
  listen.mockImplementation((): Promise<() => void> => {
    // アンリスナー関数を返す
    return Promise.resolve(() => {
      // クリーンアップ処理
    });
  });
}

/**
 * エラーを投げるinvokeモックを設定
 */
export function setupInvokeErrorMock(invoke: InvokeMockType, errorMessage = 'Mock error') {
  invoke.mockRejectedValue(new Error(errorMessage));
}
