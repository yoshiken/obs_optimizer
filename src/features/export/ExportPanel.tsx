import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useHistoryStore } from '../../stores/historyStore';
import type { ExportCsvResponse, ExportJsonResponse } from '../../types/commands';

type ExportFormat = 'json' | 'csv';

/**
 * エクスポートパネル
 * - エクスポート形式選択（JSON/CSV）
 * - セッション選択
 * - ダウンロードボタン
 * - エクスポート中プログレス
 */
export function ExportPanel() {
  const { sessions, selectedSessionIds } = useHistoryStore();
  const [exportFormat, setExportFormat] = useState<ExportFormat>('json');
  const [isExporting, setIsExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const handleExport = async () => {
    if (selectedSessionIds.length === 0) {
      setError('エクスポートするセッションを選択してください');
      return;
    }

    setIsExporting(true);
    setError(null);
    setSuccess(null);

    try {
      // 選択されたセッションをエクスポート
      const sessionId = selectedSessionIds[0]; // 最初のセッションをエクスポート

      // Tauriコマンドを呼び出してデータを取得
      const response =
        exportFormat === 'json'
          ? await invoke<ExportJsonResponse>('export_session_json', { request: { sessionId } })
          : await invoke<ExportCsvResponse>('export_session_csv', { request: { sessionId } });

      const { data, filename } = response;

      // ブラウザのダウンロード機能を使用
      const blob = new Blob([data], {
        type: exportFormat === 'json' ? 'application/json' : 'text/csv',
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      setSuccess(`セッションデータを ${filename} としてダウンロードしました`);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'エクスポートに失敗しました';
      setError(message);
      console.error('Export failed:', err);
    } finally {
      setIsExporting(false);
    }
  };

  const selectedSession = sessions.find((s) => s.sessionId === selectedSessionIds[0]);

  return (
    <div className="max-w-4xl mx-auto p-6">
      <h1 className="text-2xl font-bold text-gray-900 mb-6">セッションデータのエクスポート</h1>

      {/* エラー表示 */}
      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-4" role="alert">
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

      {/* 成功メッセージ */}
      {success && (
        <div className="bg-green-50 border border-green-200 rounded-lg p-4 mb-4" role="status">
          <div className="flex items-start justify-between">
            <div className="flex items-center gap-2">
              <span className="text-green-600 font-semibold">成功:</span>
              <span className="text-green-700">{success}</span>
            </div>
            <button
              onClick={() => setSuccess(null)}
              className="text-green-600 hover:text-green-800"
              aria-label="メッセージを閉じる"
            >
              ✕
            </button>
          </div>
        </div>
      )}

      {/* フォーマット選択 */}
      <fieldset className="bg-white border border-gray-200 rounded-lg p-6 mb-6">
        <legend className="text-lg font-semibold text-gray-900 mb-4 px-2">エクスポート形式</legend>
        <div className="space-y-3">
          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="radio"
              name="exportFormat"
              value="json"
              checked={exportFormat === 'json'}
              onChange={() => setExportFormat('json')}
              className="w-4 h-4 text-blue-600 focus:ring-2 focus:ring-blue-500"
              aria-describedby="json-description"
            />
            <div>
              <p className="font-semibold text-gray-900">JSON形式</p>
              <p id="json-description" className="text-sm text-gray-600">
                構造化されたデータ。他のツールとの連携に最適
              </p>
            </div>
          </label>
          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="radio"
              name="exportFormat"
              value="csv"
              checked={exportFormat === 'csv'}
              onChange={() => setExportFormat('csv')}
              className="w-4 h-4 text-blue-600 focus:ring-2 focus:ring-blue-500"
              aria-describedby="csv-description"
            />
            <div>
              <p className="font-semibold text-gray-900">CSV形式</p>
              <p id="csv-description" className="text-sm text-gray-600">
                表形式のデータ。Excelなどで開いて分析可能
              </p>
            </div>
          </label>
        </div>
      </fieldset>

      {/* 選択されたセッション情報 */}
      {selectedSession ? (
        <div className="bg-white border border-gray-200 rounded-lg p-6 mb-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">エクスポート対象セッション</h2>
          <div className="space-y-2">
            <p className="text-sm text-gray-700">
              <span className="font-semibold">セッションID:</span>{' '}
              {selectedSession.sessionId.substring(0, 8)}
            </p>
            <p className="text-sm text-gray-700">
              <span className="font-semibold">開始時刻:</span>{' '}
              {new Date(selectedSession.startTime).toLocaleString('ja-JP')}
            </p>
            <p className="text-sm text-gray-700">
              <span className="font-semibold">終了時刻:</span>{' '}
              {new Date(selectedSession.endTime).toLocaleString('ja-JP')}
            </p>
            <p className="text-sm text-gray-700">
              <span className="font-semibold">品質スコア:</span> {selectedSession.qualityScore}
            </p>
          </div>
        </div>
      ) : (
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-6 mb-6">
          <p className="text-yellow-700 font-semibold">セッションが選択されていません</p>
          <p className="text-sm text-yellow-600 mt-1">
            セッション履歴から1つ選択してください
          </p>
        </div>
      )}

      {/* エクスポートボタン */}
      <div className="flex justify-center">
        <button
          onClick={() => {
            void handleExport();
          }}
          disabled={isExporting || selectedSessionIds.length === 0}
          className="px-8 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-semibold"
          aria-label="データをエクスポート"
        >
          {isExporting ? (
            <span className="flex items-center gap-2">
              <div className="inline-block animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent" />
              エクスポート中...
            </span>
          ) : (
            `${exportFormat.toUpperCase()}形式でエクスポート`
          )}
        </button>
      </div>

      {/* 使用方法 */}
      <div className="mt-8 bg-gray-50 border border-gray-200 rounded-lg p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-3">使用方法</h2>
        <ol className="list-decimal list-inside space-y-2 text-sm text-gray-700">
          <li>セッション履歴からエクスポートしたいセッションを選択</li>
          <li>エクスポート形式（JSON/CSV）を選択</li>
          <li>「エクスポート」ボタンをクリック</li>
          <li>保存先を指定してファイルを保存</li>
        </ol>
      </div>
    </div>
  );
}
