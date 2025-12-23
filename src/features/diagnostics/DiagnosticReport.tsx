import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { AnalysisResult, OptimizationPreset, OptimizationResult } from '../../types/commands';
import { ComparisonCard } from './components/ComparisonCard';
import { QualityScore } from './components/QualityScore';

/**
 * 診断レポートコンポーネント
 *
 * OBS設定の診断結果を表示し、最適化を適用できる
 *
 * @example
 * <DiagnosticReport />
 */
export function DiagnosticReport() {
  const [analyzing, setAnalyzing] = useState(false);
  const [result, setResult] = useState<AnalysisResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selectedKeys, setSelectedKeys] = useState<Set<string>>(new Set());
  const [applying, setApplying] = useState(false);
  const [optimizationResult, setOptimizationResult] = useState<OptimizationResult | null>(null);

  // 初回自動診断
  useEffect(() => {
    void runAnalysis();
  }, []);

  // 診断実行
  const runAnalysis = async () => {
    setAnalyzing(true);
    setError(null);
    setOptimizationResult(null);

    try {
      const analysisResult = await invoke<AnalysisResult>('analyze_settings');
      setResult(analysisResult);
      // デフォルトでcriticalとrecommendedを選択
      const defaultSelected = new Set(
        analysisResult.recommendations
          .filter((s) => s.priority === 'critical' || s.priority === 'recommended')
          .map((s) => s.key)
      );
      setSelectedKeys(defaultSelected);
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      setError(errorMessage);
    } finally {
      setAnalyzing(false);
    }
  };

  // 最適化適用
  const applyOptimization = async (preset: OptimizationPreset) => {
    if (!result) {return;}

    setApplying(true);
    setError(null);

    try {
      const optimizationRes = await invoke<OptimizationResult>('apply_optimization', {
        params: {
          preset,
          selectedKeys: Array.from(selectedKeys),
        },
      });
      setOptimizationResult(optimizationRes);
      // 適用後に再診断
      await runAnalysis();
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      setError(errorMessage);
    } finally {
      setApplying(false);
    }
  };

  // チェックボックスのトグル
  const toggleSelection = (key: string) => {
    const newSelection = new Set(selectedKeys);
    if (newSelection.has(key)) {
      newSelection.delete(key);
    } else {
      newSelection.add(key);
    }
    setSelectedKeys(newSelection);
  };

  // すべて選択/解除
  const toggleAll = () => {
    if (!result) {return;}

    if (selectedKeys.size === result.recommendations.length) {
      setSelectedKeys(new Set());
    } else {
      setSelectedKeys(new Set(result.recommendations.map((s) => s.key)));
    }
  };

  return (
    <div className="bg-white rounded-lg shadow-md p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-gray-900">診断レポート</h2>
        <button
          onClick={() => void runAnalysis()}
          disabled={analyzing}
          className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {analyzing ? '診断中...' : '再診断'}
        </button>
      </div>

      {/* ローディング */}
      {analyzing && (
        <div className="flex justify-center py-12">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500" />
        </div>
      )}

      {/* エラー表示 */}
      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
          <p className="text-sm text-red-800">{error}</p>
        </div>
      )}

      {/* 最適化適用結果 */}
      {optimizationResult && (
        <div className="bg-green-50 border border-green-200 rounded-lg p-4 mb-6">
          <p className="text-sm text-green-800">
            {optimizationResult.appliedCount}件の設定を適用しました
            {optimizationResult.failedCount > 0 &&
              `（${optimizationResult.failedCount}件は失敗）`}
          </p>
        </div>
      )}

      {/* 診断結果 */}
      {result && !analyzing && (
        <div className="space-y-6">
          {/* スコアとシステム情報 */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* 品質スコア */}
            <QualityScore score={result.qualityScore} />

            {/* システム情報 */}
            <div className="bg-gray-50 rounded-lg p-6 border border-gray-200">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">システム環境</h3>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-600">CPU:</span>
                  <span className="font-medium text-gray-900">{result.systemInfo.cpuModel}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">GPU:</span>
                  <span className="font-medium text-gray-900">
                    {result.systemInfo.gpuModel || '検出できませんでした'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">メモリ:</span>
                  <span className="font-medium text-gray-900">
                    {result.systemInfo.totalMemoryMb.toLocaleString()} MB
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">利用可能:</span>
                  <span className="font-medium text-gray-900">
                    {result.systemInfo.availableMemoryMb.toLocaleString()} MB
                  </span>
                </div>
              </div>
            </div>
          </div>

          {/* 推奨設定 */}
          {result.recommendations.length > 0 ? (
            <div>
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold text-gray-900">
                  推奨される設定変更 ({result.recommendations.length}件)
                </h3>
                <button
                  onClick={toggleAll}
                  className="text-sm text-blue-600 hover:text-blue-800 font-medium"
                >
                  {selectedKeys.size === result.recommendations.length
                    ? 'すべて解除'
                    : 'すべて選択'}
                </button>
              </div>

              <div className="space-y-3 mb-6">
                {result.recommendations.map((setting) => (
                  <ComparisonCard
                    key={setting.key}
                    setting={setting}
                    selected={selectedKeys.has(setting.key)}
                    onToggle={toggleSelection}
                  />
                ))}
              </div>

              {/* 適用ボタン */}
              <div className="flex gap-3">
                <button
                  onClick={() => void applyOptimization('medium')}
                  disabled={applying || selectedKeys.size === 0}
                  className="flex-1 px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-semibold"
                >
                  {applying ? '適用中...' : `選択した設定を適用 (${selectedKeys.size}件)`}
                </button>
              </div>
            </div>
          ) : (
            <div className="bg-green-50 border border-green-200 rounded-lg p-6 text-center">
              <svg
                className="w-12 h-12 text-green-600 mx-auto mb-3"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fillRule="evenodd"
                  d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                  clipRule="evenodd"
                />
              </svg>
              <p className="text-green-800 font-semibold">
                現在の設定は最適です！
              </p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
