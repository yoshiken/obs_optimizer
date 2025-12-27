import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { SettingsProfile } from '../types/commands';

interface ProfileState {
  // 状態
  profiles: SettingsProfile[];
  selectedProfileId: string | null;
  isLoading: boolean;
  error: string | null;

  // アクション
  loadProfiles: () => Promise<void>;
  saveProfile: (profile: SettingsProfile) => Promise<void>;
  deleteProfile: (id: string) => Promise<void>;
  applyProfile: (id: string) => Promise<void>;
  setSelectedProfile: (id: string | null) => void;
  clearError: () => void;
}

export const useProfileStore = create<ProfileState>((set, get) => ({
  // 初期状態
  profiles: [],
  selectedProfileId: null,
  isLoading: false,
  error: null,

  // プロファイル一覧の読み込み
  loadProfiles: async () => {
    set({ isLoading: true, error: null });
    try {
      const profiles = await invoke<SettingsProfile[]>('get_profiles');
      set({ profiles, isLoading: false });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      set({ error: errorMessage, isLoading: false });
    }
  },

  // プロファイルの保存
  saveProfile: async (profile: SettingsProfile) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('save_profile', { profile });

      // ローカル状態を更新
      const { profiles } = get();
      const existingIndex = profiles.findIndex((p) => p.id === profile.id);

      let updatedProfiles: SettingsProfile[];
      if (existingIndex >= 0) {
        // 既存プロファイルの更新
        updatedProfiles = [...profiles];
        updatedProfiles[existingIndex] = profile;
      } else {
        // 新規プロファイルの追加
        updatedProfiles = [...profiles, profile];
      }

      set({ profiles: updatedProfiles, isLoading: false });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      set({ error: errorMessage, isLoading: false });
      throw error;
    }
  },

  // プロファイルの削除
  deleteProfile: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('delete_profile', { id });

      // ローカル状態を更新
      const { profiles, selectedProfileId } = get();
      const updatedProfiles = profiles.filter((p) => p.id !== id);
      const newSelectedId = selectedProfileId === id ? null : selectedProfileId;

      set({
        profiles: updatedProfiles,
        selectedProfileId: newSelectedId,
        isLoading: false
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      set({ error: errorMessage, isLoading: false });
      throw error;
    }
  },

  // プロファイルの適用
  applyProfile: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('apply_profile', { id });
      set({ selectedProfileId: id, isLoading: false });
      // プロファイル一覧を再取得（プロファイル適用後の最新状態を反映）
      await get().loadProfiles();
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      set({ error: errorMessage, isLoading: false });
      throw error;
    }
  },

  // 選択中のプロファイルを設定
  setSelectedProfile: (id: string | null) => {
    set({ selectedProfileId: id });
  },

  // エラークリア
  clearError: () => {
    set({ error: null });
  },
}));
