import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { DiagnosticReport } from '../../types/commands';

/**
 * 診断レポート生成
 * - 診断レポート生成ボタン
 * - レポートプレビュー
 * - PDF保存（ブラウザ印刷機能利用）
 */
export function ReportGenerator() {
  const [report, setReport] = useState<DiagnosticReport | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleGenerate = async () => {
    setIsGenerating(true);
    setError(null);

    try {
      const diagnosticReport = await invoke<DiagnosticReport>('generate_diagnostic_report');
      setReport(diagnosticReport);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'レポート生成に失敗しました';
      setError(message);
      console.error('Report generation failed:', err);
    } finally {
      setIsGenerating(false);
    }
  };

  const handlePrint = () => {
    window.print();
  };

  const formatDuration = (durationSecs: number): string => {
    const hours = Math.floor(durationSecs / 3600);
    const minutes = Math.floor((durationSecs % 3600) / 60);
    return `${hours}時間${minutes}分`;
  };

  return (
    <div className="max-w-5xl mx-auto p-6">
      <div className="flex items-center justify-between mb-6 print:hidden">
        <h1 className="text-2xl font-bold text-gray-900">診断レポート</h1>
        <div className="flex gap-2">
          <button
            onClick={() => {
              void handleGenerate();
            }}
            disabled={isGenerating}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            aria-label="レポートを生成"
          >
            {isGenerating ? 'レポート生成中...' : 'レポート生成'}
          </button>
          {report && (
            <button
              onClick={handlePrint}
              className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
              aria-label="PDFとして保存"
            >
              PDF保存
            </button>
          )}
        </div>
      </div>

      {/* エラー表示 */}
      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-4 print:hidden" role="alert">
          <div className="flex items-start justify-between">
            <div className="flex items-center gap-2">
              <span className="text-red-600 font-semibold">エラー:</span>
              <span className="text-red-700">{error}</span>
            </div>
            <button
              onClick={() => setError(null)}
              className="text-red-600 hover:text-red-800"
              aria-label="エラーを閉じる"
            >
              ✕
            </button>
          </div>
        </div>
      )}

      {/* レポートプレビュー */}
      {report ? (
        <div className="bg-white border border-gray-300 rounded-lg shadow-lg p-8 print:shadow-none print:border-0">
          {/* ヘッダー */}
          <div className="mb-8 pb-6 border-b border-gray-300">
            <h2 className="text-3xl font-bold text-gray-900 mb-2">配信診断レポート</h2>
            <p className="text-sm text-gray-600">
              生成日時: {new Date(report.generatedAt).toLocaleString('ja-JP')}
            </p>
          </div>

          {/* セッション情報 */}
          <section className="mb-8">
            <h3 className="text-xl font-semibold text-gray-900 mb-4">セッション情報</h3>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <p className="text-sm text-gray-600">セッションID</p>
                <p className="text-base font-semibold text-gray-900">
                  {report.session.sessionId.substring(0, 8)}
                </p>
              </div>
              <div>
                <p className="text-sm text-gray-600">配信時間</p>
                <p className="text-base font-semibold text-gray-900">
                  {formatDuration(report.session.durationSecs)}
                </p>
              </div>
              <div>
                <p className="text-sm text-gray-600">開始時刻</p>
                <p className="text-base font-semibold text-gray-900">
                  {new Date(report.session.startedAt).toLocaleString('ja-JP')}
                </p>
              </div>
              <div>
                <p className="text-sm text-gray-600">終了時刻</p>
                <p className="text-base font-semibold text-gray-900">
                  {new Date(report.session.endedAt).toLocaleString('ja-JP')}
                </p>
              </div>
            </div>
          </section>

          {/* パフォーマンス評価 */}
          <section className="mb-8">
            <h3 className="text-xl font-semibold text-gray-900 mb-4">パフォーマンス評価</h3>
            <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
              <div className="bg-blue-50 rounded-lg p-4">
                <p className="text-sm text-blue-600 mb-1">総合スコア</p>
                <p className="text-2xl font-bold text-blue-900">{report.performance.overallScore}</p>
              </div>
              <div className="bg-gray-50 rounded-lg p-4">
                <p className="text-sm text-gray-600 mb-1">CPUスコア</p>
                <p className="text-2xl font-bold text-gray-900">{report.performance.cpuScore}</p>
              </div>
              <div className="bg-gray-50 rounded-lg p-4">
                <p className="text-sm text-gray-600 mb-1">GPUスコア</p>
                <p className="text-2xl font-bold text-gray-900">{report.performance.gpuScore}</p>
              </div>
              <div className="bg-gray-50 rounded-lg p-4">
                <p className="text-sm text-gray-600 mb-1">ネットワークスコア</p>
                <p className="text-2xl font-bold text-gray-900">{report.performance.networkScore}</p>
              </div>
              <div className="bg-gray-50 rounded-lg p-4">
                <p className="text-sm text-gray-600 mb-1">安定性スコア</p>
                <p className="text-2xl font-bold text-gray-900">{report.performance.stabilityScore}</p>
              </div>
            </div>
          </section>

          {/* システム情報 */}
          <section className="mb-8">
            <h3 className="text-xl font-semibold text-gray-900 mb-4">システム情報</h3>
            <div className="space-y-2">
              <p className="text-sm">
                <span className="font-semibold text-gray-700">OS:</span>{' '}
                <span className="text-gray-900">{report.systemInfo.os}</span>
              </p>
              <p className="text-sm">
                <span className="font-semibold text-gray-700">CPU:</span>{' '}
                <span className="text-gray-900">{report.systemInfo.cpuModel}</span>
              </p>
              <p className="text-sm">
                <span className="font-semibold text-gray-700">GPU:</span>{' '}
                <span className="text-gray-900">{report.systemInfo.gpuModel || 'N/A'}</span>
              </p>
              <p className="text-sm">
                <span className="font-semibold text-gray-700">メモリ:</span>{' '}
                <span className="text-gray-900">{report.systemInfo.totalMemoryMb} MB</span>
              </p>
            </div>
          </section>

          {/* 検出された問題 */}
          <section className="mb-8">
            <h3 className="text-xl font-semibold text-gray-900 mb-4">
              検出された問題 ({report.problems.length}件)
            </h3>
            {report.problems.length === 0 ? (
              <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                <p className="text-green-700">問題は検出されませんでした</p>
              </div>
            ) : (
              <div className="space-y-4">
                {report.problems.map((problem) => (
                  <div
                    key={problem.id}
                    className="border border-gray-200 rounded-lg p-4 break-inside-avoid"
                  >
                    <div className="flex items-start justify-between mb-2">
                      <h4 className="font-semibold text-gray-900">{problem.title}</h4>
                      <span
                        className={`text-xs px-2 py-1 rounded ${
                          problem.severity === 'critical'
                            ? 'bg-red-100 text-red-700'
                            : problem.severity === 'warning'
                              ? 'bg-yellow-100 text-yellow-700'
                              : problem.severity === 'info'
                                ? 'bg-blue-100 text-blue-700'
                                : 'bg-green-100 text-green-700'
                        }`}
                      >
                        {problem.severity === 'critical'
                          ? '重大'
                          : problem.severity === 'warning'
                            ? '警告'
                            : problem.severity === 'info'
                              ? '情報'
                              : 'ヒント'}
                      </span>
                    </div>
                    <p className="text-sm text-gray-700 mb-3">{problem.description}</p>
                    {problem.suggestedActions.length > 0 && (
                      <div>
                        <p className="text-sm font-semibold text-gray-700 mb-1">推奨アクション:</p>
                        <ul className="list-disc list-inside text-sm text-gray-600 space-y-1">
                          {problem.suggestedActions.map((action, index) => (
                            <li key={index}>{action}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </section>

          {/* 推奨事項サマリー */}
          <section className="mb-8">
            <h3 className="text-xl font-semibold text-gray-900 mb-4">推奨事項</h3>
            <p className="text-sm text-gray-700 whitespace-pre-wrap">
              {report.recommendationsSummary}
            </p>
          </section>

          {/* フッター */}
          <footer className="pt-6 border-t border-gray-300">
            <p className="text-xs text-gray-500 text-center">
              このレポートはOBS配信最適化ツールによって自動生成されました
            </p>
          </footer>
        </div>
      ) : (
        <div className="bg-gray-50 border border-gray-200 rounded-lg p-8 text-center print:hidden">
          <p className="text-gray-700 text-lg font-semibold mb-2">
            レポートを生成してください
          </p>
          <p className="text-gray-600 text-sm">
            「レポート生成」ボタンをクリックすると、現在のセッションの診断レポートが表示されます
          </p>
        </div>
      )}

      {/* 印刷用スタイル */}
      <style>{`
        @media print {
          body {
            print-color-adjust: exact;
            -webkit-print-color-adjust: exact;
          }
          .print\\:hidden {
            display: none !important;
          }
          .print\\:shadow-none {
            box-shadow: none !important;
          }
          .print\\:border-0 {
            border: 0 !important;
          }
          .break-inside-avoid {
            break-inside: avoid;
          }
        }
      `}</style>
    </div>
  );
}
