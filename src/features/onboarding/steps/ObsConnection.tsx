import { ObsConnectionPanel } from '../../obs/ObsConnectionPanel';
import { useObsStore } from '../../../stores/obsStore';

/**
 * オンボーディング Step 2: OBS接続設定（必須）
 *
 * 既存のObsConnectionPanelを活用
 */
export function ObsConnection() {
  const { connectionState } = useObsStore();
  const isConnected = connectionState === 'connected';

  return (
    <div className="space-y-6">
      <div className="text-center mb-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-2">OBSに接続しましょう</h2>
        <p className="text-gray-600">
          OBSのWebSocketサーバーに接続して設定を分析します
        </p>
      </div>

      {/* 接続ステップの説明 */}
      {!isConnected && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-4">
          <h3 className="font-semibold text-blue-900 mb-2">OBSの設定手順</h3>
          <ol className="text-sm text-blue-800 space-y-2 list-decimal list-inside">
            <li>OBSを起動します</li>
            <li>メニューから「ツール」→「WebSocketサーバー設定」を開きます</li>
            <li>「WebSocketサーバーを有効にする」にチェックを入れます</li>
            <li>表示されたポート番号（通常は4455）を確認します</li>
            <li>パスワードが設定されている場合はメモしておきます</li>
          </ol>
        </div>
      )}

      {/* 接続パネル */}
      <ObsConnectionPanel />

      {/* 接続成功メッセージ */}
      {isConnected && (
        <div className="bg-green-50 border border-green-200 rounded-lg p-4 text-center">
          <div className="flex items-center justify-center gap-2 text-green-800">
            <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
              <path
                fillRule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                clipRule="evenodd"
              />
            </svg>
            <span className="font-semibold">OBSに接続しました！</span>
          </div>
          <p className="text-sm text-green-700 mt-2">次のステップに進みましょう</p>
        </div>
      )}
    </div>
  );
}
