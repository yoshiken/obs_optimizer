import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  ConnectionChangedPayload,
  ConnectionState,
  ObsConnectionParams,
  ObsErrorPayload,
  ObsStatus,
  RecordingChangedPayload,
  SceneChangedPayload,
  StreamingChangedPayload,
} from '../types/commands';
import { OBS_EVENTS } from '../types/commands';

/**
 * エラーからメッセージを抽出するヘルパー関数
 * Tauriのinvokeエラーは様々な形式で返されるため、適切に処理する
 */
function extractErrorMessage(e: unknown): string {
  if (e instanceof Error) {
    return e.message;
  }
  if (typeof e === 'string') {
    return e;
  }
  if (e && typeof e === 'object') {
    // Tauriエラー形式: { message: string } または { error: string }
    if ('message' in e && typeof (e as { message: unknown }).message === 'string') {
      return (e as { message: string }).message;
    }
    if ('error' in e && typeof (e as { error: unknown }).error === 'string') {
      return (e as { error: string }).error;
    }
    // JSON.stringifyでオブジェクトを文字列化（[object Object]を回避）
    try {
      return JSON.stringify(e);
    } catch {
      return '不明なエラーが発生しました';
    }
  }
  return '不明なエラーが発生しました';
}

// ========================================
// ストア状態の型定義
// ========================================

interface ObsState {
  // 接続状態
  connectionState: ConnectionState;
  status: ObsStatus | null;
  error: string | null;
  /** バックグラウンド処理での警告（ステータス取得失敗など、致命的でないエラー） */
  warning: string | null;
  loading: boolean;

  // シーン一覧
  scenes: string[];

  // 接続設定（前回の設定を保持）
  lastConnectionParams: ObsConnectionParams | null;

  // アクション
  connect: (params: ObsConnectionParams) => Promise<void>;
  disconnect: () => Promise<void>;
  fetchStatus: () => Promise<void>;
  fetchScenes: () => Promise<void>;
  setCurrentScene: (sceneName: string) => Promise<void>;
  startStreaming: () => Promise<void>;
  stopStreaming: () => Promise<void>;
  startRecording: () => Promise<void>;
  stopRecording: () => Promise<string>;
  startPolling: (intervalMs?: number) => () => void;
  subscribeToEvents: () => Promise<() => void>;
  clearError: () => void;
  clearWarning: () => void;
}

// ========================================
// ストア実装
// ========================================

