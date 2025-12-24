import { useObsStore } from '../../stores/obsStore';

/**
 * OBSステータスバー - コンパクトな接続状態表示
 * 接続済みの場合にダッシュボードの上部に小さく表示する
 */
export function ObsStatusBar() {
  const { connectionState, status, disconnect } = useObsStore();

  const isConnected = connectionState === 'connected';

  if (!isConnected) {
    return null;
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 px-4 py-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
            <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
              OBS接続中
            </span>
          </div>
          {status?.currentScene && (
            <span className="text-sm text-gray-500 dark:text-gray-400">
              シーン: {status.currentScene}
            </span>
          )}
          {status?.streaming && (
            <span className="px-2 py-0.5 bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-200 text-xs rounded-full font-medium">
              配信中
            </span>
          )}
          {status?.recording && (
            <span className="px-2 py-0.5 bg-purple-100 dark:bg-purple-900/30 text-purple-800 dark:text-purple-200 text-xs rounded-full font-medium">
              録画中
            </span>
          )}
        </div>
        <button
          onClick={() => void disconnect()}
          className="text-sm text-gray-500 dark:text-gray-400 hover:text-red-600 dark:hover:text-red-400 transition-colors"
        >
          切断
        </button>
      </div>
    </div>
  );
}
