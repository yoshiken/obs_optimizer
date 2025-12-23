import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useAnalysisStore } from './analysisStore';
import { invoke } from '@tauri-apps/api/core';
import type {
  AlertSeverity,
  AnalyzeProblemsResponse,
  ProblemCategory,
  ProblemReport,
} from '../types/commands';

vi.mock('@tauri-apps/api/core');

const mockInvoke = vi.mocked(invoke);

// モックデータ
const mockProblemReport: ProblemReport = {
  id: 'problem-1',
  category: 'encoding',
  severity: 'warning',
  title: 'エンコーダー設定の問題',
  description: 'x264プリセットが遅すぎます',
  suggestedActions: ['プリセットをfastに変更してください', 'ハードウェアエンコーダーの使用を検討してください'],
  affectedMetric: 'cpuUsage',
  detectedAt: Date.now(),
};

const mockAnalyzeProblemsResponse: AnalyzeProblemsResponse = {
  problems: [mockProblemReport],
  overallScore: 75,
};

const mockProblemHistory: ProblemReport[] = [
  mockProblemReport,
  {
    id: 'problem-2',
    category: 'network',
    severity: 'critical',
    title: 'ネットワーク帯域不足',
    description: 'アップロード速度が不十分です',
    suggestedActions: ['ビットレートを下げてください'],
    affectedMetric: 'networkBandwidth',
    detectedAt: Date.now() - 1000,
  },
];