export const useObsStore = create<ObsState>((set, get) => ({
  connectionState: 'disconnected',
  status: null,
  error: null,
  warning: null,
  loading: false,
  scenes: [],
  lastConnectionParams: null,

  connect: async (params: ObsConnectionParams) => {
    set({ loading: true, error: null, connectionState: 'connecting' });
    try {
      await invoke('connect_obs', { params });
      set({
        connectionState: 'connected',
        loading: false,
        lastConnectionParams: params,
      });
      // 接続成功後、ステータスとシーン一覧を取得
      const { fetchStatus, fetchScenes } = get();
      await Promise.all([fetchStatus(), fetchScenes()]);
    } catch (e) {
      set({
        connectionState: 'error',
        error: extractErrorMessage(e),
        loading: false,
      });
      throw e;
    }
  },

  disconnect: async () => {
    set({ loading: true, error: null });
    try {
      await invoke('disconnect_obs');
      set({
        connectionState: 'disconnected',
        status: null,
        scenes: [],
        loading: false,
      });
    } catch (e) {
      set({
        error: extractErrorMessage(e),
        loading: false,
      });
      throw e;
    }
  },

  fetchStatus: async () => {
    try {
      const status = await invoke<ObsStatus>('get_obs_status');
      set({ status, warning: null }); // 成功時は警告をクリア
      // 接続状態を更新
      if (status.connected && get().connectionState !== 'connected') {
        set({ connectionState: 'connected' });
      } else if (!status.connected && get().connectionState === 'connected') {
        set({ connectionState: 'disconnected' });
      }
    } catch (e) {
      // ステータス取得失敗時は警告として記録（接続状態は変更しない）
      const message = extractErrorMessage(e);
      set({ warning: `ステータス取得失敗: ${message}` });
    }
  },

  fetchScenes: async () => {
    try {
      const scenes = await invoke<string[]>('get_scene_list');
      set({ scenes });
    } catch (e) {
      // シーン取得失敗時は警告として記録
      const message = extractErrorMessage(e);
      set({ scenes: [], warning: `シーン一覧取得失敗: ${message}` });
    }
  },

  setCurrentScene: async (sceneName: string) => {
    set({ loading: true, error: null });
    try {
      await invoke('set_current_scene', { sceneName });
      // シーン変更後、ステータスを更新
      await get().fetchStatus();
      set({ loading: false });
    } catch (e) {
      set({
        error: extractErrorMessage(e),
        loading: false,
      });
      throw e;
    }
  },

  startStreaming: async () => {
    set({ loading: true, error: null });
    try {
      await invoke('start_streaming');
      await get().fetchStatus();
      set({ loading: false });
    } catch (e) {
      set({
        error: extractErrorMessage(e),
        loading: false,
      });
      throw e;
    }
  },

  stopStreaming: async () => {
    set({ loading: true, error: null });
    try {
      await invoke('stop_streaming');
      await get().fetchStatus();
      set({ loading: false });
    } catch (e) {
      set({
        error: extractErrorMessage(e),
        loading: false,
      });
      throw e;
    }
  },

  startRecording: async () => {
    set({ loading: true, error: null });
    try {
      await invoke('start_recording');
      await get().fetchStatus();
      set({ loading: false });
    } catch (e) {
      set({
        error: extractErrorMessage(e),
        loading: false,
      });
      throw e;
    }
  },

  stopRecording: async () => {
    set({ loading: true, error: null });
    try {
      const outputPath = await invoke<string>('stop_recording');
      await get().fetchStatus();
      set({ loading: false });
      return outputPath;
    } catch (e) {
      set({
        error: extractErrorMessage(e),
        loading: false,
      });
      throw e;
    }
  },

  startPolling: (intervalMs = 1000) => {
    const { fetchStatus, fetchScenes } = get();

    // 初回取得
    void fetchStatus();
    void fetchScenes();

    // 定期取得
    const statusIntervalId = setInterval(fetchStatus, intervalMs);
    // シーン一覧は変更頻度が低いので、5秒ごとに更新
    const scenesIntervalId = setInterval(fetchScenes, 5000);

    return () => {
      clearInterval(statusIntervalId);
      clearInterval(scenesIntervalId);
    };
  },

  subscribeToEvents: async () => {
    const unlisteners: UnlistenFn[] = [];

    // 接続状態変更イベント
    const unlistenConnection = await listen<ConnectionChangedPayload>(
      OBS_EVENTS.CONNECTION_CHANGED,
      (event) => {
        set({ connectionState: event.payload.currentState });
      }
    );
    unlisteners.push(unlistenConnection);

    // 配信状態変更イベント
    const unlistenStreaming = await listen<StreamingChangedPayload>(
      OBS_EVENTS.STREAMING_CHANGED,
      (event) => {
        const { status } = get();
        if (status) {
          set({
            status: {
              ...status,
              streaming: event.payload.isStreaming,
            },
          });
        }
      }
    );
    unlisteners.push(unlistenStreaming);

    // 録画状態変更イベント
    const unlistenRecording = await listen<RecordingChangedPayload>(
      OBS_EVENTS.RECORDING_CHANGED,
      (event) => {
        const { status } = get();
        if (status) {
          set({
            status: {
              ...status,
              recording: event.payload.isRecording,
            },
          });
        }
      }
    );
    unlisteners.push(unlistenRecording);

    // シーン変更イベント
    const unlistenScene = await listen<SceneChangedPayload>(
      OBS_EVENTS.SCENE_CHANGED,
      (event) => {
        const { status } = get();
        if (status) {
          set({
            status: {
              ...status,
              currentScene: event.payload.currentScene,
            },
          });
        }
      }
    );
    unlisteners.push(unlistenScene);

    // ステータス更新イベント
    const unlistenStatus = await listen<ObsStatus>(
      OBS_EVENTS.STATUS_UPDATE,
      (event) => {
        set({ status: event.payload });
      }
    );
    unlisteners.push(unlistenStatus);

    // エラーイベント
    const unlistenError = await listen<ObsErrorPayload>(
      OBS_EVENTS.ERROR,
      (event) => {
        set({ error: event.payload.message });
        if (!event.payload.recoverable) {
          set({ connectionState: 'error' });
        }
      }
    );
    unlisteners.push(unlistenError);

    // クリーンアップ関数を返す
    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  },

  clearError: () => {
    set({ error: null });
  },

  clearWarning: () => {
    set({ warning: null });
  },
}));
