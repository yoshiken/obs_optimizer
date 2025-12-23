import { useEffect, useState } from 'react';
import { useObsStore } from '../../../stores/obsStore';
import type { ObsConnectionParams } from '../../../types/commands';

/**
 * オンボーディング Step 3: OBS接続テスト
 *
 * localhost:4455 をデフォルトで接続テスト
 */
export function ConnectionTest() {
  const { connect, disconnect, connectionState, error } = useObsStore();
  const [connectionParams, setConnectionParams] = useState<ObsConnectionParams>({
    host: 'localhost',
    port: 4455,
    password: '',
  });
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [testAttempts, setTestAttempts] = useState(0);

  const isConnected = connectionState === 'connected';
  const isConnecting = connectionState === 'connecting';

  // 接続状態が変わったときにテスト試行回数をリセット
  useEffect(() => {
    if (isConnected) {
      setTestAttempts(0);
    }
  }, [isConnected]);

  const handleTestConnection = async () => {
    setTestAttempts((prev) => prev + 1);
    try {
      await connect(connectionParams);
    } catch {
      // エラーはストアで処理される
    }
  };

  const handleDisconnect = async () => {
    try {
      await disconnect();
    } catch {
      // エラーはストアで処理される
    }
  };

  return (
    <div className="space-y-6">
      <div className="text-center mb-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-2">
          OBS接続テスト
        </h2>
        <p className="text-gray-600">
          設定したWebSocketサーバーに接続できるか確認します
        </p>
      </div>

      {/* 接続ステータス */}
      <div
        className={`border-2 rounded-lg p-6 transition-colors ${
          isConnected
            ? 'border-green-300 bg-green-50'
            : error
            ? 'border-red-300 bg-red-50'
            : 'border-gray-200 bg-white'
        }`}
      >
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <div
              className={`w-4 h-4 rounded-full ${
                isConnected
                  ? 'bg-green-500 animate-pulse'
                  : isConnecting
                  ? 'bg-yellow-500 animate-pulse'
                  : 'bg-gray-300'
              }`}
              aria-label={
                isConnected ? '接続中' : isConnecting ? '接続試行中' : '未接続'
              }
            />
            <span className="font-semibold text-gray-900">
              {isConnected
                ? '接続成功'
                : isConnecting
                ? '接続中...'
                : '未接続'}
            </span>
          </div>
          {isConnected && (
            <button
              onClick={() => void handleDisconnect()}
              className="text-sm text-gray-600 hover:text-gray-900 underline"
            >
              切断
            </button>
          )}
        </div>

        {/* 接続成功メッセージ */}
        {isConnected && (
          <div className="text-green-800">
            <p className="text-sm font-medium mb-2">
              OBS WebSocketサーバーに正常に接続しました！
            </p>
            <p className="text-xs text-green-700">
              次のステップに進んで、セットアップを完了してください。
            </p>
          </div>
        )}

        {/* エラーメッセージ */}
        {error && !isConnected && (
          <div className="text-red-800">
            <p className="text-sm font-medium mb-2">接続に失敗しました</p>
            <p className="text-xs text-red-700 mb-3">{error}</p>
            <ul className="text-xs text-red-700 space-y-1 list-disc list-inside">
              <li>OBSが起動していることを確認してください</li>
              <li>WebSocketサーバーが有効になっていることを確認してください</li>
              <li>ポート番号とパスワードが正しいか確認してください</li>
              <li>ファイアウォールでブロックされていないか確認してください</li>
            </ul>
          </div>
        )}
      </div>

      {/* 接続設定フォーム */}
      {!isConnected && (
        <div className="bg-white border border-gray-200 rounded-lg p-6">
          <h3 className="font-semibold text-gray-900 mb-4">接続設定</h3>

          {/* 基本設定 */}
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label
                  htmlFor="host"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  ホスト
                </label>
                <input
                  id="host"
                  type="text"
                  value={connectionParams.host}
                  onChange={(e) =>
                    setConnectionParams({ ...connectionParams, host: e.target.value })
                  }
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="localhost"
                  disabled={isConnecting}
                />
              </div>
              <div>
                <label
                  htmlFor="port"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  ポート
                </label>
                <input
                  id="port"
                  type="number"
                  value={connectionParams.port}
                  onChange={(e) =>
                    setConnectionParams({
                      ...connectionParams,
                      port: parseInt(e.target.value) || 4455,
                    })
                  }
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="4455"
                  disabled={isConnecting}
                />
              </div>
            </div>

            {/* 詳細設定トグル */}
            <button
              onClick={() => setShowAdvanced(!showAdvanced)}
              className="text-sm text-blue-600 hover:text-blue-700 flex items-center gap-1"
              type="button"
            >
              {showAdvanced ? '詳細設定を隠す' : '詳細設定を表示'}
              <svg
                className={`w-4 h-4 transition-transform ${
                  showAdvanced ? 'rotate-180' : ''
                }`}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M19 9l-7 7-7-7"
                />
              </svg>
            </button>

            {/* パスワード入力（詳細設定） */}
            {showAdvanced && (
              <div className="pt-2 border-t border-gray-200">
                <label
                  htmlFor="password"
                  className="block text-sm font-medium text-gray-700 mb-1"
                >
                  パスワード（オプション）
                </label>
                <input
                  id="password"
                  type="password"
                  value={connectionParams.password || ''}
                  onChange={(e) =>
                    setConnectionParams({
                      ...connectionParams,
                      password: e.target.value,
                    })
                  }
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="WebSocketサーバーのパスワード"
                  disabled={isConnecting}
                />
                <p className="text-xs text-gray-500 mt-1">
                  パスワードを設定していない場合は空欄のままにしてください
                </p>
              </div>
            )}
          </div>

          {/* 接続テストボタン */}
          <button
            onClick={() => void handleTestConnection()}
            disabled={isConnecting}
            className="w-full mt-6 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors font-medium"
          >
            {isConnecting ? '接続中...' : '接続テスト'}
          </button>
        </div>
      )}

      {/* ヒント */}
      {!isConnected && testAttempts > 2 && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div className="flex items-start gap-3">
            <svg
              className="w-5 h-5 text-blue-600 flex-shrink-0 mt-0.5"
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path
                fillRule="evenodd"
                d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
                clipRule="evenodd"
              />
            </svg>
            <div className="text-sm text-blue-800">
              <p className="font-semibold mb-1">うまく接続できませんか？</p>
              <p className="mb-2">以下を確認してみてください：</p>
              <ul className="space-y-1 list-disc list-inside">
                <li>前のステップに戻って、WebSocket設定を再確認</li>
                <li>OBSを再起動してみる</li>
                <li>他のアプリケーションがポート4455を使用していないか確認</li>
              </ul>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
