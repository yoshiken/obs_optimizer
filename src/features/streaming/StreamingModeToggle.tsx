import { useEffect, useState } from 'react';
import { useStreamingModeStore } from '../../stores/streamingModeStore';
import { useObsStore } from '../../stores/obsStore';

interface StreamingModeToggleProps {
  /** トグル変更時のコールバック */
  onChange?: (enabled: boolean) => void;
  /** エラー発生時のコールバック */
  onError?: (error: string) => void;
}

/**
 * 配信中モードトグルコンポーネント
 * 配信中の通知制御などを切り替えます
 */
export function StreamingModeToggle({ onChange, onError }: StreamingModeToggleProps) {
  const {
    isEnabled,
    isLoading,
    error,
    setEnabled,
    loadStreamingMode,
    initializeAutoMode,
    clearError,
  } = useStreamingModeStore();

  const { status } = useObsStore();
  const [isInitialized, setIsInitialized] = useState(false);

  // 初期化
  useEffect(() => {
    if (!isInitialized) {
      void loadStreamingMode();
      initializeAutoMode();
      setIsInitialized(true);
    }
  }, [isInitialized, loadStreamingMode, initializeAutoMode]);

  // エラーハンドリング
  useEffect(() => {
    if (error) {
      onError?.(error);
      clearError();
    }
  }, [error, onError, clearError]);

  // トグル処理
  const handleToggle = async () => {
    const newValue = !isEnabled;
    try {
      await setEnabled(newValue);
      onChange?.(newValue);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      onError?.(errorMessage);
    }
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
      <div className="flex items-center justify-between">
        <div className="flex-1">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-1">
            配信中モード
          </h3>
          <p className="text-sm text-gray-600 dark:text-gray-300">
            {status?.streaming
              ? 'OBS配信中のため自動的にONになっています'
              : '配信開始時に自動的にONになります'}
          </p>
        </div>

        {/* トグルスイッチ */}
        <button
          onClick={() => void handleToggle()}
          disabled={isLoading}
          className={`
            relative inline-flex h-8 w-14 items-center rounded-full
            transition-colors duration-200 ease-in-out
            focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2
            disabled:opacity-50 disabled:cursor-not-allowed
            ${isEnabled ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'}
          `}
          role="switch"
          aria-checked={isEnabled}
          aria-label="配信中モードの切り替え"
        >
          <span
            className={`
              inline-block h-6 w-6 transform rounded-full
              bg-white transition-transform duration-200 ease-in-out
              ${isEnabled ? 'translate-x-7' : 'translate-x-1'}
            `}
          />
        </button>
      </div>

      {/* 状態表示 */}
      <div className="mt-4 flex items-center gap-2">
        <div
          className={`w-3 h-3 rounded-full ${
            isEnabled
              ? 'bg-green-500 animate-pulse'
              : 'bg-gray-400 dark:bg-gray-600'
          }`}
          aria-hidden="true"
        />
        <span className="text-sm text-gray-600 dark:text-gray-300">
          {isEnabled ? '配信中モード有効' : '配信中モード無効'}
        </span>
      </div>

      {/* 機能説明 */}
      <div className="mt-4 p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
        <p className="text-xs text-gray-700 dark:text-gray-300">
          <strong className="font-semibold">配信中モードの機能:</strong>
        </p>
        <ul className="text-xs text-gray-600 dark:text-gray-400 mt-2 space-y-1 list-disc list-inside">
          <li>Windows通知を自動的にオフ</li>
          <li>システムリソースの優先割り当て</li>
          <li>バックグラウンドプロセスの制限</li>
        </ul>
      </div>

      {/* OBS配信状態との連動表示 */}
      {status?.streaming && (
        <div className="mt-3 p-3 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 bg-red-600 rounded-full animate-pulse" />
            <span className="text-sm font-medium text-green-800 dark:text-green-200">
              配信中
            </span>
          </div>
          {status.streamTimecode !== null && (
            <p className="text-xs text-green-700 dark:text-green-300 mt-1">
              配信時間: {formatTimecode(status.streamTimecode)}
            </p>
          )}
        </div>
      )}
    </div>
  );
}

// タイムコードのフォーマット (HH:MM:SS)
function formatTimecode(milliseconds: number): string {
  const totalSeconds = Math.floor(milliseconds / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return [hours, minutes, seconds]
    .map((n) => n.toString().padStart(2, '0'))
    .join(':');
}
