import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { SimpleAppConfig } from '../types/commands';

// ========================================
// ストア状態の型定義
// ========================================

interface ConfigState {
  config: SimpleAppConfig | null;
  loading: boolean;
  error: string | null;

  // アクション
  loadConfig: () => Promise<void>;
  saveConfig: (config: SimpleAppConfig) => Promise<void>;
  updateConfig: (partial: Partial<SimpleAppConfig>) => Promise<void>;
  clearError: () => void;
}

// ========================================
// デフォルト設定
// ========================================

const DEFAULT_CONFIG: SimpleAppConfig = {
  saveConnection: false,
  autoConnect: false,
  streamStyle: null,
  platform: null,
  onboardingCompleted: false,
  streamingModeEnabled: false,
};

// ========================================
// ストア実装
// ========================================

export const useConfigStore = create<ConfigState>((set, get) => ({
  config: null,
  loading: false,
  error: null,

  loadConfig: async () => {
    set({ loading: true, error: null });
    try {
      const config = await invoke<SimpleAppConfig>('get_config');
      set({ config, loading: false });
    } catch (e) {
      // 設定ファイルが存在しない場合はデフォルト設定を使用
      const errorMessage = e instanceof Error ? e.message : String(e);
      if (errorMessage.includes('not found') || errorMessage.includes('存在しません')) {
        set({ config: DEFAULT_CONFIG, loading: false, error: null });
      } else {
        set({
          error: e instanceof Error ? e.message : String(e),
          loading: false,
        });
      }
    }
  },

  saveConfig: async (config: SimpleAppConfig) => {
    set({ loading: true, error: null });
    try {
      await invoke('save_app_config', { config });
      set({ config, loading: false });
    } catch (e) {
      set({
        error: e instanceof Error ? e.message : String(e),
        loading: false,
      });
      throw e;
    }
  },

  updateConfig: async (partial: Partial<SimpleAppConfig>) => {
    const { config, saveConfig } = get();
    // configがnullの場合はデフォルト設定を使用
    const baseConfig = config ?? DEFAULT_CONFIG;
    const updatedConfig = { ...baseConfig, ...partial };
    await saveConfig(updatedConfig);
  },

  clearError: () => {
    set({ error: null });
  },
}));
