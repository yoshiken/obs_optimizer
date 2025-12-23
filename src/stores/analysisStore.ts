import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type {
  AnalyzeProblemsRequest,
  AnalyzeProblemsResponse,
  ProblemReport,
} from '../types/commands';

interface AnalysisState {
  /** 現在検出されている問題一覧 */
  problems: ProblemReport[];
  /** 問題履歴 */
  problemHistory: ProblemReport[];
  /** 分析実行中フラグ */
  isAnalyzing: boolean;
  /** 総合スコア */
  overallScore: number | null;
  /** エラーメッセージ */
  error: string | null;

  /** 問題分析を実行 */
  analyzeProblems: (encoderType?: string, targetBitrate?: number) => Promise<void>;
  /** 問題履歴を取得 */
  loadProblemHistory: (limit?: number) => Promise<void>;
  /** 問題をクリア */
  clearProblems: () => void;
  /** エラーをクリア */
  clearError: () => void;
}

export const useAnalysisStore = create<AnalysisState>((set) => ({
  problems: [],
  problemHistory: [],
  isAnalyzing: false,
  overallScore: null,
  error: null,

  analyzeProblems: async (encoderType = 'x264', targetBitrate = 6000) => {
    set({ isAnalyzing: true, error: null });
    try {
      const request: AnalyzeProblemsRequest = { encoderType, targetBitrate };
      const response = await invoke<AnalyzeProblemsResponse>('analyze_problems', { request });
      set({ problems: response.problems, overallScore: response.overallScore, isAnalyzing: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : '問題の分析に失敗しました';
      set({ error: message, isAnalyzing: false });
      console.error('Failed to analyze problems:', error);
    }
  },

  loadProblemHistory: async (limit = 100) => {
    try {
      const problemHistory = await invoke<ProblemReport[]>('get_problem_history', { limit });
      set({ problemHistory });
    } catch (error) {
      const message = error instanceof Error ? error.message : '問題履歴の取得に失敗しました';
      set({ error: message });
      console.error('Failed to load problem history:', error);
    }
  },

  clearProblems: () => {
    set({ problems: [], error: null });
  },

  clearError: () => {
    set({ error: null });
  },
}));
