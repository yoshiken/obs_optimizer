import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useStreamingModeStore } from './streamingModeStore';
import { invoke } from '@tauri-apps/api/core';
import { type Event, listen } from '@tauri-apps/api/event';
import type { StreamingChangedPayload } from '../types/commands';
import { OBS_EVENTS } from '../types/commands';

vi.mock('@tauri-apps/api/core');
vi.mock('@tauri-apps/api/event');

const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

describe('streamingModeStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // ストアをリセット
    useStreamingModeStore.setState({
      isEnabled: false,
      isLoading: false,
      error: null,
    });
  });

  describe('初期状態', () => {
    it('配信中モードが無効で初期化される', () => {
      const state = useStreamingModeStore.getState();
      expect(state.isEnabled).toBe(false);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();
    });
  });

  describe('setEnabled', () => {
    it('配信中モードを有効化できる', async () => {
      mockInvoke.mockResolvedValue(undefined);

      const { setEnabled } = useStreamingModeStore.getState();
      await setEnabled(true);

      const state = useStreamingModeStore.getState();
      expect(state.isEnabled).toBe(true);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();

      expect(mockInvoke).toHaveBeenCalledWith('set_streaming_mode', { enabled: true });
    });

    it('配信中モードを無効化できる', async () => {
      useStreamingModeStore.setState({ isEnabled: true });

      mockInvoke.mockResolvedValue(undefined);

      const { setEnabled } = useStreamingModeStore.getState();
      await setEnabled(false);

      const state = useStreamingModeStore.getState();
      expect(state.isEnabled).toBe(false);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();

      expect(mockInvoke).toHaveBeenCalledWith('set_streaming_mode', { enabled: false });
    });

    it('設定中はローディング状態になる', async () => {
      mockInvoke.mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(() => resolve(undefined), 100);
          })
      );

      const { setEnabled } = useStreamingModeStore.getState();
      const promise = setEnabled(true);

      // 設定開始直後はローディング中
      expect(useStreamingModeStore.getState().isLoading).toBe(true);

      await promise;

      // 完了後はローディング解除
      expect(useStreamingModeStore.getState().isLoading).toBe(false);
    });

    it('エラー時にエラーを投げる', async () => {
      const errorMessage = 'Failed to set streaming mode';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      const { setEnabled } = useStreamingModeStore.getState();

      await expect(setEnabled(true)).rejects.toThrow(errorMessage);

      const state = useStreamingModeStore.getState();
      expect(state.error).toBe(errorMessage);
      expect(state.isLoading).toBe(false);
    });

    it('エラーが文字列の場合も処理できる', async () => {
      mockInvoke.mockRejectedValue('String error');

      const { setEnabled } = useStreamingModeStore.getState();

      await expect(setEnabled(true)).rejects.toBe('String error');

      const state = useStreamingModeStore.getState();
      expect(state.error).toBe('String error');
    });

    it('連続してトグルできる', async () => {
      mockInvoke.mockResolvedValue(undefined);

      const { setEnabled } = useStreamingModeStore.getState();

      await setEnabled(true);
      expect(useStreamingModeStore.getState().isEnabled).toBe(true);

      await setEnabled(false);
      expect(useStreamingModeStore.getState().isEnabled).toBe(false);

      await setEnabled(true);
      expect(useStreamingModeStore.getState().isEnabled).toBe(true);
    });
  });

  describe('loadStreamingMode', () => {
    it('配信中モード状態を読み込める（有効）', async () => {
      mockInvoke.mockResolvedValue(true);

      const { loadStreamingMode } = useStreamingModeStore.getState();
      await loadStreamingMode();

      const state = useStreamingModeStore.getState();
      expect(state.isEnabled).toBe(true);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();

      expect(mockInvoke).toHaveBeenCalledWith('get_streaming_mode');
    });

    it('配信中モード状態を読み込める（無効）', async () => {
      mockInvoke.mockResolvedValue(false);

      const { loadStreamingMode } = useStreamingModeStore.getState();
      await loadStreamingMode();

      const state = useStreamingModeStore.getState();
      expect(state.isEnabled).toBe(false);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();
    });

    it('読み込み中はローディング状態になる', async () => {
      mockInvoke.mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(() => resolve(true), 100);
          })
      );

      const { loadStreamingMode } = useStreamingModeStore.getState();
      const promise = loadStreamingMode();

      // 読み込み開始直後はローディング中
      expect(useStreamingModeStore.getState().isLoading).toBe(true);

      await promise;

      // 完了後はローディング解除
      expect(useStreamingModeStore.getState().isLoading).toBe(false);
    });

    it('エラー時にエラー状態を設定する（例外は投げない）', async () => {
      const errorMessage = 'Failed to load streaming mode';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      const { loadStreamingMode } = useStreamingModeStore.getState();
      await loadStreamingMode();

      const state = useStreamingModeStore.getState();
      expect(state.error).toBe(errorMessage);
      expect(state.isLoading).toBe(false);
    });

    it('エラーが文字列の場合も処理できる', async () => {
      mockInvoke.mockRejectedValue('String error');

      const { loadStreamingMode } = useStreamingModeStore.getState();
      await loadStreamingMode();

      const state = useStreamingModeStore.getState();
      expect(state.error).toBe('String error');
    });
  });

  describe('initializeAutoMode', () => {
    it('OBS配信開始イベントのリスナーを登録する', () => {
      type EventHandler = (event: Event<StreamingChangedPayload>) => void;
      let eventHandler: EventHandler | null = null;

      mockListen.mockImplementation((eventName: string, handler: EventHandler) => {
        if (eventName === OBS_EVENTS.STREAMING_CHANGED) {
          eventHandler = handler;
        }
        return Promise.resolve(() => {});
      });

      const { initializeAutoMode } = useStreamingModeStore.getState();
      initializeAutoMode();

      expect(mockListen).toHaveBeenCalledWith(
        OBS_EVENTS.STREAMING_CHANGED,
        expect.any(Function)
      );
      expect(eventHandler).not.toBeNull();
    });

    it('配信開始イベントで配信中モードを自動的にONにする', async () => {
      type EventHandler = (event: Event<StreamingChangedPayload>) => void;
      let eventHandler: EventHandler | null = null;

      mockListen.mockImplementation((eventName: string, handler: EventHandler) => {
        if (eventName === OBS_EVENTS.STREAMING_CHANGED) {
          eventHandler = handler;
        }
        return Promise.resolve(() => {});
      });

      mockInvoke.mockResolvedValue(undefined);

      const { initializeAutoMode } = useStreamingModeStore.getState();
      initializeAutoMode();

      // 配信開始イベントをシミュレート
      eventHandler!({
        event: OBS_EVENTS.STREAMING_CHANGED,
        id: 1,
        payload: {
          isStreaming: true,
          startedAt: Date.now(),
        },
      });

      // invokeの完了を待つ
      await vi.waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('set_streaming_mode', { enabled: true });
      });

      // 状態が更新されるまで待つ
      await vi.waitFor(() => {
        expect(useStreamingModeStore.getState().isEnabled).toBe(true);
      });
    });

    it('配信終了イベントで配信中モードを自動的にOFFにする', async () => {
      type EventHandler = (event: Event<StreamingChangedPayload>) => void;
      let eventHandler: EventHandler | null = null;

      mockListen.mockImplementation((eventName: string, handler: EventHandler) => {
        if (eventName === OBS_EVENTS.STREAMING_CHANGED) {
          eventHandler = handler;
        }
        return Promise.resolve(() => {});
      });

      mockInvoke.mockResolvedValue(undefined);

      // 初期状態を配信中モードONに設定
      useStreamingModeStore.setState({ isEnabled: true });

      const { initializeAutoMode } = useStreamingModeStore.getState();
      initializeAutoMode();

      // 配信終了イベントをシミュレート
      eventHandler!({
        event: OBS_EVENTS.STREAMING_CHANGED,
        id: 2,
        payload: {
          isStreaming: false,
          startedAt: null,
        },
      });

      // invokeの完了を待つ
      await vi.waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('set_streaming_mode', { enabled: false });
      });

      // 状態が更新されるまで待つ
      await vi.waitFor(() => {
        expect(useStreamingModeStore.getState().isEnabled).toBe(false);
      });
    });

    it('イベントリスナー登録エラー時にエラー状態を設定する', async () => {
      const errorMessage = 'Failed to register listener';
      mockListen.mockRejectedValue(new Error(errorMessage));

      const { initializeAutoMode } = useStreamingModeStore.getState();
      initializeAutoMode();

      // エラー処理を待つ
      await vi.waitFor(() => {
        const state = useStreamingModeStore.getState();
        expect(state.error).toContain('イベントリスナーの初期化に失敗');
      });
    });

    it('配信中モード設定エラー時にエラー状態を設定する', async () => {
      type EventHandler = (event: Event<StreamingChangedPayload>) => void;
      let eventHandler: EventHandler | null = null;

      mockListen.mockImplementation((eventName: string, handler: EventHandler) => {
        if (eventName === OBS_EVENTS.STREAMING_CHANGED) {
          eventHandler = handler;
        }
        return Promise.resolve(() => {});
      });

      const errorMessage = 'Failed to set streaming mode';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      const { initializeAutoMode } = useStreamingModeStore.getState();
      initializeAutoMode();

      // 配信開始イベントをシミュレート
      eventHandler!({
        event: OBS_EVENTS.STREAMING_CHANGED,
        id: 3,
        payload: {
          isStreaming: true,
          startedAt: Date.now(),
        },
      });

      // エラー処理を待つ
      await vi.waitFor(() => {
        const state = useStreamingModeStore.getState();
        expect(state.error).toBe(errorMessage);
      });
    });
  });

  describe('clearError', () => {
    it('エラーメッセージをクリアできる', () => {
      useStreamingModeStore.setState({ error: 'エラーメッセージ' });

      const { clearError } = useStreamingModeStore.getState();
      clearError();

      const state = useStreamingModeStore.getState();
      expect(state.error).toBeNull();
    });

    it('エラーがない状態でclearErrorを呼んでもエラーにならない', () => {
      const { clearError } = useStreamingModeStore.getState();

      expect(() => clearError()).not.toThrow();

      const state = useStreamingModeStore.getState();
      expect(state.error).toBeNull();
    });
  });

  describe('複合操作', () => {
    it('読み込み→設定→クリアの一連の流れが正しく動作する', async () => {
      mockInvoke.mockResolvedValueOnce(false);

      const { loadStreamingMode, setEnabled } = useStreamingModeStore.getState();

      // 状態を読み込む
      await loadStreamingMode();
      expect(useStreamingModeStore.getState().isEnabled).toBe(false);

      // 配信中モードを有効化
      mockInvoke.mockResolvedValueOnce(undefined);
      await setEnabled(true);
      expect(useStreamingModeStore.getState().isEnabled).toBe(true);

      // 無効化
      mockInvoke.mockResolvedValueOnce(undefined);
      await setEnabled(false);
      expect(useStreamingModeStore.getState().isEnabled).toBe(false);
    });

    it('エラー発生後にclearErrorを呼んで再実行できる', async () => {
      // 最初はエラー
      mockInvoke.mockRejectedValueOnce(new Error('Initial error'));

      const { setEnabled, clearError } = useStreamingModeStore.getState();

      await expect(setEnabled(true)).rejects.toThrow('Initial error');
      expect(useStreamingModeStore.getState().error).toBe('Initial error');

      // エラーをクリア
      clearError();
      expect(useStreamingModeStore.getState().error).toBeNull();

      // 再実行成功
      mockInvoke.mockResolvedValueOnce(undefined);
      await setEnabled(true);

      const state = useStreamingModeStore.getState();
      expect(state.error).toBeNull();
      expect(state.isEnabled).toBe(true);
    });

    it('自動モード初期化と手動設定が併用できる', async () => {
      type EventHandler = (event: Event<StreamingChangedPayload>) => void;
      let eventHandler: EventHandler | null = null;

      mockListen.mockImplementation((eventName: string, handler: EventHandler) => {
        if (eventName === OBS_EVENTS.STREAMING_CHANGED) {
          eventHandler = handler;
        }
        return Promise.resolve(() => {});
      });

      mockInvoke.mockResolvedValue(undefined);

      const { initializeAutoMode, setEnabled } = useStreamingModeStore.getState();

      // 自動モードを初期化
      initializeAutoMode();

      // 手動で有効化
      await setEnabled(true);
      expect(useStreamingModeStore.getState().isEnabled).toBe(true);

      // 配信終了イベントで自動的にOFF
      eventHandler!({
        event: OBS_EVENTS.STREAMING_CHANGED,
        id: 4,
        payload: {
          isStreaming: false,
          startedAt: null,
        },
      });

      await vi.waitFor(() => {
        expect(useStreamingModeStore.getState().isEnabled).toBe(false);
      });
    });
  });

  describe('配信ライフサイクル', () => {
    it('配信開始→終了の一連のイベントフローが正しく動作する', async () => {
      type EventHandler = (event: Event<StreamingChangedPayload>) => void;
      let eventHandler: EventHandler | null = null;

      mockListen.mockImplementation((eventName: string, handler: EventHandler) => {
        if (eventName === OBS_EVENTS.STREAMING_CHANGED) {
          eventHandler = handler;
        }
        return Promise.resolve(() => {});
      });

      mockInvoke.mockResolvedValue(undefined);

      const { initializeAutoMode } = useStreamingModeStore.getState();
      initializeAutoMode();

      // 配信開始
      eventHandler!({
        event: OBS_EVENTS.STREAMING_CHANGED,
        id: 5,
        payload: {
          isStreaming: true,
          startedAt: Date.now(),
        },
      });

      await vi.waitFor(() => {
        expect(useStreamingModeStore.getState().isEnabled).toBe(true);
      });

      // 配信終了
      eventHandler!({
        event: OBS_EVENTS.STREAMING_CHANGED,
        id: 6,
        payload: {
          isStreaming: false,
          startedAt: null,
        },
      });

      await vi.waitFor(() => {
        expect(useStreamingModeStore.getState().isEnabled).toBe(false);
      });
    });
  });
});
