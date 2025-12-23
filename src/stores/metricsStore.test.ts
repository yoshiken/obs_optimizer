import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useMetricsStore } from './metricsStore';
import { invoke } from '@tauri-apps/api/core';
import {
  mockSystemMetrics,
  mockObsProcessMetrics,
  setupInvokeMock,
  setupInvokeErrorMock,
} from '../tests/mocks/tauriMocks';

vi.mock('@tauri-apps/api/core');

const mockInvoke = vi.mocked(invoke);

describe('metricsStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // ストアをリセット
    useMetricsStore.setState({
      metrics: null,
      obsProcessMetrics: null,
      history: {
        cpuUsage: [],
        memoryUsage: [],
        gpuUsage: [],
        networkUpload: [],
        networkDownload: [],
      },
      loading: false,
      error: null,
      lastUpdate: null,
    });
  });

  describe('初期状態', () => {
    it('メトリクスがnullで初期化される', () => {
      const state = useMetricsStore.getState();
      expect(state.metrics).toBeNull();
      expect(state.obsProcessMetrics).toBeNull();
      expect(state.loading).toBe(false);
      expect(state.error).toBeNull();
      expect(state.lastUpdate).toBeNull();
    });

    it('履歴が空配列で初期化される', () => {
      const state = useMetricsStore.getState();
      expect(state.history.cpuUsage).toEqual([]);
      expect(state.history.memoryUsage).toEqual([]);
      expect(state.history.gpuUsage).toEqual([]);
      expect(state.history.networkUpload).toEqual([]);
      expect(state.history.networkDownload).toEqual([]);
    });
  });

  describe('fetchMetrics', () => {
    it('システムメトリクスを取得できる', async () => {
      setupInvokeMock(mockInvoke);

      const { fetchMetrics } = useMetricsStore.getState();
      await fetchMetrics();

      const state = useMetricsStore.getState();
      expect(state.metrics).toEqual(mockSystemMetrics);
      expect(state.loading).toBe(false);
      expect(state.error).toBeNull();
      expect(state.lastUpdate).toBeGreaterThan(0);

      expect(mockInvoke).toHaveBeenCalledWith('get_system_metrics');
    });

    it('履歴にデータポイントを追加する', async () => {
      setupInvokeMock(mockInvoke);

      const { fetchMetrics } = useMetricsStore.getState();
      await fetchMetrics();

      const state = useMetricsStore.getState();
      expect(state.history.cpuUsage.length).toBe(1);
      expect(state.history.cpuUsage[0].value).toBe(mockSystemMetrics.cpu.usagePercent);
      expect(state.history.memoryUsage.length).toBe(1);
      expect(state.history.memoryUsage[0].value).toBe(mockSystemMetrics.memory.usagePercent);
    });

    it('GPUが存在する場合は履歴に追加する', async () => {
      setupInvokeMock(mockInvoke);

      const { fetchMetrics } = useMetricsStore.getState();
      await fetchMetrics();

      const state = useMetricsStore.getState();
      expect(state.history.gpuUsage.length).toBe(1);
      expect(state.history.gpuUsage[0].value).toBe(mockSystemMetrics.gpu!.usagePercent);
    });

    it('GPUが存在しない場合は履歴をクリアする', async () => {
      // GPU無しのモックデータ
      mockInvoke.mockResolvedValue({
        ...mockSystemMetrics,
        gpu: null,
      });

      const { fetchMetrics } = useMetricsStore.getState();
      await fetchMetrics();

      const state = useMetricsStore.getState();
      expect(state.history.gpuUsage).toEqual([]);
    });

    it('複数回呼び出すと履歴が蓄積される', async () => {
      setupInvokeMock(mockInvoke);

      const { fetchMetrics } = useMetricsStore.getState();
      await fetchMetrics();
      await fetchMetrics();
      await fetchMetrics();

      const state = useMetricsStore.getState();
      expect(state.history.cpuUsage.length).toBe(3);
      expect(state.history.memoryUsage.length).toBe(3);
    });

    it('エラー時にエラー状態を設定する', async () => {
      const errorMessage = 'Metrics fetch failed';
      setupInvokeErrorMock(mockInvoke, errorMessage);

      const { fetchMetrics } = useMetricsStore.getState();
      await fetchMetrics();

      const state = useMetricsStore.getState();
      expect(state.error).toBe(errorMessage);
      expect(state.loading).toBe(false);
    });
  });

  describe('fetchObsProcessMetrics', () => {
    it('OBSプロセスメトリクスを取得できる', async () => {
      setupInvokeMock(mockInvoke);

      const { fetchObsProcessMetrics } = useMetricsStore.getState();
      await fetchObsProcessMetrics();

      const state = useMetricsStore.getState();
      expect(state.obsProcessMetrics).toEqual(mockObsProcessMetrics);

      expect(mockInvoke).toHaveBeenCalledWith('get_process_metrics');
    });

    it('エラー時はnullを設定する（エラーとして扱わない）', async () => {
      setupInvokeErrorMock(mockInvoke, 'Process metrics failed');

      const { fetchObsProcessMetrics } = useMetricsStore.getState();
      await fetchObsProcessMetrics();

      const state = useMetricsStore.getState();
      expect(state.obsProcessMetrics).toBeNull();
      expect(state.error).toBeNull();
    });
  });

  describe('startPolling', () => {
    it('ポーリングを開始し、停止関数を返す', () => {
      vi.useFakeTimers();
      setupInvokeMock(mockInvoke);

      const { startPolling } = useMetricsStore.getState();
      const stopPolling = startPolling(100);

      // 初回取得を確認
      expect(mockInvoke).toHaveBeenCalled();

      // タイマーをスキップして定期取得を確認
      vi.advanceTimersByTime(100);
      expect(mockInvoke).toHaveBeenCalled();

      // 停止
      stopPolling();
      vi.useRealTimers();
    });

    it('システムメトリクスとOBSプロセスメトリクスを異なる間隔で取得する', () => {
      vi.useFakeTimers();
      setupInvokeMock(mockInvoke);

      const { startPolling } = useMetricsStore.getState();
      const stopPolling = startPolling(1000);

      mockInvoke.mockClear();

      // 1秒後: システムメトリクスのみ
      vi.advanceTimersByTime(1000);
      const calls1 = mockInvoke.mock.calls.filter(
        (call) => call[0] === 'get_system_metrics'
      );
      expect(calls1.length).toBeGreaterThan(0);

      mockInvoke.mockClear();

      // 2秒後: OBSプロセスメトリクスも取得
      vi.advanceTimersByTime(1000);
      const calls2 = mockInvoke.mock.calls.filter(
        (call) => call[0] === 'get_process_metrics'
      );
      expect(calls2.length).toBeGreaterThan(0);

      stopPolling();
      vi.useRealTimers();
    });
  });

  describe('clearHistory', () => {
    it('履歴をクリアできる', async () => {
      setupInvokeMock(mockInvoke);

      const { fetchMetrics, clearHistory } = useMetricsStore.getState();
      await fetchMetrics();
      await fetchMetrics();

      // 履歴があることを確認
      expect(useMetricsStore.getState().history.cpuUsage.length).toBeGreaterThan(0);

      clearHistory();

      const state = useMetricsStore.getState();
      expect(state.history.cpuUsage).toEqual([]);
      expect(state.history.memoryUsage).toEqual([]);
      expect(state.history.gpuUsage).toEqual([]);
      expect(state.history.networkUpload).toEqual([]);
      expect(state.history.networkDownload).toEqual([]);
    });
  });

  describe('履歴の最大ポイント数', () => {
    it('60ポイントを超えると古いデータを削除する', async () => {
      setupInvokeMock(mockInvoke);

      const { fetchMetrics } = useMetricsStore.getState();

      // 70回データを取得
      for (let i = 0; i < 70; i++) {
        await fetchMetrics();
      }

      const state = useMetricsStore.getState();
      expect(state.history.cpuUsage.length).toBe(60);
      expect(state.history.memoryUsage.length).toBe(60);
    });
  });
});
