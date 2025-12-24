import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { AnalysisResult } from '../../../types/commands';

/**
 * オンボーディング Step 5: 環境分析中（必須）
 */
export function Analysis() {
  const [progress, setProgress] = useState(0);
  const [status, setStatus] = useState('システム情報を取得中...');
  const [result, setResult] = useState<AnalysisResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    const runAnalysis = async () => {
      try {
        // 進捗シミュレーション（実際のバックエンド実装後は削除）
        const steps = [
          { delay: 500, progress: 20, status: 'CPU・メモリ情報を取得中...' },
          { delay: 800, progress: 40, status: 'GPU情報を取得中...' },
          { delay: 1000, progress: 60, status: 'OBS設定を読み込み中...' },
          { delay: 1200, progress: 80, status: '最適化案を計算中...' },
        ];

        for (const step of steps) {
          if (cancelled) {return;}
          await new Promise((resolve) => setTimeout(resolve, step.delay));
          setProgress(step.progress);
          setStatus(step.status);
        }

        // バックエンドの診断コマンドを実行
        if (!cancelled) {
          const analysisResult = await invoke<AnalysisResult>('analyze_settings');
          setResult(analysisResult);
          setProgress(100);
          setStatus('分析完了！');
        }
      } catch (e) {
        if (!cancelled) {
          const errorMessage = e instanceof Error ? e.message : String(e);
          setError(errorMessage);
          setStatus('分析に失敗しました');
        }
      }
    };

    void runAnalysis();

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="space-y-6">
      <div className="text-center mb-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-2">環境を分析中</h2>
        <p className="text-gray-600">
          あなたのPC環境とOBS設定を分析しています
        </p>
      </div>

      {/* プログレスバー */}
      <div className="max-w-md mx-auto">
        <div className="mb-2 flex justify-between text-sm">
          <span className="text-gray-600">{status}</span>
          <span className="font-semibold text-gray-900">{progress}%</span>
        </div>
        <div className="w-full bg-gray-200 rounded-full h-4 overflow-hidden">
          <div
            className="bg-blue-500 h-full transition-all duration-500 ease-out"
            style={{ width: `${progress}%` }}
            role="progressbar"
            aria-valuenow={progress}
            aria-valuemin={0}
            aria-valuemax={100}
          />
        </div>
      </div>

      {/* ローディングアニメーション */}
      {!result && !error && (
        <div className="flex justify-center py-8">
          <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-blue-500" />
        </div>
      )}

      {/* 分析結果 */}
      {result && (
        <div className="space-y-4">
          {/* 完了メッセージ */}
          <div className="bg-green-50 border border-green-200 rounded-lg p-4">
            <div className="flex items-center gap-2 text-green-800 mb-2">
              <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
                <path
                  fillRule="evenodd"
                  d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                  clipRule="evenodd"
                />
              </svg>
              <span className="text-lg font-semibold">分析完了！</span>
            </div>
          </div>

          {/* PCスペック表示 */}
          <div className="bg-gray-50 rounded-lg p-4">
            <h3 className="font-semibold text-gray-900 mb-3">あなたのPC</h3>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
              <div className="bg-white rounded p-3">
                <div className="text-xs text-gray-500 mb-1">GPU</div>
                <p className="font-medium text-sm text-gray-900 break-words">
                  {result.systemInfo.gpuModel || 'GPUなし'}
                </p>
              </div>
              <div className="bg-white rounded p-3">
                <div className="text-xs text-gray-500 mb-1">CPU</div>
                <p className="font-medium text-sm text-gray-900 break-words">
                  {result.systemInfo.cpuModel}
                </p>
              </div>
              <div className="bg-white rounded p-3">
                <div className="text-xs text-gray-500 mb-1">メモリ</div>
                <p className="font-medium text-sm text-gray-900">
                  {(result.systemInfo.totalMemoryMb / 1024).toFixed(0)}GB
                </p>
              </div>
            </div>
          </div>

          {/* 推奨設定の概要 */}
          <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
            <div className="flex items-start gap-2">
              <span className="text-2xl">💡</span>
              <div className="flex-1">
                <p className="text-blue-900 font-medium mb-2">
                  あなたのPCに最適な設定を見つけました
                </p>
                <p className="text-sm text-blue-800">
                  品質スコア: <span className="font-semibold">{result.qualityScore}/100</span>
                  {result.issueCount > 0 && (
                    <> · {result.issueCount}件の改善案があります</>
                  )}
                </p>
              </div>
            </div>
          </div>

          {/* 推奨設定詳細（展開可能） */}
          {result.recommendations.length > 0 && (
            <details className="bg-white border border-gray-200 rounded-lg">
              <summary className="px-4 py-3 cursor-pointer hover:bg-gray-50 transition-colors font-medium text-gray-900">
                推奨設定の詳細を見る ({result.recommendations.length}件)
              </summary>
              <div className="px-4 pb-4 pt-2 space-y-3">
                {result.recommendations.map((rec, index) => (
                  <div key={index} className="border-l-2 border-blue-500 pl-3 py-1">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="font-medium text-sm text-gray-900">
                        {rec.displayName}
                      </span>
                      <span
                        className={`text-xs px-2 py-0.5 rounded ${
                          rec.priority === 'critical'
                            ? 'bg-red-100 text-red-800'
                            : rec.priority === 'recommended'
                            ? 'bg-yellow-100 text-yellow-800'
                            : 'bg-gray-100 text-gray-800'
                        }`}
                      >
                        {rec.priority === 'critical'
                          ? '重要'
                          : rec.priority === 'recommended'
                          ? '推奨'
                          : '任意'}
                      </span>
                    </div>
                    <p className="text-xs text-gray-600 mb-1">{rec.reason}</p>
                    <p className="text-xs text-gray-500">
                      現在: {String(rec.currentValue)} → 推奨: {String(rec.recommendedValue)}
                    </p>
                  </div>
                ))}
              </div>
            </details>
          )}
        </div>
      )}

      {/* エラー表示 */}
      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-6 text-center">
          <div className="text-red-800 mb-2">
            <svg className="w-8 h-8 mx-auto mb-2" fill="currentColor" viewBox="0 0 20 20">
              <path
                fillRule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                clipRule="evenodd"
              />
            </svg>
            <p className="font-semibold">分析に失敗しました</p>
          </div>
          <p className="text-sm text-red-700">{error}</p>
        </div>
      )}
    </div>
  );
}
