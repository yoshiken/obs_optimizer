import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { HistoricalMetrics, SessionSummary } from '../types/commands';

interface HistoryState {
  /** セッション一覧 */
  sessions: SessionSummary[];
  /** 選択中のセッションID（最大2つ） */
  selectedSessionIds: string[];
  /** メトリクスデータ */
  metricsData: HistoricalMetrics[];
  /** ローディング状態 */
  isLoading: boolean;
  /** エラーメッセージ */
  error: string | null;

  /** セッション一覧を読み込み */
  loadSessions: () => Promise<void>;
  /** セッションを選択 */
  selectSession: (id: string) => void;
  /** セッション選択を解除 */
  deselectSession: (id: string) => void;
  /** メトリクスデータを読み込み */
  loadMetrics: (sessionId: string, from: number, to: number) => Promise<void>;
  /** 選択をクリア */
  clearSelection: () => void;
  /** エラーをクリア */
  clearError: () => void;
}

export const useHistoryStore = create<HistoryState>((set, get) => ({
  sessions: [],
  selectedSessionIds: [],
  metricsData: [],
  isLoading: false,
  error: null,

  loadSessions: async () => {
    set({ isLoading: true, error: null });
    try {
      const sessions = await invoke<SessionSummary[]>('get_sessions');
      set({ sessions, isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'セッション一覧の取得に失敗しました';
      set({ error: message, isLoading: false });
      console.error('Failed to load sessions:', error);
    }
  },

  selectSession: (id: string) => {
    const { selectedSessionIds } = get();

    // 既に選択されている場合は何もしない
    if (selectedSessionIds.includes(id)) {
      return;
    }

    // 最大2つまで選択可能
    if (selectedSessionIds.length >= 2) {
      // 最初のセッションを削除して新しいものを追加
      set({ selectedSessionIds: [selectedSessionIds[1], id] });
    } else {
      set({ selectedSessionIds: [...selectedSessionIds, id] });
    }
  },

  deselectSession: (id: string) => {
    const { selectedSessionIds } = get();
    set({ selectedSessionIds: selectedSessionIds.filter((sid) => sid !== id) });
  },

  loadMetrics: async (sessionId: string, from: number, to: number) => {
    set({ isLoading: true, error: null });
    try {
      const metrics = await invoke<HistoricalMetrics[]>('get_metrics_range', {
        sessionId,
        from,
        to,
      });
      set({ metricsData: metrics, isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'メトリクスデータの取得に失敗しました';
      set({ error: message, isLoading: false });
      console.error('Failed to load metrics:', error);
    }
  },

  clearSelection: () => {
    set({ selectedSessionIds: [], metricsData: [] });
  },

  clearError: () => {
    set({ error: null });
  },
}));
