import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useProfileStore } from './profileStore';
import { invoke } from '@tauri-apps/api/core';
import type { SettingsProfile } from '../types/commands';

vi.mock('@tauri-apps/api/core');

const mockInvoke = vi.mocked(invoke);

// モックデータ
const mockProfile1: SettingsProfile = {
  id: 'profile-1',
  name: 'YouTube配信用',
  description: 'YouTube配信に最適化された設定',
  platform: 'youTube',
  style: 'gaming',
  settings: {
    video: {
      outputWidth: 1920,
      outputHeight: 1080,
      fps: 60,
      downscaleFilter: 'lanczos',
    },
    audio: {
      sampleRate: 48000,
      bitrateKbps: 160,
    },
    output: {
      encoder: 'nvenc_h264',
      bitrateKbps: 8000,
      keyframeIntervalSecs: 2,
      preset: 'quality',
      rateControl: 'cbr',
    },
  },
  createdAt: Date.now() - 86400000, // 1日前
  updatedAt: Date.now() - 86400000,
};

const mockProfile2: SettingsProfile = {
  id: 'profile-2',
  name: 'Twitch配信用',
  description: 'Twitch配信に最適化された設定',
  platform: 'twitch',
  style: 'talk',
  settings: {
    video: {
      outputWidth: 1920,
      outputHeight: 1080,
      fps: 30,
      downscaleFilter: 'bicubic',
    },
    audio: {
      sampleRate: 48000,
      bitrateKbps: 128,
    },
    output: {
      encoder: 'x264',
      bitrateKbps: 6000,
      keyframeIntervalSecs: 2,
      preset: 'veryfast',
      rateControl: 'cbr',
    },
  },
  createdAt: Date.now() - 172800000, // 2日前
  updatedAt: Date.now(),
};

const mockProfiles: SettingsProfile[] = [mockProfile1, mockProfile2];

