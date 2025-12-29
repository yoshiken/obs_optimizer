import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useHistoryStore } from './historyStore';
import { invoke } from '@tauri-apps/api/core';
import type { HistoricalMetrics, ObsStatusSnapshot, SessionSummary, SystemMetrics } from '../types/commands';

vi.mock('@tauri-apps/api/core');

const mockInvoke = vi.mocked(invoke);

// モックデータ
const mockSessionSummary: SessionSummary = {
  sessionId: 'session-1',
  startTime: Date.now() - 3600000, // 1時間前
  endTime: Date.now(),
  avgCpu: 45.5,
  avgGpu: 60.2,
  totalDroppedFrames: 120,
  peakBitrate: 8000,
  qualityScore: 85,
};

const mockSessionSummary2: SessionSummary = {
  sessionId: 'session-2',
  startTime: Date.now() - 7200000, // 2時間前
  endTime: Date.now() - 3600000, // 1時間前
  avgCpu: 50.0,
  avgGpu: 65.0,
  totalDroppedFrames: 200,
  peakBitrate: 7500,
  qualityScore: 80,
};

const mockSessions: SessionSummary[] = [mockSessionSummary, mockSessionSummary2];

const mockSystemMetrics: SystemMetrics = {
  cpu: {
    usagePercent: 45.5,
    coreCount: 8,
    perCoreUsage: [40, 50, 45, 42, 48, 43, 47, 44],
    cpuName: 'Intel(R) Core(TM) i7-9700K CPU @ 3.60GHz',
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

const mockObsStatusSnapshot: ObsStatusSnapshot = {
  streaming: true,
  recording: false,
  fps: 60.0,
  renderDroppedFrames: 10,
  outputDroppedFrames: 5,
  streamBitrate: 6000,
};

const mockHistoricalMetrics: HistoricalMetrics = {
  timestamp: Date.now(),
  sessionId: 'session-1',
  system: mockSystemMetrics,
  obs: mockObsStatusSnapshot,
};

const mockMetricsData: HistoricalMetrics[] = [
  mockHistoricalMetrics,
  {
    ...mockHistoricalMetrics,
    timestamp: Date.now() - 1000,
  },
];

describe('historyStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // ストアをリセット
    useHistoryStore.setState({
      sessions: [],
      selectedSessionIds: [],
      metricsData: [],
      isLoading: false,
      error: null,
    });
  });

  describe('初期状態', () => {
    it('空の状態で初期化される', () => {
      const state = useHistoryStore.getState();
      expect(state.sessions).toEqual([]);
      expect(state.selectedSessionIds).toEqual([]);
      expect(state.metricsData).toEqual([]);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();
    });
  });

  describe('loadSessions', () => {
    it('セッション一覧を取得できる', async () => {
      mockInvoke.mockResolvedValue(mockSessions);

      const { loadSessions } = useHistoryStore.getState();
      await loadSessions();

      const state = useHistoryStore.getState();
      expect(state.sessions).toEqual(mockSessions);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();

      expect(mockInvoke).toHaveBeenCalledWith('get_sessions');
    });

    it('読み込み中はローディング状態になる', async () => {
      mockInvoke.mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(() => resolve(mockSessions), 100);
          })
      );

      const { loadSessions } = useHistoryStore.getState();
      const promise = loadSessions();

      // 読み込み開始直後はローディング中
      expect(useHistoryStore.getState().isLoading).toBe(true);

      await promise;

      // 完了後はローディング解除
      expect(useHistoryStore.getState().isLoading).toBe(false);
    });

    it('エラー時にエラー状態を設定する', async () => {
      const errorMessage = 'Failed to load sessions';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      const { loadSessions } = useHistoryStore.getState();
      await loadSessions();

      const state = useHistoryStore.getState();
      expect(state.error).toBe(errorMessage);
      expect(state.isLoading).toBe(false);
    });

    it('エラーが文字列の場合もエラー処理できる', async () => {
      mockInvoke.mockRejectedValue('String error');

      const { loadSessions } = useHistoryStore.getState();
      await loadSessions();

      const state = useHistoryStore.getState();
      expect(state.error).toBe('セッション一覧の取得に失敗しました');
      expect(state.isLoading).toBe(false);
    });

    it('空のセッション一覧も処理できる', async () => {
      mockInvoke.mockResolvedValue([]);

      const { loadSessions } = useHistoryStore.getState();
      await loadSessions();

      const state = useHistoryStore.getState();
      expect(state.sessions).toEqual([]);
      expect(state.error).toBeNull();
    });
  });

  describe('selectSession', () => {
    it('セッションを選択できる', () => {
      const { selectSession } = useHistoryStore.getState();

      selectSession('session-1');

      const state = useHistoryStore.getState();
      expect(state.selectedSessionIds).toEqual(['session-1']);
    });

    it('複数のセッションを選択できる（最大2つ）', () => {
      const { selectSession } = useHistoryStore.getState();

      selectSession('session-1');
      selectSession('session-2');

      const state = useHistoryStore.getState();
      expect(state.selectedSessionIds).toEqual(['session-1', 'session-2']);
    });

    it('既に選択されているセッションは重複しない', () => {
      const { selectSession } = useHistoryStore.getState();

      selectSession('session-1');
      selectSession('session-1'); // 同じIDを再選択

      const state = useHistoryStore.getState();
      expect(state.selectedSessionIds).toEqual(['session-1']);
    });

    it('3つ目のセッションを選択すると最初のセッションが削除される', () => {
      const { selectSession } = useHistoryStore.getState();

      selectSession('session-1');
      selectSession('session-2');
      selectSession('session-3');

      const state = useHistoryStore.getState();
      expect(state.selectedSessionIds).toEqual(['session-2', 'session-3']);
      expect(state.selectedSessionIds).not.toContain('session-1');
    });

    it('4つ目以降も正しく置き換えられる', () => {
      const { selectSession } = useHistoryStore.getState();

      selectSession('session-1');
      selectSession('session-2');
      selectSession('session-3');
      selectSession('session-4');

      const state = useHistoryStore.getState();
      expect(state.selectedSessionIds).toEqual(['session-3', 'session-4']);
      expect(state.selectedSessionIds).toHaveLength(2);
    });
  });

  describe('deselectSession', () => {
    it('選択されているセッションを解除できる', () => {
      useHistoryStore.setState({ selectedSessionIds: ['session-1', 'session-2'] });

      const { deselectSession } = useHistoryStore.getState();
      deselectSession('session-1');

      const state = useHistoryStore.getState();
      expect(state.selectedSessionIds).toEqual(['session-2']);
    });

    it('複数選択されているセッションから特定のものを解除できる', () => {
      useHistoryStore.setState({ selectedSessionIds: ['session-1', 'session-2'] });

      const { deselectSession } = useHistoryStore.getState();
      deselectSession('session-2');

      const state = useHistoryStore.getState();
      expect(state.selectedSessionIds).toEqual(['session-1']);
    });

    it('存在しないIDを解除しても他のセッションに影響しない', () => {
      useHistoryStore.setState({ selectedSessionIds: ['session-1', 'session-2'] });

      const { deselectSession } = useHistoryStore.getState();
      deselectSession('non-existent');

      const state = useHistoryStore.getState();
      expect(state.selectedSessionIds).toEqual(['session-1', 'session-2']);
    });

    it('全てのセッションを解除できる', () => {
      useHistoryStore.setState({ selectedSessionIds: ['session-1', 'session-2'] });

      const { deselectSession } = useHistoryStore.getState();
      deselectSession('session-1');
      deselectSession('session-2');

      const state = useHistoryStore.getState();
      expect(state.selectedSessionIds).toEqual([]);
    });
  });

  describe('loadMetrics', () => {
    it('メトリクスデータを取得できる', async () => {
      mockInvoke.mockResolvedValue(mockMetricsData);

      const { loadMetrics } = useHistoryStore.getState();
      const from = Date.now() - 3600000;
      const to = Date.now();

      await loadMetrics('session-1', from, to);

      const state = useHistoryStore.getState();
      expect(state.metricsData).toEqual(mockMetricsData);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();

      expect(mockInvoke).toHaveBeenCalledWith('get_metrics_range', {
        sessionId: 'session-1',
        from,
        to,
      });
    });

    it('読み込み中はローディング状態になる', async () => {
      mockInvoke.mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(() => resolve(mockMetricsData), 100);
          })
      );

      const { loadMetrics } = useHistoryStore.getState();
      const promise = loadMetrics('session-1', Date.now() - 3600000, Date.now());

      // 読み込み開始直後はローディング中
      expect(useHistoryStore.getState().isLoading).toBe(true);

      await promise;

      // 完了後はローディング解除
      expect(useHistoryStore.getState().isLoading).toBe(false);
    });

    it('エラー時にエラー状態を設定する', async () => {
      const errorMessage = 'Failed to load metrics';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      const { loadMetrics } = useHistoryStore.getState();
      await loadMetrics('session-1', Date.now() - 3600000, Date.now());

      const state = useHistoryStore.getState();
      expect(state.error).toBe(errorMessage);
      expect(state.isLoading).toBe(false);
    });

    it('エラーが文字列の場合もエラー処理できる', async () => {
      mockInvoke.mockRejectedValue('String error');

      const { loadMetrics } = useHistoryStore.getState();
      await loadMetrics('session-1', Date.now() - 3600000, Date.now());

      const state = useHistoryStore.getState();
      expect(state.error).toBe('メトリクスデータの取得に失敗しました');
    });

    it('空のメトリクスデータも処理できる', async () => {
      mockInvoke.mockResolvedValue([]);

      const { loadMetrics } = useHistoryStore.getState();
      await loadMetrics('session-1', Date.now() - 3600000, Date.now());

      const state = useHistoryStore.getState();
      expect(state.metricsData).toEqual([]);
      expect(state.error).toBeNull();
    });

    it('異なるセッションのメトリクスを読み込むと上書きされる', async () => {
      mockInvoke.mockResolvedValueOnce(mockMetricsData);

      const { loadMetrics } = useHistoryStore.getState();
      await loadMetrics('session-1', Date.now() - 3600000, Date.now());

      expect(useHistoryStore.getState().metricsData).toHaveLength(2);

      const newMetrics: HistoricalMetrics[] = [mockHistoricalMetrics];
      mockInvoke.mockResolvedValueOnce(newMetrics);
      await loadMetrics('session-2', Date.now() - 3600000, Date.now());

      const state = useHistoryStore.getState();
      expect(state.metricsData).toEqual(newMetrics);
      expect(state.metricsData).toHaveLength(1);
    });
  });

  describe('clearSelection', () => {
    it('選択とメトリクスデータをクリアできる', () => {
      useHistoryStore.setState({
        selectedSessionIds: ['session-1', 'session-2'],
        metricsData: mockMetricsData,
      });

      const { clearSelection } = useHistoryStore.getState();
      clearSelection();

      const state = useHistoryStore.getState();
      expect(state.selectedSessionIds).toEqual([]);
      expect(state.metricsData).toEqual([]);
    });

    it('セッション一覧はクリアされない', () => {
      useHistoryStore.setState({
        sessions: mockSessions,
        selectedSessionIds: ['session-1'],
        metricsData: mockMetricsData,
      });

      const { clearSelection } = useHistoryStore.getState();
      clearSelection();

      const state = useHistoryStore.getState();
      expect(state.sessions).toEqual(mockSessions);
      expect(state.selectedSessionIds).toEqual([]);
      expect(state.metricsData).toEqual([]);
    });

    it('空の状態でclearSelectionを呼んでもエラーにならない', () => {
      const { clearSelection } = useHistoryStore.getState();

      expect(() => clearSelection()).not.toThrow();

      const state = useHistoryStore.getState();
      expect(state.selectedSessionIds).toEqual([]);
      expect(state.metricsData).toEqual([]);
    });
  });

  describe('clearError', () => {
    it('エラーメッセージをクリアできる', () => {
      useHistoryStore.setState({ error: 'エラーメッセージ' });

      const { clearError } = useHistoryStore.getState();
      clearError();

      const state = useHistoryStore.getState();
      expect(state.error).toBeNull();
    });

    it('エラーがない状態でclearErrorを呼んでもエラーにならない', () => {
      const { clearError } = useHistoryStore.getState();

      expect(() => clearError()).not.toThrow();

      const state = useHistoryStore.getState();
      expect(state.error).toBeNull();
    });
  });

  describe('複合操作', () => {
    it('セッション読み込み→選択→メトリクス読み込みの一連の流れが正しく動作する', async () => {
      mockInvoke.mockResolvedValueOnce(mockSessions);

      const { loadSessions, selectSession, loadMetrics } = useHistoryStore.getState();

      // セッション一覧を読み込む
      await loadSessions();
      expect(useHistoryStore.getState().sessions).toHaveLength(2);

      // セッションを選択
      selectSession('session-1');
      expect(useHistoryStore.getState().selectedSessionIds).toEqual(['session-1']);

      // メトリクスを読み込む
      mockInvoke.mockResolvedValueOnce(mockMetricsData);
      await loadMetrics('session-1', Date.now() - 3600000, Date.now());
      expect(useHistoryStore.getState().metricsData).toHaveLength(2);
    });

    it('複数セッションの選択と解除が正しく動作する', () => {
      const { selectSession, deselectSession } = useHistoryStore.getState();

      selectSession('session-1');
      selectSession('session-2');
      expect(useHistoryStore.getState().selectedSessionIds).toEqual(['session-1', 'session-2']);

      deselectSession('session-1');
      expect(useHistoryStore.getState().selectedSessionIds).toEqual(['session-2']);

      selectSession('session-3');
      expect(useHistoryStore.getState().selectedSessionIds).toEqual(['session-2', 'session-3']);

      deselectSession('session-2');
      deselectSession('session-3');
      expect(useHistoryStore.getState().selectedSessionIds).toEqual([]);
    });

    it('エラー発生後にclearErrorを呼んで再読み込みできる', async () => {
      // 最初はエラー
      mockInvoke.mockRejectedValueOnce(new Error('Initial error'));

      const { loadSessions, clearError } = useHistoryStore.getState();
      await loadSessions();

      expect(useHistoryStore.getState().error).toBe('Initial error');

      // エラーをクリア
      clearError();
      expect(useHistoryStore.getState().error).toBeNull();

      // 再読み込み成功
      mockInvoke.mockResolvedValueOnce(mockSessions);
      await loadSessions();

      const state = useHistoryStore.getState();
      expect(state.error).toBeNull();
      expect(state.sessions).toEqual(mockSessions);
    });

    it('clearSelectionはエラー状態に影響しない', () => {
      useHistoryStore.setState({
        selectedSessionIds: ['session-1'],
        error: 'エラーメッセージ',
      });

      const { clearSelection } = useHistoryStore.getState();
      clearSelection();

      const state = useHistoryStore.getState();
      expect(state.error).toBe('エラーメッセージ');
      expect(state.selectedSessionIds).toEqual([]);
    });
  });

  describe('セッション比較機能', () => {
    it('2つのセッションを選択して比較用のデータを保持できる', async () => {
      mockInvoke.mockResolvedValueOnce(mockSessions);

      const { loadSessions, selectSession, loadMetrics } = useHistoryStore.getState();

      // セッション一覧を読み込む
      await loadSessions();

      // 2つのセッションを選択
      selectSession('session-1');
      selectSession('session-2');

      expect(useHistoryStore.getState().selectedSessionIds).toEqual(['session-1', 'session-2']);

      // 各セッションのメトリクスを読み込む（実際のUIでは個別に呼ぶ）
      mockInvoke.mockResolvedValueOnce(mockMetricsData);
      await loadMetrics('session-1', Date.now() - 3600000, Date.now());

      expect(useHistoryStore.getState().metricsData).toHaveLength(2);
    });
  });
});
