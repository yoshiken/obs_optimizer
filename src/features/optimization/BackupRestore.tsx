import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { BackupInfo } from '../../types/commands';

interface BackupRestoreProps {
  /** 復元完了時のコールバック */
  onRestored?: () => void;
  /** エラー発生時のコールバック */
  onError?: (error: string) => void;
}

/**
 * バックアップ一覧・復元コンポーネント
 * OBS設定のバックアップ管理機能を提供します
 */
export function BackupRestore({ onRestored, onError }: BackupRestoreProps) {
  const [backups, setBackups] = useState<BackupInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [confirmRestoreId, setConfirmRestoreId] = useState<string | null>(null);
  const [isRestoring, setIsRestoring] = useState(false);

  // バックアップ一覧の読み込み
  const loadBackups = useCallback(async () => {
    setIsLoading(true);
    try {
      const data = await invoke<BackupInfo[]>('get_backups');
      // 新しい順にソート
      const sorted = [...data].sort((a, b) => b.createdAt - a.createdAt);
      setBackups(sorted);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      onError?.(errorMessage);
    } finally {
      setIsLoading(false);
    }
  }, [onError]);

  // 初回読み込み
  useEffect(() => {
    void loadBackups();
  }, [loadBackups]);

  // 復元処理
  const handleRestore = async (id: string) => {
    setIsRestoring(true);
    try {
      await invoke('restore_backup', { id });
      onRestored?.();
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      onError?.(errorMessage);
    } finally {
      setIsRestoring(false);
      setConfirmRestoreId(null);
    }
  };

  // 日時のフォーマット
  const formatDate = (timestamp: number): string => {
    const date = new Date(timestamp);
    return date.toLocaleString('ja-JP', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="space-y-4">
      {/* ヘッダー */}
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          バックアップ
        </h3>
        <button
          onClick={() => void loadBackups()}
          disabled={isLoading}
          className="text-sm text-blue-600 dark:text-blue-400 hover:underline
                     disabled:opacity-50"
          aria-label="バックアップ一覧を再読み込み"
        >
          {isLoading ? '読み込み中...' : '更新'}
        </button>
      </div>

      {/* バックアップ一覧 */}
      {backups.length === 0 ? (
        <p className="text-gray-500 dark:text-gray-400 text-sm text-center py-8">
          {isLoading ? '読み込み中...' : 'バックアップがありません'}
        </p>
      ) : (
        <ul className="space-y-2" role="list" aria-label="バックアップ一覧">
          {backups.map((backup) => (
            <li
              key={backup.id}
              className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4
                         border border-gray-200 dark:border-gray-700"
            >
              <div className="flex justify-between items-start gap-4">
                <div className="flex-1 min-w-0">
                  <p className="font-medium text-gray-900 dark:text-white truncate">
                    {backup.description}
                  </p>
                  <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                    {formatDate(backup.createdAt)}
                  </p>
                </div>
                <button
                  onClick={() => setConfirmRestoreId(backup.id)}
                  disabled={isRestoring}
                  className="px-3 py-1 text-sm bg-blue-600 text-white rounded
                             hover:bg-blue-700 disabled:opacity-50
                             transition-colors whitespace-nowrap
                             focus:outline-none focus:ring-2 focus:ring-blue-500"
                  aria-label={`${backup.description}を復元`}
                >
                  復元
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}

      {/* 復元確認ダイアログ */}
      {confirmRestoreId && (
        <div
          className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
          role="dialog"
          aria-labelledby="restore-dialog-title"
          aria-modal="true"
        >
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
            <h2
              id="restore-dialog-title"
              className="text-xl font-bold mb-4 text-gray-900 dark:text-white"
            >
              バックアップを復元しますか?
            </h2>
            <p className="text-gray-600 dark:text-gray-300 mb-6">
              現在のOBS設定がこのバックアップの内容で上書きされます。
              復元前に現在の設定をバックアップすることをお勧めします。
            </p>

            <div className="flex gap-3 justify-end">
              <button
                onClick={() => setConfirmRestoreId(null)}
                disabled={isRestoring}
                className="px-4 py-2 text-gray-700 dark:text-gray-300
                           bg-gray-200 dark:bg-gray-700 rounded
                           hover:bg-gray-300 dark:hover:bg-gray-600
                           disabled:opacity-50 transition-colors"
                aria-label="キャンセル"
              >
                キャンセル
              </button>
              <button
                onClick={() => void handleRestore(confirmRestoreId)}
                disabled={isRestoring}
                className="px-4 py-2 bg-blue-600 text-white rounded
                           hover:bg-blue-700 disabled:opacity-50
                           transition-colors
                           focus:outline-none focus:ring-2 focus:ring-blue-500"
                aria-label="復元を確定"
              >
                {isRestoring ? '復元中...' : '復元'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