describe('profileStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // ストアをリセット
    useProfileStore.setState({
      profiles: [],
      selectedProfileId: null,
      isLoading: false,
      error: null,
    });
  });

  describe('初期状態', () => {
    it('空の状態で初期化される', () => {
      const state = useProfileStore.getState();
      expect(state.profiles).toEqual([]);
      expect(state.selectedProfileId).toBeNull();
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();
    });
  });

  describe('loadProfiles', () => {
    it('プロファイル一覧を取得できる', async () => {
      mockInvoke.mockResolvedValue(mockProfiles);

      const { loadProfiles } = useProfileStore.getState();
      await loadProfiles();

      const state = useProfileStore.getState();
      expect(state.profiles).toEqual(mockProfiles);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();

      expect(mockInvoke).toHaveBeenCalledWith('get_profiles');
    });

    it('読み込み中はローディング状態になる', async () => {
      mockInvoke.mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(() => resolve(mockProfiles), 100);
          })
      );

      const { loadProfiles } = useProfileStore.getState();
      const promise = loadProfiles();

      // 読み込み開始直後はローディング中
      expect(useProfileStore.getState().isLoading).toBe(true);

      await promise;

      // 完了後はローディング解除
      expect(useProfileStore.getState().isLoading).toBe(false);
    });

    it('エラー時にエラー状態を設定する', async () => {
      const errorMessage = 'Failed to load profiles';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      const { loadProfiles } = useProfileStore.getState();
      await loadProfiles();

      const state = useProfileStore.getState();
      expect(state.error).toBe(errorMessage);
      expect(state.isLoading).toBe(false);
    });

    it('エラーが文字列の場合も処理できる', async () => {
      mockInvoke.mockRejectedValue('String error');

      const { loadProfiles } = useProfileStore.getState();
      await loadProfiles();

      const state = useProfileStore.getState();
      expect(state.error).toBe('String error');
    });

    it('空のプロファイル一覧も処理できる', async () => {
      mockInvoke.mockResolvedValue([]);

      const { loadProfiles } = useProfileStore.getState();
      await loadProfiles();

      const state = useProfileStore.getState();
      expect(state.profiles).toEqual([]);
      expect(state.error).toBeNull();
    });
  });

  describe('saveProfile', () => {
    it('新しいプロファイルを保存できる', async () => {
      mockInvoke.mockResolvedValue(undefined);

      const { saveProfile } = useProfileStore.getState();
      await saveProfile(mockProfile1);

      const state = useProfileStore.getState();
      expect(state.profiles).toContainEqual(mockProfile1);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();

      expect(mockInvoke).toHaveBeenCalledWith('save_profile', { profile: mockProfile1 });
    });

    it('既存のプロファイルを更新できる', async () => {
      // 最初にプロファイルを追加
      useProfileStore.setState({ profiles: [mockProfile1] });

      mockInvoke.mockResolvedValue(undefined);

      const updatedProfile: SettingsProfile = {
        ...mockProfile1,
        name: '更新されたプロファイル',
        description: '更新された説明',
      };

      const { saveProfile } = useProfileStore.getState();
      await saveProfile(updatedProfile);

      const state = useProfileStore.getState();
      expect(state.profiles).toHaveLength(1);
      expect(state.profiles[0].name).toBe('更新されたプロファイル');
      expect(state.profiles[0].description).toBe('更新された説明');
    });

    it('保存中はローディング状態になる', async () => {
      mockInvoke.mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(() => resolve(undefined), 100);
          })
      );

      const { saveProfile } = useProfileStore.getState();
      const promise = saveProfile(mockProfile1);

      // 保存開始直後はローディング中
      expect(useProfileStore.getState().isLoading).toBe(true);

      await promise;

      // 完了後はローディング解除
      expect(useProfileStore.getState().isLoading).toBe(false);
    });

    it('保存失敗時にエラーを投げる', async () => {
      const errorMessage = 'Failed to save profile';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      const { saveProfile } = useProfileStore.getState();

      await expect(saveProfile(mockProfile1)).rejects.toThrow(errorMessage);

      const state = useProfileStore.getState();
      expect(state.error).toBe(errorMessage);
      expect(state.isLoading).toBe(false);
    });

    it('複数のプロファイルを保存できる', async () => {
      mockInvoke.mockResolvedValue(undefined);

      const { saveProfile } = useProfileStore.getState();

      await saveProfile(mockProfile1);
      await saveProfile(mockProfile2);

      const state = useProfileStore.getState();
      expect(state.profiles).toHaveLength(2);
      expect(state.profiles).toContainEqual(mockProfile1);
      expect(state.profiles).toContainEqual(mockProfile2);
    });
  });

  describe('deleteProfile', () => {
    it('プロファイルを削除できる', async () => {
      useProfileStore.setState({ profiles: [mockProfile1, mockProfile2] });

      mockInvoke.mockResolvedValue(undefined);

      const { deleteProfile } = useProfileStore.getState();
      await deleteProfile('profile-1');

      const state = useProfileStore.getState();
      expect(state.profiles).toHaveLength(1);
      expect(state.profiles[0].id).toBe('profile-2');
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();

      expect(mockInvoke).toHaveBeenCalledWith('delete_profile', { id: 'profile-1' });
    });

    it('選択中のプロファイルを削除すると選択が解除される', async () => {
      useProfileStore.setState({
        profiles: [mockProfile1, mockProfile2],
        selectedProfileId: 'profile-1',
      });

      mockInvoke.mockResolvedValue(undefined);

      const { deleteProfile } = useProfileStore.getState();
      await deleteProfile('profile-1');

      const state = useProfileStore.getState();
      expect(state.selectedProfileId).toBeNull();
    });

    it('選択されていないプロファイルを削除しても選択は維持される', async () => {
      useProfileStore.setState({
        profiles: [mockProfile1, mockProfile2],
        selectedProfileId: 'profile-1',
      });

      mockInvoke.mockResolvedValue(undefined);

      const { deleteProfile } = useProfileStore.getState();
      await deleteProfile('profile-2');

      const state = useProfileStore.getState();
      expect(state.selectedProfileId).toBe('profile-1');
    });

    it('削除中はローディング状態になる', async () => {
      useProfileStore.setState({ profiles: [mockProfile1] });

      mockInvoke.mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(() => resolve(undefined), 100);
          })
      );

      const { deleteProfile } = useProfileStore.getState();
      const promise = deleteProfile('profile-1');

      // 削除開始直後はローディング中
      expect(useProfileStore.getState().isLoading).toBe(true);

      await promise;

      // 完了後はローディング解除
      expect(useProfileStore.getState().isLoading).toBe(false);
    });

    it('削除失敗時にエラーを投げる', async () => {
      useProfileStore.setState({ profiles: [mockProfile1] });

      const errorMessage = 'Failed to delete profile';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      const { deleteProfile } = useProfileStore.getState();

      await expect(deleteProfile('profile-1')).rejects.toThrow(errorMessage);

      const state = useProfileStore.getState();
      expect(state.error).toBe(errorMessage);
    });

    it('全てのプロファイルを削除できる', async () => {
      useProfileStore.setState({ profiles: [mockProfile1, mockProfile2] });

      mockInvoke.mockResolvedValue(undefined);

      const { deleteProfile } = useProfileStore.getState();

      await deleteProfile('profile-1');
      await deleteProfile('profile-2');

      const state = useProfileStore.getState();
      expect(state.profiles).toEqual([]);
    });
  });

  describe('applyProfile', () => {
    it('プロファイルを適用できる', async () => {
      mockInvoke.mockResolvedValue(undefined);

      const { applyProfile } = useProfileStore.getState();
      await applyProfile('profile-1');

      const state = useProfileStore.getState();
      expect(state.selectedProfileId).toBe('profile-1');
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();

      expect(mockInvoke).toHaveBeenCalledWith('apply_profile', { id: 'profile-1' });
    });

    it('適用中はローディング状態になる', async () => {
      mockInvoke.mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(() => resolve(undefined), 100);
          })
      );

      const { applyProfile } = useProfileStore.getState();
      const promise = applyProfile('profile-1');

      // 適用開始直後はローディング中
      expect(useProfileStore.getState().isLoading).toBe(true);

      await promise;

      // 完了後はローディング解除
      expect(useProfileStore.getState().isLoading).toBe(false);
    });

    it('適用失敗時にエラーを投げる', async () => {
      const errorMessage = 'Failed to apply profile';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      const { applyProfile } = useProfileStore.getState();

      await expect(applyProfile('profile-1')).rejects.toThrow(errorMessage);

      const state = useProfileStore.getState();
      expect(state.error).toBe(errorMessage);
    });

    it('異なるプロファイルを連続して適用できる', async () => {
      mockInvoke.mockResolvedValue(undefined);

      const { applyProfile } = useProfileStore.getState();

      await applyProfile('profile-1');
      expect(useProfileStore.getState().selectedProfileId).toBe('profile-1');

      await applyProfile('profile-2');
      expect(useProfileStore.getState().selectedProfileId).toBe('profile-2');
    });
  });

  describe('setSelectedProfile', () => {
    it('選択中のプロファイルIDを設定できる', () => {
      const { setSelectedProfile } = useProfileStore.getState();

      setSelectedProfile('profile-1');

      const state = useProfileStore.getState();
      expect(state.selectedProfileId).toBe('profile-1');
    });

    it('選択を解除できる', () => {
      useProfileStore.setState({ selectedProfileId: 'profile-1' });

      const { setSelectedProfile } = useProfileStore.getState();
      setSelectedProfile(null);

      const state = useProfileStore.getState();
      expect(state.selectedProfileId).toBeNull();
    });

    it('選択を変更できる', () => {
      useProfileStore.setState({ selectedProfileId: 'profile-1' });

      const { setSelectedProfile } = useProfileStore.getState();
      setSelectedProfile('profile-2');

      const state = useProfileStore.getState();
      expect(state.selectedProfileId).toBe('profile-2');
    });
  });

  describe('clearError', () => {
    it('エラーメッセージをクリアできる', () => {
      useProfileStore.setState({ error: 'エラーメッセージ' });

      const { clearError } = useProfileStore.getState();
      clearError();

      const state = useProfileStore.getState();
      expect(state.error).toBeNull();
    });

    it('エラーがない状態でclearErrorを呼んでもエラーにならない', () => {
      const { clearError } = useProfileStore.getState();

      expect(() => clearError()).not.toThrow();

      const state = useProfileStore.getState();
      expect(state.error).toBeNull();
    });
  });

  describe('複合操作', () => {
    it('読み込み→保存→適用の一連の流れが正しく動作する', async () => {
      mockInvoke.mockResolvedValueOnce(mockProfiles);

      const { loadProfiles, saveProfile, applyProfile } = useProfileStore.getState();

      // プロファイル一覧を読み込む
      await loadProfiles();
      expect(useProfileStore.getState().profiles).toHaveLength(2);

      // 新しいプロファイルを保存
      const newProfile: SettingsProfile = {
        ...mockProfile1,
        id: 'profile-3',
        name: '新規プロファイル',
      };

      mockInvoke.mockResolvedValueOnce(undefined);
      await saveProfile(newProfile);
      expect(useProfileStore.getState().profiles).toHaveLength(3);

      // プロファイルを適用
      mockInvoke.mockResolvedValueOnce(undefined);
      await applyProfile('profile-3');
      expect(useProfileStore.getState().selectedProfileId).toBe('profile-3');
    });

    it('保存→適用→削除の一連の流れが正しく動作する', async () => {
      mockInvoke.mockResolvedValue(undefined);

      const { saveProfile, applyProfile, deleteProfile } = useProfileStore.getState();

      // プロファイルを保存
      await saveProfile(mockProfile1);
      expect(useProfileStore.getState().profiles).toHaveLength(1);

      // プロファイルを適用
      await applyProfile('profile-1');
      expect(useProfileStore.getState().selectedProfileId).toBe('profile-1');

      // プロファイルを削除
      await deleteProfile('profile-1');
      expect(useProfileStore.getState().profiles).toHaveLength(0);
      expect(useProfileStore.getState().selectedProfileId).toBeNull();
    });

    it('エラー発生後にclearErrorを呼んで再実行できる', async () => {
      // 最初はエラー
      mockInvoke.mockRejectedValueOnce(new Error('Initial error'));

      const { loadProfiles, clearError } = useProfileStore.getState();
      await loadProfiles();

      expect(useProfileStore.getState().error).toBe('Initial error');

      // エラーをクリア
      clearError();
      expect(useProfileStore.getState().error).toBeNull();

      // 再読み込み成功
      mockInvoke.mockResolvedValueOnce(mockProfiles);
      await loadProfiles();

      const state = useProfileStore.getState();
      expect(state.error).toBeNull();
      expect(state.profiles).toEqual(mockProfiles);
    });
  });

  describe('全てのplatformとstyleの組み合わせ', () => {
    it('全てのplatformのプロファイルを保存できる', async () => {
      mockInvoke.mockResolvedValue(undefined);

      const platforms = ['youTube', 'twitch', 'nicoNico', 'other'] as const;
      const { saveProfile } = useProfileStore.getState();

      for (const platform of platforms) {
        const profile: SettingsProfile = {
          ...mockProfile1,
          id: `profile-${platform}`,
          platform,
        };
        await saveProfile(profile);
      }

      const state = useProfileStore.getState();
      expect(state.profiles).toHaveLength(4);
      platforms.forEach((platform) => {
        const profile = state.profiles.find((p) => p.platform === platform);
        expect(profile).toBeDefined();
      });
    });

    it('全てのstyleのプロファイルを保存できる', async () => {
      mockInvoke.mockResolvedValue(undefined);

      const styles = ['talk', 'gaming', 'music', 'art', 'other'] as const;
      const { saveProfile } = useProfileStore.getState();

      for (const style of styles) {
        const profile: SettingsProfile = {
          ...mockProfile1,
          id: `profile-${style}`,
          style,
        };
        await saveProfile(profile);
      }

      const state = useProfileStore.getState();
      expect(state.profiles).toHaveLength(5);
      styles.forEach((style) => {
        const profile = state.profiles.find((p) => p.style === style);
        expect(profile).toBeDefined();
      });
    });
  });
});
