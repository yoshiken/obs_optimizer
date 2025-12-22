import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { SystemMetrics, ObsProcessMetrics } from '../types';

// ========================================
// ヒストリー用の型定義
// ========================================

/** 時系列データポイント */
export interface TimeSeriesDataPoint {
  timestamp: number;
  value: number;
}

/** メトリクス履歴（チャート用） */
export interface MetricsHistory {
  cpuUsage: TimeSeriesDataPoint[];
  memoryUsage: TimeSeriesDataPoint[];
  gpuUsage: TimeSeriesDataPoint[];
  networkUpload: TimeSeriesDataPoint[];
  networkDownload: TimeSeriesDataPoint[];
}

// 履歴の最大データポイント数（60秒分 @1Hz）
const MAX_HISTORY_POINTS = 60;

// ========================================
// ストア状態の型定義
// ========================================

interface MetricsState {
  // 現在のメトリクス
  metrics: SystemMetrics | null;
  obsProcessMetrics: ObsProcessMetrics | null;

  // 履歴データ
  history: MetricsHistory;

  // UI状態
  loading: boolean;
  error: string | null;
  lastUpdate: number | null;

  // アクション
  fetchMetrics: () => Promise<void>;
  fetchObsProcessMetrics: () => Promise<void>;
  startPolling: (intervalMs?: number) => () => void;
  clearHistory: () => void;
}

// ========================================
// 履歴データにポイントを追加するヘルパー
// ========================================

function addToHistory(
  history: TimeSeriesDataPoint[],
  value: number,
  maxPoints: number = MAX_HISTORY_POINTS
): TimeSeriesDataPoint[] {
  const newPoint: TimeSeriesDataPoint = {
    timestamp: Date.now(),
    value,
  };

  const updated = [...history, newPoint];

  // 古いデータを削除（最大ポイント数を超えた場合）
  if (updated.length > maxPoints) {
    return updated.slice(-maxPoints);
  }

  return updated;
}

// ========================================
// ストア実装
// ========================================

export const useMetricsStore = create<MetricsState>((set, get) => ({
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

  fetchMetrics: async () => {
    set({ loading: true, error: null });
    try {
      const metrics = await invoke<SystemMetrics>('get_system_metrics');

      // 履歴を更新
      const { history } = get();
      // GPU履歴の処理:
      // - GPUデータがある場合: 使用率を履歴に追加
      // - GPUデータがない場合（GPUなし/検出失敗）: 履歴をクリアして不整合を防止
      const gpuUsage = metrics.gpu
        ? addToHistory(history.gpuUsage, metrics.gpu.usagePercent)
        : []; // GPUなしの場合は履歴をクリア

      const newHistory: MetricsHistory = {
        cpuUsage: addToHistory(history.cpuUsage, metrics.cpu.usagePercent),
        memoryUsage: addToHistory(history.memoryUsage, metrics.memory.usagePercent),
        gpuUsage,
        networkUpload: addToHistory(history.networkUpload, metrics.network.uploadBytesPerSec),
        networkDownload: addToHistory(history.networkDownload, metrics.network.downloadBytesPerSec),
      };

      set({
        metrics,
        history: newHistory,
        loading: false,
        lastUpdate: Date.now(),
      });
    } catch (e) {
      set({ error: e instanceof Error ? e.message : String(e), loading: false });
    }
  },

  fetchObsProcessMetrics: async () => {
    try {
      const obsProcessMetrics = await invoke<ObsProcessMetrics>('get_process_metrics');
      set({ obsProcessMetrics });
    } catch (e) {
      // OBSプロセスメトリクスの取得失敗はエラーとして扱わない（OBSが起動していない可能性）
      set({ obsProcessMetrics: null });
    }
  },

  startPolling: (intervalMs = 1000) => {
    const { fetchMetrics, fetchObsProcessMetrics } = get();

    // 初回取得
    fetchMetrics();
    fetchObsProcessMetrics();

    // 定期取得
    const metricsIntervalId = setInterval(fetchMetrics, intervalMs);
    // OBSプロセスメトリクスは少し頻度を下げる（2倍の間隔）
    const obsIntervalId = setInterval(fetchObsProcessMetrics, intervalMs * 2);

    return () => {
      clearInterval(metricsIntervalId);
      clearInterval(obsIntervalId);
    };
  },

  clearHistory: () => {
    set({
      history: {
        cpuUsage: [],
        memoryUsage: [],
        gpuUsage: [],
        networkUpload: [],
        networkDownload: [],
      },
    });
  },
}));
