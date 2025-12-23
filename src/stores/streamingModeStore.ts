import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { StreamingChangedPayload } from '../types/commands';
import { OBS_EVENTS } from '../types/commands';

interface StreamingModeState {
  // 状態
  isEnabled: boolean;
  isLoading: boolean;
  error: string | null;

  // アクション
  setEnabled: (enabled: boolean) => Promise<void>;
  loadStreamingMode: () => Promise<void>;
  initializeAutoMode: () => void;
  clearError: () => void;
}

export const useStreamingModeStore = create<StreamingModeState>((set) => ({
  // 初期状態
  isEnabled: false,
  isLoading: false,
  error: null,

  // 配信中モードの有効化/無効化
  setEnabled: async (enabled: boolean) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('set_streaming_mode', { enabled });
      set({ isEnabled: enabled, isLoading: false });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      set({ error: errorMessage, isLoading: false });
      throw error;
    }
  },

  // 現在の配信中モード状態を読み込み
  loadStreamingMode: async () => {
    set({ isLoading: true, error: null });
    try {
      const enabled = await invoke<boolean>('get_streaming_mode');
      set({ isEnabled: enabled, isLoading: false });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      set({ error: errorMessage, isLoading: false });
    }
  },

  // OBS配信状態との自動連動を初期化
  initializeAutoMode: () => {
    // OBS配信開始イベントのリスナーを設定
    void listen<StreamingChangedPayload>(OBS_EVENTS.STREAMING_CHANGED, (event) => {
      const { isStreaming } = event.payload;

      // 配信開始時: 自動的に配信中モードをON
      // 配信終了時: 自動的に配信中モードをOFF
      invoke('set_streaming_mode', { enabled: isStreaming })
        .then(() => {
          set({ isEnabled: isStreaming });
        })
        .catch((error) => {
          const errorMessage = error instanceof Error ? error.message : String(error);
          set({ error: errorMessage });
        });
    }).catch((error) => {
      const errorMessage = error instanceof Error ? error.message : String(error);
      set({ error: `イベントリスナーの初期化に失敗: ${errorMessage}` });
    });
  },

  // エラークリア
  clearError: () => {
    set({ error: null });
  },
}));
