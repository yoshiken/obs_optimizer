import { useEffect, useState } from 'react';
import { useAnalysisStore } from '../../stores/analysisStore';
import { ProblemCard } from './ProblemCard';
import type { AlertSeverity, ProblemCategory } from '../../types/commands';

/**
 * 問題分析ダッシュボード
 * - 現在検出されている問題の一覧
 * - 問題のカテゴリ別フィルタリング
 * - 重要度別ソート
 * - 推奨アクションの表示
 */
export function ProblemDashboard() {
  const { problems, isAnalyzing, error, analyzeProblems, clearError } = useAnalysisStore();
  const [categoryFilter, setCategoryFilter] = useState<ProblemCategory | 'all'>('all');
  const [severityFilter, setSeverityFilter] = useState<AlertSeverity | 'all'>('all');

  useEffect(() => {
    // 初回マウント時に問題分析を実行
    void analyzeProblems();
  }, [analyzeProblems]);

  // フィルタリングされた問題一覧
  const filteredProblems = problems.filter((problem) => {
    const matchCategory = categoryFilter === 'all' || problem.category === categoryFilter;
    const matchSeverity = severityFilter === 'all' || problem.severity === severityFilter;
    return matchCategory && matchSeverity;
  });

  // 重要度順にソート（critical > warning > info > tips）
  const severityOrder: Record<AlertSeverity, number> = {
    critical: 0,
    warning: 1,
    info: 2,
    tips: 3,
  };

  const sortedProblems = [...filteredProblems].sort((a, b) => {
    return severityOrder[a.severity] - severityOrder[b.severity];
  });

  // 統計情報
  const stats = {
    total: problems.length,
    critical: problems.filter((p) => p.severity === 'critical').length,
    warning: problems.filter((p) => p.severity === 'warning').length,
    info: problems.filter((p) => p.severity === 'info').length,
    tips: problems.filter((p) => p.severity === 'tips').length,
  };

  return (
    <div className="max-w-6xl mx-auto p-6">
      {/* ヘッダー */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-4">
          <h1 className="text-2xl font-bold text-gray-900">問題分析ダッシュボード</h1>
          <button
            onClick={() => {
              void analyzeProblems();
            }}
            disabled={isAnalyzing}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            aria-label="問題を再分析"
          >
            {isAnalyzing ? '分析中...' : '再分析'}
          </button>
        </div>

        {/* エラー表示 */}
        {error && (
          <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-4" role="alert">
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-2">
                <span className="text-red-600 font-semibold">エラー:</span>
                <span className="text-red-700">{error}</span>
              </div>
              <button
                onClick={clearError}
                className="text-red-600 hover:text-red-800"
                aria-label="エラーを閉じる"
              >
                ✕
              </button>
            </div>
          </div>
        )}

        {/* 統計情報 */}
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-4 mb-6">
          <div className="bg-gray-100 rounded-lg p-4">
            <div className="text-sm text-gray-600 mb-1">合計</div>
            <div className="text-2xl font-bold text-gray-900">{stats.total}</div>
          </div>
          <div className="bg-red-50 rounded-lg p-4">
            <div className="text-sm text-red-600 mb-1">重大</div>
            <div className="text-2xl font-bold text-red-700">{stats.critical}</div>
          </div>
          <div className="bg-yellow-50 rounded-lg p-4">
            <div className="text-sm text-yellow-600 mb-1">警告</div>
            <div className="text-2xl font-bold text-yellow-700">{stats.warning}</div>
          </div>
          <div className="bg-blue-50 rounded-lg p-4">
            <div className="text-sm text-blue-600 mb-1">情報</div>
            <div className="text-2xl font-bold text-blue-700">{stats.info}</div>
          </div>
          <div className="bg-green-50 rounded-lg p-4">
            <div className="text-sm text-green-600 mb-1">ヒント</div>
            <div className="text-2xl font-bold text-green-700">{stats.tips}</div>
          </div>
        </div>
      </div>

      {/* フィルター */}
      <div className="bg-white rounded-lg shadow-sm p-4 mb-6">
        <h2 className="text-sm font-semibold text-gray-700 mb-3">フィルター</h2>
        <div className="flex flex-wrap gap-4">
          {/* カテゴリフィルター */}
          <div className="flex-1 min-w-[200px]">
            <label htmlFor="category-filter" className="block text-sm text-gray-600 mb-1">
              カテゴリ
            </label>
            <select
              id="category-filter"
              value={categoryFilter}
              onChange={(e) => setCategoryFilter(e.target.value as ProblemCategory | 'all')}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">すべて</option>
              <option value="encoding">エンコーディング</option>
              <option value="network">ネットワーク</option>
              <option value="resource">リソース</option>
              <option value="settings">設定</option>
            </select>
          </div>

          {/* 重要度フィルター */}
          <div className="flex-1 min-w-[200px]">
            <label htmlFor="severity-filter" className="block text-sm text-gray-600 mb-1">
              重要度
            </label>
            <select
              id="severity-filter"
              value={severityFilter}
              onChange={(e) => setSeverityFilter(e.target.value as AlertSeverity | 'all')}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">すべて</option>
              <option value="critical">重大</option>
              <option value="warning">警告</option>
              <option value="info">情報</option>
              <option value="tips">ヒント</option>
            </select>
          </div>
        </div>
      </div>

      {/* 問題一覧 */}
      <div className="space-y-4">
        {isAnalyzing ? (
          <div className="text-center py-12" role="status" aria-live="polite">
            <div className="inline-block animate-spin rounded-full h-12 w-12 border-4 border-gray-300 border-t-blue-600" />
            <p className="mt-4 text-gray-600">問題を分析中...</p>
          </div>
        ) : sortedProblems.length === 0 ? (
          <div className="bg-green-50 border border-green-200 rounded-lg p-8 text-center">
            <p className="text-green-700 text-lg font-semibold">問題は検出されませんでした</p>
            <p className="text-green-600 text-sm mt-2">配信設定は最適な状態です</p>
          </div>
        ) : (
          <div role="list" aria-label="検出された問題一覧">
            {sortedProblems.map((problem) => (
              <ProblemCard key={problem.id} problem={problem} />
            ))}
          </div>
        )}
      </div>

      {/* フィルター結果表示 */}
      {!isAnalyzing && sortedProblems.length > 0 && (
        <div className="mt-4 text-sm text-gray-600 text-center">
          {filteredProblems.length} / {problems.length} 件の問題を表示中
        </div>
      )}
    </div>
  );
}