describe('analysisStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // ストアをリセット
    useAnalysisStore.setState({
      problems: [],
      problemHistory: [],
      isAnalyzing: false,
      overallScore: null,
      error: null,
    });
  });

  describe('初期状態', () => {
    it('問題リストが空で初期化される', () => {
      const state = useAnalysisStore.getState();
      expect(state.problems).toEqual([]);
      expect(state.problemHistory).toEqual([]);
      expect(state.isAnalyzing).toBe(false);
      expect(state.overallScore).toBeNull();
      expect(state.error).toBeNull();
    });
  });

  describe('analyzeProblems', () => {
    it('問題分析を実行して結果を取得できる', async () => {
      mockInvoke.mockResolvedValue(mockAnalyzeProblemsResponse);

      const { analyzeProblems } = useAnalysisStore.getState();
      await analyzeProblems('x264', 6000);

      const state = useAnalysisStore.getState();
      expect(state.problems).toEqual(mockAnalyzeProblemsResponse.problems);
      expect(state.overallScore).toBe(75);
      expect(state.isAnalyzing).toBe(false);
      expect(state.error).toBeNull();

      expect(mockInvoke).toHaveBeenCalledWith('analyze_problems', {
        request: { encoderType: 'x264', targetBitrate: 6000 },
      });
    });

    it('分析中はローディング状態になる', async () => {
      mockInvoke.mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(() => resolve(mockAnalyzeProblemsResponse), 100);
          })
      );

      const { analyzeProblems } = useAnalysisStore.getState();
      const promise = analyzeProblems('nvencH264', 8000);

      // 分析開始直後はローディング中
      expect(useAnalysisStore.getState().isAnalyzing).toBe(true);

      await promise;

      // 完了後はローディング解除
      expect(useAnalysisStore.getState().isAnalyzing).toBe(false);
    });

    it('デフォルトパラメータで分析できる', async () => {
      mockInvoke.mockResolvedValue(mockAnalyzeProblemsResponse);

      const { analyzeProblems } = useAnalysisStore.getState();
      await analyzeProblems();

      expect(mockInvoke).toHaveBeenCalledWith('analyze_problems', {
        request: { encoderType: 'x264', targetBitrate: 6000 },
      });
    });

    it('カスタムパラメータで分析できる', async () => {
      mockInvoke.mockResolvedValue(mockAnalyzeProblemsResponse);

      const { analyzeProblems } = useAnalysisStore.getState();
      await analyzeProblems('nvencH264', 10000);

      expect(mockInvoke).toHaveBeenCalledWith('analyze_problems', {
        request: { encoderType: 'nvencH264', targetBitrate: 10000 },
      });
    });

    it('エラー時にエラー状態を設定する', async () => {
      const errorMessage = 'Analysis failed';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      const { analyzeProblems } = useAnalysisStore.getState();
      await analyzeProblems('x264', 6000);

      const state = useAnalysisStore.getState();
      expect(state.error).toBe(errorMessage);
      expect(state.isAnalyzing).toBe(false);
    });

    it('エラーが文字列の場合もエラー処理できる', async () => {
      mockInvoke.mockRejectedValue('String error');

      const { analyzeProblems } = useAnalysisStore.getState();
      await analyzeProblems('x264', 6000);

      const state = useAnalysisStore.getState();
      expect(state.error).toBe('問題の分析に失敗しました');
      expect(state.isAnalyzing).toBe(false);
    });

    it('分析開始時にエラーをクリアする', async () => {
      // 最初にエラー状態を作る
      useAnalysisStore.setState({ error: '前回のエラー' });

      mockInvoke.mockResolvedValue(mockAnalyzeProblemsResponse);

      const { analyzeProblems } = useAnalysisStore.getState();
      await analyzeProblems('x264', 6000);

      const state = useAnalysisStore.getState();
      expect(state.error).toBeNull();
    });

    it('複数回分析しても結果が正しく上書きされる', async () => {
      const firstResponse: AnalyzeProblemsResponse = {
        problems: [mockProblemReport],
        overallScore: 60,
      };

      const secondResponse: AnalyzeProblemsResponse = {
        problems: [],
        overallScore: 90,
      };

      mockInvoke.mockResolvedValueOnce(firstResponse);

      const { analyzeProblems } = useAnalysisStore.getState();
      await analyzeProblems('x264', 6000);

      expect(useAnalysisStore.getState().overallScore).toBe(60);
      expect(useAnalysisStore.getState().problems).toHaveLength(1);

      mockInvoke.mockResolvedValueOnce(secondResponse);
      await analyzeProblems('nvencH264', 8000);

      expect(useAnalysisStore.getState().overallScore).toBe(90);
      expect(useAnalysisStore.getState().problems).toHaveLength(0);
    });
  });

  describe('loadProblemHistory', () => {
    it('問題履歴を取得できる', async () => {
      mockInvoke.mockResolvedValue(mockProblemHistory);

      const { loadProblemHistory } = useAnalysisStore.getState();
      await loadProblemHistory();

      const state = useAnalysisStore.getState();
      expect(state.problemHistory).toEqual(mockProblemHistory);
      expect(state.problemHistory).toHaveLength(2);

      expect(mockInvoke).toHaveBeenCalledWith('get_problem_history', { limit: 100 });
    });

    it('カスタムlimitパラメータで履歴を取得できる', async () => {
      mockInvoke.mockResolvedValue(mockProblemHistory);

      const { loadProblemHistory } = useAnalysisStore.getState();
      await loadProblemHistory(50);

      expect(mockInvoke).toHaveBeenCalledWith('get_problem_history', { limit: 50 });
    });

    it('エラー時にエラー状態を設定する', async () => {
      const errorMessage = 'Failed to load history';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      const { loadProblemHistory } = useAnalysisStore.getState();
      await loadProblemHistory();

      const state = useAnalysisStore.getState();
      expect(state.error).toBe(errorMessage);
    });

    it('エラーが文字列の場合もエラー処理できる', async () => {
      mockInvoke.mockRejectedValue('String error');

      const { loadProblemHistory } = useAnalysisStore.getState();
      await loadProblemHistory();

      const state = useAnalysisStore.getState();
      expect(state.error).toBe('問題履歴の取得に失敗しました');
    });
  });

  describe('clearProblems', () => {
    it('問題リストとエラーをクリアできる', () => {
      // 状態を設定
      useAnalysisStore.setState({
        problems: [mockProblemReport],
        error: 'エラーメッセージ',
      });

      const { clearProblems } = useAnalysisStore.getState();
      clearProblems();

      const state = useAnalysisStore.getState();
      expect(state.problems).toEqual([]);
      expect(state.error).toBeNull();
    });

    it('問題がない状態でclearProblemsを呼んでもエラーにならない', () => {
      const { clearProblems } = useAnalysisStore.getState();

      expect(() => clearProblems()).not.toThrow();

      const state = useAnalysisStore.getState();
      expect(state.problems).toEqual([]);
    });

    it('overallScoreはクリアされない', () => {
      useAnalysisStore.setState({
        problems: [mockProblemReport],
        overallScore: 75,
      });

      const { clearProblems } = useAnalysisStore.getState();
      clearProblems();

      const state = useAnalysisStore.getState();
      expect(state.overallScore).toBe(75);
    });
  });

  describe('clearError', () => {
    it('エラーメッセージをクリアできる', () => {
      useAnalysisStore.setState({ error: 'エラーメッセージ' });

      const { clearError } = useAnalysisStore.getState();
      clearError();

      const state = useAnalysisStore.getState();
      expect(state.error).toBeNull();
    });

    it('エラーがない状態でclearErrorを呼んでもエラーにならない', () => {
      const { clearError } = useAnalysisStore.getState();

      expect(() => clearError()).not.toThrow();

      const state = useAnalysisStore.getState();
      expect(state.error).toBeNull();
    });
  });

  describe('複合操作', () => {
    it('分析→履歴読み込み→クリアの一連の流れが正しく動作する', async () => {
      mockInvoke.mockResolvedValueOnce(mockAnalyzeProblemsResponse);
      mockInvoke.mockResolvedValueOnce(mockProblemHistory);

      const { analyzeProblems, loadProblemHistory, clearProblems } = useAnalysisStore.getState();

      // 分析実行
      await analyzeProblems('x264', 6000);
      expect(useAnalysisStore.getState().problems).toHaveLength(1);
      expect(useAnalysisStore.getState().overallScore).toBe(75);

      // 履歴読み込み
      await loadProblemHistory(50);
      expect(useAnalysisStore.getState().problemHistory).toHaveLength(2);

      // 問題をクリア
      clearProblems();
      expect(useAnalysisStore.getState().problems).toEqual([]);
      // 履歴はクリアされない
      expect(useAnalysisStore.getState().problemHistory).toHaveLength(2);
    });

    it('エラー発生後にclearErrorを呼んで再分析できる', async () => {
      // 最初はエラー
      mockInvoke.mockRejectedValueOnce(new Error('Initial error'));

      const { analyzeProblems, clearError } = useAnalysisStore.getState();
      await analyzeProblems('x264', 6000);

      expect(useAnalysisStore.getState().error).toBe('Initial error');

      // エラーをクリア
      clearError();
      expect(useAnalysisStore.getState().error).toBeNull();

      // 再分析成功
      mockInvoke.mockResolvedValueOnce(mockAnalyzeProblemsResponse);
      await analyzeProblems('x264', 6000);

      const state = useAnalysisStore.getState();
      expect(state.error).toBeNull();
      expect(state.problems).toHaveLength(1);
    });
  });

  describe('問題カテゴリーとseverityの処理', () => {
    it('全てのカテゴリーの問題を処理できる', async () => {
      const categories: ProblemCategory[] = ['encoding', 'network', 'resource', 'settings'];
      const problems: ProblemReport[] = categories.map((category, index) => ({
        id: `problem-${index}`,
        category,
        severity: 'warning',
        title: `${category}の問題`,
        description: `${category}に関する問題`,
        suggestedActions: ['対処してください'],
        affectedMetric: 'cpuUsage',
        detectedAt: Date.now(),
      }));

      mockInvoke.mockResolvedValue({ problems, overallScore: 50 });

      const { analyzeProblems } = useAnalysisStore.getState();
      await analyzeProblems();

      const state = useAnalysisStore.getState();
      expect(state.problems).toHaveLength(4);
      categories.forEach((category, index) => {
        expect(state.problems[index].category).toBe(category);
      });
    });

    it('全てのseverityレベルの問題を処理できる', async () => {
      const severities: AlertSeverity[] = ['critical', 'warning', 'info', 'tips'];
      const problems: ProblemReport[] = severities.map((severity, index) => ({
        id: `problem-${index}`,
        category: 'encoding',
        severity,
        title: `${severity}問題`,
        description: `${severity}レベルの問題`,
        suggestedActions: ['対処してください'],
        affectedMetric: 'cpuUsage',
        detectedAt: Date.now(),
      }));

      mockInvoke.mockResolvedValue({ problems, overallScore: 60 });

      const { analyzeProblems } = useAnalysisStore.getState();
      await analyzeProblems();

      const state = useAnalysisStore.getState();
      expect(state.problems).toHaveLength(4);
      severities.forEach((severity, index) => {
        expect(state.problems[index].severity).toBe(severity);
      });
    });
  });
});
