import { useObsStore } from '../../stores/obsStore';

/**
 * 時間コード（ミリ秒）を HH:MM:SS 形式に変換
 */
function formatTimecode(ms: number | null): string {
  if (ms === null || ms < 0) return '--:--:--';

  const totalSeconds = Math.floor(ms / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return `${hours.toString().padStart(2, '0')}:${minutes
    .toString()
    .padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
}

/**
 * ビットレートを Mbps 形式に変換
 */
function formatBitrate(bps: number | null): string {
  if (bps === null || bps < 0) return '-- Mbps';
  const mbps = bps / 1_000_000;
  return `${mbps.toFixed(2)} Mbps`;
}

/**
 * OBSステータス表示コンポーネント
 *
 * 接続状態、配信/録画状態、現在のシーン、FPS、ドロップフレームを表示
 */
export function ObsStatusIndicator() {
  const { connectionState, status } = useObsStore();

  const isConnected = connectionState === 'connected';

  // 接続バッジの色
  const getConnectionBadgeClass = () => {
    if (!status?.connected && connectionState !== 'connected') {
      return 'bg-gray-500';
    }
    return 'bg-green-500';
  };

  // 配信バッジの色
  const getStreamingBadgeClass = () => {
    if (!status?.streaming) return 'bg-gray-400';
    return 'bg-red-500 animate-pulse';
  };

  // 録画バッジの色
  const getRecordingBadgeClass = () => {
    if (!status?.recording) return 'bg-gray-400';
    return 'bg-red-500 animate-pulse';
  };

  return (
    <div className="bg-white rounded-lg shadow-md p-6">
      <h3 className="text-lg font-semibold text-gray-800 mb-4">OBSステータス</h3>

      {!isConnected ? (
        <div className="text-center py-8 text-gray-500">
          <p>OBSに接続されていません</p>
        </div>
      ) : (
        <div className="space-y-4" role="status" aria-live="polite">
          {/* 接続状態とバージョン情報 */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span
                className={`w-3 h-3 rounded-full ${getConnectionBadgeClass()}`}
                aria-hidden="true"
              />
              <span className="text-sm font-medium text-gray-700">接続済み</span>
            </div>
            {status?.obsVersion && (
              <span className="text-xs text-gray-500">
                OBS {status.obsVersion} / WS {status.websocketVersion}
              </span>
            )}
          </div>

          {/* 配信/録画インジケーター */}
          <div className="grid grid-cols-2 gap-4">
            {/* 配信状態 */}
            <div className="bg-gray-50 rounded-md p-3">
              <div className="flex items-center gap-2 mb-2">
                <span
                  className={`w-2.5 h-2.5 rounded-full ${getStreamingBadgeClass()}`}
                  aria-hidden="true"
                />
                <span className="text-sm font-medium text-gray-700">
                  {status?.streaming ? '配信中' : '配信停止'}
                </span>
              </div>
              {status?.streaming && (
                <div className="text-xs text-gray-500 space-y-1">
                  <div>時間: {formatTimecode(status.streamTimecode)}</div>
                  <div>ビットレート: {formatBitrate(status.streamBitrate)}</div>
                </div>
              )}
            </div>

            {/* 録画状態 */}
            <div className="bg-gray-50 rounded-md p-3">
              <div className="flex items-center gap-2 mb-2">
                <span
                  className={`w-2.5 h-2.5 rounded-full ${getRecordingBadgeClass()}`}
                  aria-hidden="true"
                />
                <span className="text-sm font-medium text-gray-700">
                  {status?.recording ? '録画中' : '録画停止'}
                </span>
              </div>
              {status?.recording && (
                <div className="text-xs text-gray-500 space-y-1">
                  <div>時間: {formatTimecode(status.recordTimecode)}</div>
                  <div>ビットレート: {formatBitrate(status.recordBitrate)}</div>
                </div>
              )}
            </div>
          </div>

          {/* 現在のシーン */}
          <div className="bg-gray-50 rounded-md p-3">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-700">現在のシーン</span>
              <span className="text-sm text-gray-800 font-mono">
                {status?.currentScene ?? '不明'}
              </span>
            </div>
          </div>

          {/* パフォーマンス統計 */}
          <div className="border-t pt-4">
            <h4 className="text-sm font-medium text-gray-700 mb-3">パフォーマンス</h4>
            <div className="grid grid-cols-3 gap-4">
              {/* FPS */}
              <div className="text-center">
                <div className="text-2xl font-bold text-gray-800">
                  {status?.fps?.toFixed(1) ?? '--'}
                </div>
                <div className="text-xs text-gray-500">FPS</div>
              </div>

              {/* レンダリングドロップ */}
              <div className="text-center">
                <div
                  className={`text-2xl font-bold ${
                    (status?.renderDroppedFrames ?? 0) > 0
                      ? 'text-yellow-600'
                      : 'text-gray-800'
                  }`}
                >
                  {status?.renderDroppedFrames ?? '--'}
                </div>
                <div className="text-xs text-gray-500">レンダードロップ</div>
              </div>

              {/* 出力ドロップ */}
              <div className="text-center">
                <div
                  className={`text-2xl font-bold ${
                    (status?.outputDroppedFrames ?? 0) > 0
                      ? 'text-red-600'
                      : 'text-gray-800'
                  }`}
                >
                  {status?.outputDroppedFrames ?? '--'}
                </div>
                <div className="text-xs text-gray-500">出力ドロップ</div>
              </div>
            </div>
          </div>

          {/* 仮想カメラ */}
          {status?.virtualCamActive && (
            <div className="flex items-center gap-2 text-sm text-gray-600">
              <svg
                className="w-4 h-4 text-green-500"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path d="M2 6a2 2 0 012-2h6a2 2 0 012 2v8a2 2 0 01-2 2H4a2 2 0 01-2-2V6zM14.553 7.106A1 1 0 0014 8v4a1 1 0 00.553.894l2 1A1 1 0 0018 13V7a1 1 0 00-1.447-.894l-2 1z" />
              </svg>
              <span>仮想カメラ有効</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
