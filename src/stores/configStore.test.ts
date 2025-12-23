import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useConfigStore } from './configStore';
import { invoke } from '@tauri-apps/api/core';
import type { SimpleAppConfig } from '../types/commands';

vi.mock('@tauri-apps/api/core');

const mockInvoke = vi.mocked(invoke);

// モックデータ
const mockConfig: SimpleAppConfig = {
  saveConnection: true,
  autoConnect: true,
  streamStyle: 'game',
  platform: 'youtube',
  onboardingCompleted: true,
  streamingModeEnabled: false,
};

const defaultConfig: SimpleAppConfig = {
  saveConnection: false,
  autoConnect: false,
  streamStyle: null,
  platform: null,
  onboardingCompleted: false,
  streamingModeEnabled: false,
};

describe('configStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // ストアをリセット
    useConfigStore.setState({
      config: null,
      loading: false,
      error: null,
    });
  });

  describe('初期状態', () => {
    it('設定がnullで初期化される', () => {
      const state = useConfigStore.getState();
      expect(state.config).toBeNull();
      expect(state.loading).toBe(false);
      expect(state.error).toBeNull();
    });
  });

  describe('loadConfig', () => {
    it('設定を正常に読み込める', async () => {
      mockInvoke.mockResolvedValue(mockConfig);

      const { loadConfig } = useConfigStore.getState();
      await loadConfig();

      const state = useConfigStore.getState();
      expect(state.config).toEqual(mockConfig);
      expect(state.loading).toBe(false);
      expect(state.error).toBeNull();

      expect(mockInvoke).toHaveBeenCalledWith('get_config');
    });

    it('読み込み中はローディング状態になる', async () => {
      mockInvoke.mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(() => resolve(mockConfig), 100);
          })
      );

      const { loadConfig } = useConfigStore.getState();
      const promise = loadConfig();

      // 読み込み開始直後はローディング中
      expect(useConfigStore.getState().loading).toBe(true);

      await promise;

      // 完了後はローディング解除
      expect(useConfigStore.getState().loading).toBe(false);
    });

    it('設定ファイルが存在しない場合はデフォルト設定を使用する', async () => {
      mockInvoke.mockRejectedValue(new Error('Config file not found'));

      const { loadConfig } = useConfigStore.getState();
      await loadConfig();

      const state = useConfigStore.getState();
      expect(state.config).toEqual(defaultConfig);
      expect(state.error).toBeNull();
      expect(state.loading).toBe(false);
    });

    it('「存在しません」メッセージの場合もデフォルト設定を使用する', async () => {
      mockInvoke.mockRejectedValue(new Error('設定ファイルは存在しません'));

      const { loadConfig } = useConfigStore.getState();
      await loadConfig();

      const state = useConfigStore.getState();
      expect(state.config).toEqual(defaultConfig);
      expect(state.error).toBeNull();
    });

    it('その他のエラーの場合はエラー状態を設定する', async () => {
      const errorMessage = 'Permission denied';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      const { loadConfig } = useConfigStore.getState();
      await loadConfig();

      const state = useConfigStore.getState();
      expect(state.error).toBe(errorMessage);
      expect(state.loading).toBe(false);
    });

    it('エラーが文字列の場合も処理できる', async () => {
      mockInvoke.mockRejectedValue('String error');

      const { loadConfig } = useConfigStore.getState();
      await loadConfig();

      const state = useConfigStore.getState();
      expect(state.error).toBe('String error');
    });
  });

  describe('saveConfig', () => {
    it('設定を正常に保存できる', async () => {
      mockInvoke.mockResolvedValue(undefined);

      const { saveConfig } = useConfigStore.getState();
      await saveConfig(mockConfig);

      const state = useConfigStore.getState();
      expect(state.config).toEqual(mockConfig);
      expect(state.loading).toBe(false);
      expect(state.error).toBeNull();

      expect(mockInvoke).toHaveBeenCalledWith('save_config', { config: mockConfig });
    });

    it('保存中はローディング状態になる', async () => {
      mockInvoke.mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(() => resolve(undefined), 100);
          })
      );

      const { saveConfig } = useConfigStore.getState();
      const promise = saveConfig(mockConfig);

      // 保存開始直後はローディング中
      expect(useConfigStore.getState().loading).toBe(true);

      await promise;

      // 完了後はローディング解除
      expect(useConfigStore.getState().loading).toBe(false);
    });

    it('保存失敗時にエラーを投げる', async () => {
      const errorMessage = 'Failed to save config';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      const { saveConfig } = useConfigStore.getState();

      await expect(saveConfig(mockConfig)).rejects.toThrow(errorMessage);

      const state = useConfigStore.getState();
      expect(state.error).toBe(errorMessage);
      expect(state.loading).toBe(false);
    });

    it('エラーが文字列の場合も処理できる', async () => {
      mockInvoke.mockRejectedValue('String error');

      const { saveConfig } = useConfigStore.getState();

      await expect(saveConfig(mockConfig)).rejects.toBe('String error');

      const state = useConfigStore.getState();
      expect(state.error).toBe('String error');
    });

    it('部分的な設定も保存できる', async () => {
      mockInvoke.mockResolvedValue(undefined);

      const partialConfig: SimpleAppConfig = {
        ...defaultConfig,
        saveConnection: true,
        onboardingCompleted: true,
      };

      const { saveConfig } = useConfigStore.getState();
      await saveConfig(partialConfig);

      const state = useConfigStore.getState();
      expect(state.config?.saveConnection).toBe(true);
      expect(state.config?.onboardingCompleted).toBe(true);
      expect(state.config?.autoConnect).toBe(false);
    });
  });

  describe('updateConfig', () => {
    it('部分的な設定を更新できる', async () => {
      // 最初に設定を読み込む
      useConfigStore.setState({ config: mockConfig });

      mockInvoke.mockResolvedValue(undefined);

      const { updateConfig } = useConfigStore.getState();
      await updateConfig({ saveConnection: false, autoConnect: false });

      const state = useConfigStore.getState();
      expect(state.config?.saveConnection).toBe(false);
      expect(state.config?.autoConnect).toBe(false);
      // 他のフィールドは変更されない
      expect(state.config?.streamStyle).toBe('game');
      expect(state.config?.platform).toBe('youtube');

      expect(mockInvoke).toHaveBeenCalledWith('save_config', {
        config: {
          ...mockConfig,
          saveConnection: false,
          autoConnect: false,
        },
      });
    });

    it('単一フィールドを更新できる', async () => {
      useConfigStore.setState({ config: mockConfig });

      mockInvoke.mockResolvedValue(undefined);

      const { updateConfig } = useConfigStore.getState();
      await updateConfig({ streamingModeEnabled: true });

      const state = useConfigStore.getState();
      expect(state.config?.streamingModeEnabled).toBe(true);
    });

    it('設定が読み込まれていない場合はエラーを投げる', async () => {
      useConfigStore.setState({ config: null });

      const { updateConfig } = useConfigStore.getState();

      await expect(updateConfig({ saveConnection: true })).rejects.toThrow(
        '設定が読み込まれていません'
      );
    });

    it('更新中はローディング状態になる', async () => {
      useConfigStore.setState({ config: mockConfig });

      mockInvoke.mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(() => resolve(undefined), 100);
          })
      );

      const { updateConfig } = useConfigStore.getState();
      const promise = updateConfig({ saveConnection: false });

      // 更新開始直後はローディング中
      expect(useConfigStore.getState().loading).toBe(true);

      await promise;

      // 完了後はローディング解除
      expect(useConfigStore.getState().loading).toBe(false);
    });

    it('nullフィールドを更新できる', async () => {
      useConfigStore.setState({ config: defaultConfig });

      mockInvoke.mockResolvedValue(undefined);

      const { updateConfig } = useConfigStore.getState();
      await updateConfig({ streamStyle: 'talk', platform: 'youtube' });

      const state = useConfigStore.getState();
      expect(state.config?.streamStyle).toBe('talk');
      expect(state.config?.platform).toBe('youtube');
    });
  });

  describe('clearError', () => {
    it('エラーメッセージをクリアできる', () => {
      useConfigStore.setState({ error: 'エラーメッセージ' });

      const { clearError } = useConfigStore.getState();
      clearError();

      const state = useConfigStore.getState();
      expect(state.error).toBeNull();
    });

    it('エラーがない状態でclearErrorを呼んでもエラーにならない', () => {
      const { clearError } = useConfigStore.getState();

      expect(() => clearError()).not.toThrow();

      const state = useConfigStore.getState();
      expect(state.error).toBeNull();
    });
  });

  describe('複合操作', () => {
    it('読み込み→更新→保存の一連の流れが正しく動作する', async () => {
      mockInvoke.mockResolvedValueOnce(mockConfig);

      const { loadConfig, updateConfig, saveConfig } = useConfigStore.getState();

      // 設定を読み込む
      await loadConfig();
      expect(useConfigStore.getState().config).toEqual(mockConfig);

      // 部分更新
      mockInvoke.mockResolvedValueOnce(undefined);
      await updateConfig({ streamingModeEnabled: true });
      expect(useConfigStore.getState().config?.streamingModeEnabled).toBe(true);

      // 全体を保存
      const newConfig = { ...mockConfig, streamingModeEnabled: true };
      mockInvoke.mockResolvedValueOnce(undefined);
      await saveConfig(newConfig);
      expect(useConfigStore.getState().config).toEqual(newConfig);
    });

    it('エラー発生後にclearErrorを呼んで再読み込みできる', async () => {
      // 最初はエラー
      mockInvoke.mockRejectedValueOnce(new Error('Initial error'));

      const { loadConfig, clearError } = useConfigStore.getState();
      await loadConfig();

      expect(useConfigStore.getState().error).toBe('Initial error');

      // エラーをクリア
      clearError();
      expect(useConfigStore.getState().error).toBeNull();

      // 再読み込み成功
      mockInvoke.mockResolvedValueOnce(mockConfig);
      await loadConfig();

      const state = useConfigStore.getState();
      expect(state.error).toBeNull();
      expect(state.config).toEqual(mockConfig);
    });

    it('複数回の部分更新が正しく適用される', async () => {
      useConfigStore.setState({ config: defaultConfig });

      mockInvoke.mockResolvedValue(undefined);

      const { updateConfig } = useConfigStore.getState();

      // 1回目の更新
      await updateConfig({ saveConnection: true });
      expect(useConfigStore.getState().config?.saveConnection).toBe(true);

      // 2回目の更新
      await updateConfig({ autoConnect: true });
      expect(useConfigStore.getState().config?.autoConnect).toBe(true);
      expect(useConfigStore.getState().config?.saveConnection).toBe(true);

      // 3回目の更新
      await updateConfig({ streamStyle: 'game' });
      expect(useConfigStore.getState().config?.streamStyle).toBe('game');
      expect(useConfigStore.getState().config?.saveConnection).toBe(true);
      expect(useConfigStore.getState().config?.autoConnect).toBe(true);
    });
  });

  describe('オンボーディングフロー', () => {
    it('オンボーディング完了フローが正しく動作する', async () => {
      // 初期状態: オンボーディング未完了
      mockInvoke.mockResolvedValueOnce(defaultConfig);

      const { loadConfig, updateConfig } = useConfigStore.getState();
      await loadConfig();

      expect(useConfigStore.getState().config?.onboardingCompleted).toBe(false);

      // オンボーディング完了を設定
      mockInvoke.mockResolvedValueOnce(undefined);
      await updateConfig({
        onboardingCompleted: true,
        streamStyle: 'game',
        platform: 'youtube',
        saveConnection: true,
        autoConnect: true,
      });

      const state = useConfigStore.getState();
      expect(state.config?.onboardingCompleted).toBe(true);
      expect(state.config?.streamStyle).toBe('game');
      expect(state.config?.platform).toBe('youtube');
    });
  });

  describe('全てのstreamStyleとplatformの組み合わせ', () => {
    it('全てのstreamStyleを設定できる', async () => {
      useConfigStore.setState({ config: defaultConfig });
      mockInvoke.mockResolvedValue(undefined);

      const styles = ['talk', 'game', 'music', 'art'] as const;
      const { updateConfig } = useConfigStore.getState();

      for (const style of styles) {
        await updateConfig({ streamStyle: style });
        expect(useConfigStore.getState().config?.streamStyle).toBe(style);
      }
    });

    it('全てのplatformを設定できる', async () => {
      useConfigStore.setState({ config: defaultConfig });
      mockInvoke.mockResolvedValue(undefined);

      const platforms = ['youtube', 'twitch', 'niconico', 'other'] as const;
      const { updateConfig } = useConfigStore.getState();

      for (const platform of platforms) {
        await updateConfig({ platform });
        expect(useConfigStore.getState().config?.platform).toBe(platform);
      }
    });
  });
});
