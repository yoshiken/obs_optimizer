import { useCallback } from 'react';
import { useObsStore } from '../../stores/obsStore';

/**
 * OBSシーン選択コンポーネント
 *
 * 利用可能なシーンをリスト表示し、クリックでシーンを切り替える
 */
export function ObsSceneSelector() {
  const {
    connectionState,
    status,
    scenes,
    loading,
    setCurrentScene,
  } = useObsStore();

  const isConnected = connectionState === 'connected';
  const currentScene = status?.currentScene ?? null;

  // シーン変更ハンドラ
  const handleSceneChange = useCallback(
    async (sceneName: string) => {
      if (sceneName === currentScene) return;
      try {
        await setCurrentScene(sceneName);
      } catch (e) {
        console.error('Scene change failed:', e);
      }
    },
    [currentScene, setCurrentScene]
  );

  // ドロップダウンからの変更ハンドラ
  const handleSelectChange = useCallback(
    (event: React.ChangeEvent<HTMLSelectElement>) => {
      handleSceneChange(event.target.value);
    },
    [handleSceneChange]
  );

  if (!isConnected) {
    return (
      <div className="bg-white rounded-lg shadow-md p-6">
        <h3 className="text-lg font-semibold text-gray-800 mb-4">シーン</h3>
        <div className="text-center py-8 text-gray-500">
          <p>OBSに接続されていません</p>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow-md p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-800">シーン</h3>
        {currentScene && (
          <span className="text-sm text-gray-500">
            現在: <span className="font-medium text-gray-700">{currentScene}</span>
          </span>
        )}
      </div>

      {scenes.length === 0 ? (
        <div className="text-center py-8 text-gray-500">
          <p>シーンが見つかりません</p>
        </div>
      ) : (
        <>
          {/* モバイル用ドロップダウン */}
          <div className="sm:hidden">
            <label htmlFor="scene-select" className="sr-only">
              シーンを選択
            </label>
            <select
              id="scene-select"
              value={currentScene ?? ''}
              onChange={handleSelectChange}
              disabled={loading}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {scenes.map((scene) => (
                <option key={scene} value={scene}>
                  {scene}
                </option>
              ))}
            </select>
          </div>

          {/* デスクトップ用グリッド表示 */}
          <div className="hidden sm:grid gap-2">
            {scenes.map((scene) => {
              const isActive = scene === currentScene;
              return (
                <button
                  key={scene}
                  onClick={() => handleSceneChange(scene)}
                  disabled={loading || isActive}
                  className={`flex items-center justify-between px-4 py-3 rounded-md border transition-all ${
                    isActive
                      ? 'bg-blue-50 border-blue-500 text-blue-700'
                      : 'bg-white border-gray-200 text-gray-700 hover:bg-gray-50 hover:border-gray-300'
                  } ${loading ? 'opacity-50 cursor-not-allowed' : ''}`}
                >
                  <span className="flex items-center gap-3">
                    {/* シーンアイコン */}
                    <svg
                      className={`w-5 h-5 ${isActive ? 'text-blue-500' : 'text-gray-400'}`}
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                      />
                    </svg>
                    <span className="font-medium">{scene}</span>
                  </span>

                  {/* アクティブインジケーター */}
                  {isActive && (
                    <span className="flex items-center gap-1 text-xs text-blue-600">
                      <span className="w-2 h-2 bg-blue-500 rounded-full" />
                      アクティブ
                    </span>
                  )}
                </button>
              );
            })}
          </div>
        </>
      )}

      {/* シーン数表示 */}
      {scenes.length > 0 && (
        <div className="mt-4 pt-4 border-t text-center">
          <span className="text-xs text-gray-500">
            {scenes.length} 個のシーン
          </span>
        </div>
      )}
    </div>
  );
}
