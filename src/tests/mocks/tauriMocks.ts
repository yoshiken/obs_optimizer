import { vi } from 'vitest';
import type {
  ObsStatus,
  SystemMetrics,
  ObsProcessMetrics,
  ObsConnectionParams,
} from '../../types/commands';

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
export function setupInvokeMock(invoke: ReturnType<typeof vi.fn>) {
  invoke.mockImplementation(async (command: string, args?: unknown) => {
    switch (command) {
      case 'get_obs_status':
        return mockObsStatus;
      case 'get_system_metrics':
        return mockSystemMetrics;
      case 'get_process_metrics':
        return mockObsProcessMetrics;
      case 'get_scene_list':
        return mockSceneList;
      case 'connect_obs':
        return undefined;
      case 'disconnect_obs':
        return undefined;
      case 'start_streaming':
        return undefined;
      case 'stop_streaming':
        return undefined;
      case 'start_recording':
        return undefined;
      case 'stop_recording':
        return 'C:\\recordings\\video.mp4';
      case 'set_current_scene':
        return undefined;
      default:
        throw new Error(`Unknown command: ${command}`);
    }
  });
}

/**
 * Tauri listenイベントのモックを設定するヘルパー関数
 */
export function setupListenMock(listen: ReturnType<typeof vi.fn>) {
  listen.mockImplementation(async (eventName: string, handler: (event: unknown) => void) => {
    // アンリスナー関数を返す
    return () => {
      // クリーンアップ処理
    };
  });
}

/**
 * エラーを投げるinvokeモックを設定
 */
export function setupInvokeErrorMock(invoke: ReturnType<typeof vi.fn>, errorMessage = 'Mock error') {
  invoke.mockRejectedValue(new Error(errorMessage));
}
