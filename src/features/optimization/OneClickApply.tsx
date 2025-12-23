import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { OptimizationResult } from '../../types/commands';

interface OneClickApplyProps {
  /** 適用完了時のコールバック */
  onApplied?: (result: OptimizationResult) => void;
  /** 適用失敗時のコールバック */
  onError?: (error: string) => void;
}

/**
 * ワンクリック最適化適用コンポーネント
 * 推奨設定を一括でOBSに適用します
 */
export function OneClickApply({ onApplied, onError }: OneClickApplyProps) {
  const [isConfirmOpen, setIsConfirmOpen] = useState(false);
  const [isApplying, setIsApplying] = useState(false);
  const [result, setResult] = useState<OptimizationResult | null>(null);

  // 適用処理
  const handleApply = async () => {
    setIsApplying(true);
    setResult(null);

    try {
      // バックアップを先に作成
      await invoke('backup_current_settings');

      // 推奨設定を適用
      const applyResult = await invoke<OptimizationResult>('apply_recommended_settings');
      setResult(applyResult);

      if (applyResult.failedCount === 0) {
        onApplied?.(applyResult);
      } else {
        onError?.(`${applyResult.failedCount}件の設定適用に失敗しました`);
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      onError?.(errorMessage);
    } finally {
      setIsApplying(false);
      setIsConfirmOpen(false);
    }
  };

  return (
    <div className="space-y-4">
      {/* 適用ボタン */}
      <button
        onClick={() => setIsConfirmOpen(true)}
        disabled={isApplying}
        className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400
                   text-white font-bold py-3 px-6 rounded-lg
                   transition-colors duration-200
                   focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
        aria-label="推奨設定を適用"
      >
        {isApplying ? '適用中...' : '最適化を適用'}
      </button>

      {/* 確認ダイアログ */}
      {isConfirmOpen && (
        <div
          className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
          role="dialog"
          aria-labelledby="confirm-dialog-title"
          aria-modal="true"
        >
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
            <h2
              id="confirm-dialog-title"
              className="text-xl font-bold mb-4 text-gray-900 dark:text-white"
            >
              最適化を適用しますか?
            </h2>
            <p className="text-gray-600 dark:text-gray-300 mb-6">
              現在のOBS設定を自動的にバックアップしてから、推奨設定を適用します。
              いつでもバックアップから復元できます。
            </p>

            <div className="flex gap-3 justify-end">
              <button
                onClick={() => setIsConfirmOpen(false)}
                disabled={isApplying}
                className="px-4 py-2 text-gray-700 dark:text-gray-300
                           bg-gray-200 dark:bg-gray-700 rounded
                           hover:bg-gray-300 dark:hover:bg-gray-600
                           disabled:opacity-50 transition-colors"
                aria-label="キャンセル"
              >
                キャンセル
              </button>
              <button
                onClick={() => void handleApply()}
                disabled={isApplying}
                className="px-4 py-2 bg-blue-600 text-white rounded
                           hover:bg-blue-700 disabled:opacity-50
                           transition-colors
                           focus:outline-none focus:ring-2 focus:ring-blue-500"
                aria-label="適用を確定"
              >
                {isApplying ? '適用中...' : '適用'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 適用結果の表示 */}
      {result && (
        <div
          className={`p-4 rounded-lg ${
            result.failedCount === 0
              ? 'bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-100'
              : 'bg-yellow-100 dark:bg-yellow-900 text-yellow-800 dark:text-yellow-100'
          }`}
          role="status"
          aria-live="polite"
        >
          <p className="font-semibold mb-2">
            {result.failedCount === 0
              ? '最適化が完了しました'
              : '一部の設定適用に失敗しました'}
          </p>
          <p className="text-sm">
            適用: {result.appliedCount}件
            {result.failedCount > 0 && ` / 失敗: ${result.failedCount}件`}
          </p>

          {result.errors.length > 0 && (
            <ul className="mt-2 text-sm list-disc list-inside">
              {result.errors.map((error, index) => (
                <li key={index}>{error}</li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}
