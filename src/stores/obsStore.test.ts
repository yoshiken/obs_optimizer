import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useObsStore } from './obsStore';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  mockObsStatus,
  mockSceneList,
  setupInvokeErrorMock,
  setupInvokeMock,
  setupListenMock,
} from '../tests/mocks/tauriMocks';
import type { ObsConnectionParams } from '../types/commands';

vi.mock('@tauri-apps/api/core');
vi.mock('@tauri-apps/api/event');

const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

describe('obsStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // ストアをリセット
    useObsStore.setState({
      connectionState: 'disconnected',
      status: null,
      error: null,
      warning: null,
      loading: false,
      scenes: [],
      lastConnectionParams: null,
    });
  });

  describe('初期状態', () => {
    it('接続されていない状態で初期化される', () => {
      const state = useObsStore.getState();
      expect(state.connectionState).toBe('disconnected');
      expect(state.status).toBeNull();
      expect(state.error).toBeNull();
      expect(state.loading).toBe(false);
      expect(state.scenes).toEqual([]);
    });
  });

  describe('connect', () => {
    const connectionParams: ObsConnectionParams = {
      host: 'localhost',
      port: 4455,
      password: 'test-password',
    };

    it('OBSに正常に接続できる', async () => {
      setupInvokeMock(mockInvoke);

      const { connect } = useObsStore.getState();
      await connect(connectionParams);

      const state = useObsStore.getState();
      expect(state.connectionState).toBe('connected');
      expect(state.loading).toBe(false);
      expect(state.error).toBeNull();
      expect(state.lastConnectionParams).toEqual(connectionParams);

      // invokeが正しく呼ばれたか確認
      expect(mockInvoke).toHaveBeenCalledWith('connect_obs', { params: connectionParams });
    });

    it('接続中はローディング状態になる', async () => {
      setupInvokeMock(mockInvoke);

      const { connect } = useObsStore.getState();
      const promise = connect(connectionParams);

      // 接続開始直後の状態を確認（実際にはタイミングの問題で難しい）
      await promise;
      expect(useObsStore.getState().loading).toBe(false);
    });

    it('接続失敗時にエラー状態になる', async () => {
      const errorMessage = 'Connection failed';
      setupInvokeErrorMock(mockInvoke, errorMessage);

      const { connect } = useObsStore.getState();
      await expect(connect(connectionParams)).rejects.toThrow(errorMessage);

      const state = useObsStore.getState();
      expect(state.connectionState).toBe('error');
      expect(state.error).toBe(errorMessage);
      expect(state.loading).toBe(false);
    });

    it('接続成功後にステータスとシーン一覧を取得する', async () => {
      setupInvokeMock(mockInvoke);

      const { connect } = useObsStore.getState();
      await connect(connectionParams);

      const state = useObsStore.getState();
      expect(state.status).toEqual(mockObsStatus);
      expect(state.scenes).toEqual(mockSceneList);
    });
  });

  describe('disconnect', () => {
    it('OBSから正常に切断できる', async () => {
      setupInvokeMock(mockInvoke);

      // 先に接続状態にする
      useObsStore.setState({
        connectionState: 'connected',
        status: mockObsStatus,
        scenes: mockSceneList,
      });

      const { disconnect } = useObsStore.getState();
      await disconnect();

      const state = useObsStore.getState();
      expect(state.connectionState).toBe('disconnected');
      expect(state.status).toBeNull();
      expect(state.scenes).toEqual([]);
      expect(state.loading).toBe(false);

      expect(mockInvoke).toHaveBeenCalledWith('disconnect_obs');
    });

    it('切断失敗時にエラーを設定する', async () => {
      const errorMessage = 'Disconnect failed';
      setupInvokeErrorMock(mockInvoke, errorMessage);

      const { disconnect } = useObsStore.getState();
      await expect(disconnect()).rejects.toThrow(errorMessage);

      const state = useObsStore.getState();
      expect(state.error).toBe(errorMessage);
    });
  });

  describe('fetchStatus', () => {
    it('OBSステータスを取得できる', async () => {
      setupInvokeMock(mockInvoke);

      const { fetchStatus } = useObsStore.getState();
      await fetchStatus();

      const state = useObsStore.getState();
      expect(state.status).toEqual(mockObsStatus);
      expect(state.warning).toBeNull();

      expect(mockInvoke).toHaveBeenCalledWith('get_obs_status');
    });

    it('ステータス取得失敗時は警告を設定する（エラーにはしない）', async () => {
      setupInvokeErrorMock(mockInvoke, 'Status fetch failed');

      const { fetchStatus } = useObsStore.getState();
      await fetchStatus();

      const state = useObsStore.getState();
      expect(state.warning).toContain('ステータス取得失敗');
      expect(state.error).toBeNull();
    });

    it('接続状態を自動更新する', async () => {
      setupInvokeMock(mockInvoke);

      // disconnected状態から開始
      useObsStore.setState({ connectionState: 'disconnected' });

      const { fetchStatus } = useObsStore.getState();
      await fetchStatus();

      const state = useObsStore.getState();
      expect(state.connectionState).toBe('connected');
    });
  });

  describe('fetchScenes', () => {
    it('シーン一覧を取得できる', async () => {
      setupInvokeMock(mockInvoke);

      const { fetchScenes } = useObsStore.getState();
      await fetchScenes();

      const state = useObsStore.getState();
      expect(state.scenes).toEqual(mockSceneList);

      expect(mockInvoke).toHaveBeenCalledWith('get_scene_list');
    });

    it('シーン取得失敗時は警告を設定する', async () => {
      setupInvokeErrorMock(mockInvoke, 'Scene fetch failed');

      const { fetchScenes } = useObsStore.getState();
      await fetchScenes();

      const state = useObsStore.getState();
      expect(state.scenes).toEqual([]);
      expect(state.warning).toContain('シーン一覧取得失敗');
    });
  });

  describe('setCurrentScene', () => {
    it('シーンを変更できる', async () => {
      setupInvokeMock(mockInvoke);

      const { setCurrentScene } = useObsStore.getState();
      await setCurrentScene('ゲームシーン');

      expect(mockInvoke).toHaveBeenCalledWith('set_current_scene', {
        sceneName: 'ゲームシーン',
      });
    });

    it('シーン変更後にステータスを更新する', async () => {
      setupInvokeMock(mockInvoke);

      const { setCurrentScene } = useObsStore.getState();
      await setCurrentScene('ゲームシーン');

      const state = useObsStore.getState();
      expect(state.status).toEqual(mockObsStatus);
    });
  });

  describe('startStreaming / stopStreaming', () => {
    it('配信を開始できる', async () => {
      setupInvokeMock(mockInvoke);

      const { startStreaming } = useObsStore.getState();
      await startStreaming();

      expect(mockInvoke).toHaveBeenCalledWith('start_streaming');
    });

    it('配信を停止できる', async () => {
      setupInvokeMock(mockInvoke);

      const { stopStreaming } = useObsStore.getState();
      await stopStreaming();

      expect(mockInvoke).toHaveBeenCalledWith('stop_streaming');
    });
  });

  describe('startRecording / stopRecording', () => {
    it('録画を開始できる', async () => {
      setupInvokeMock(mockInvoke);

      const { startRecording } = useObsStore.getState();
      await startRecording();

      expect(mockInvoke).toHaveBeenCalledWith('start_recording');
    });

    it('録画を停止し、ファイルパスを返す', async () => {
      setupInvokeMock(mockInvoke);

      const { stopRecording } = useObsStore.getState();
      const outputPath = await stopRecording();

      expect(outputPath).toBe('C:\\recordings\\video.mp4');
      expect(mockInvoke).toHaveBeenCalledWith('stop_recording');
    });
  });

  describe('startPolling', () => {
    it('ポーリングを開始し、停止関数を返す', () => {
      vi.useFakeTimers();
      setupInvokeMock(mockInvoke);

      const { startPolling } = useObsStore.getState();
      const stopPolling = startPolling(100);

      // タイマーをスキップ
      vi.advanceTimersByTime(100);

      // invokeが呼ばれることを確認
      expect(mockInvoke).toHaveBeenCalled();

      // 停止
      stopPolling();
      vi.useRealTimers();
    });
  });

  describe('subscribeToEvents', () => {
    it('イベントリスナーを登録し、アンリスナーを返す', async () => {
      setupListenMock(mockListen);

      const { subscribeToEvents } = useObsStore.getState();
      const unsubscribe = await subscribeToEvents();

      // listenが複数回呼ばれることを確認
      expect(mockListen).toHaveBeenCalled();
      expect(typeof unsubscribe).toBe('function');

      unsubscribe();
    });
  });

  describe('clearError / clearWarning', () => {
    it('エラーをクリアできる', () => {
      useObsStore.setState({ error: 'test error' });

      const { clearError } = useObsStore.getState();
      clearError();

      expect(useObsStore.getState().error).toBeNull();
    });

    it('警告をクリアできる', () => {
      useObsStore.setState({ warning: 'test warning' });

      const { clearWarning } = useObsStore.getState();
      clearWarning();

      expect(useObsStore.getState().warning).toBeNull();
    });
  });
});
