import { useCallback, useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useObsStore } from '../../stores/obsStore';
import type { ObsConnectionParams, SavedConnectionInfo } from '../../types/commands';
import { translateError } from '../../utils/errorTranslation';

/** ポート番号のバリデーション（Backend側と統一: 1024-65535） */
function validatePort(value: string): { valid: boolean; port: number; error?: string } {
  const trimmed = value.trim();
  if (trimmed === '') {
    return { valid: false, port: 4455, error: 'ポート番号を入力してください' };
  }
  const num = parseInt(trimmed, 10);
  if (isNaN(num)) {
    return { valid: false, port: 4455, error: '数値を入力してください' };
  }
  // Well-known ports（1-1023）はシステム予約のため除外（Backend側と統一）
  if (num < 1024 || num > 65535) {
    return { valid: false, port: num, error: 'ポートは1024〜65535の範囲で入力してください' };
  }
  return { valid: true, port: num };
}

/**
 * OBS接続設定パネル
 *
 * ホスト、ポート、パスワードを入力してOBSに接続/切断する
 */
export function ObsConnectionPanel() {
  const {
    connectionState,
    loading,
    error,
    lastConnectionParams,
    connect,
    disconnect,
    clearError,
  } = useObsStore();

  // フォーム状態（前回の接続設定があれば使用）
  const [host, setHost] = useState(lastConnectionParams?.host ?? 'localhost');
  // ポートは文字列として管理し、バリデーションを行う
  const [portInput, setPortInput] = useState(String(lastConnectionParams?.port ?? 4455));
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [savePassword, setSavePassword] = useState(false);

  // 保存された接続情報を読み込む
  useEffect(() => {
    const loadSavedConnection = async () => {
      try {
        const saved = await invoke<SavedConnectionInfo>('get_saved_connection');
        setHost(saved.host);
        setPortInput(String(saved.port));
        setSavePassword(saved.savePassword);
        // パスワードが保存されていれば復元
        if (saved.savedPassword) {
          setPassword(saved.savedPassword);
        }
      } catch (err) {
        console.warn('保存された接続情報の読み込みに失敗:', err);
      }
    };
    void loadSavedConnection();
  }, []);

  // ポートのバリデーション結果をメモ化
  const portValidation = useMemo(() => validatePort(portInput), [portInput]);

  // エラーメッセージを翻訳
  const translatedError = useMemo(() => {
    if (!error) {return null;}
    return translateError(error);
  }, [error]);

  const isConnected = connectionState === 'connected';
  const isConnecting = connectionState === 'connecting' || connectionState === 'reconnecting';
  // 接続ボタンの有効状態（ホストが空でなく、ポートが有効な場合）
  const canConnect = host.trim() !== '' && portValidation.valid;

  // 接続ハンドラ
  const handleConnect = useCallback(async () => {
    if (!portValidation.valid) {return;}

    const params: ObsConnectionParams = {
      host,
      port: portValidation.port,
      password: password || undefined,
      savePassword,
    };

    try {
      await connect(params);
      // パスワード保存が無効の場合のみパスワードをクリア
      if (!savePassword) {
        setPassword('');
      }
    } catch {
      // エラーはストアで処理される
    }
  }, [host, portValidation, password, savePassword, connect]);

  // 切断ハンドラ
  const handleDisconnect = useCallback(async () => {
    try {
      await disconnect();
    } catch {
      // エラーはストアで処理される
    }
  }, [disconnect]);

  // 接続状態に応じたバッジの色
  const getStatusBadgeClass = () => {
    switch (connectionState) {
      case 'connected':
        return 'bg-green-500';
      case 'connecting':
      case 'reconnecting':
        return 'bg-yellow-500';
      case 'error':
        return 'bg-red-500';
      default:
        return 'bg-gray-500';
    }
  };

  // 接続状態のラベル
  const getStatusLabel = () => {
    switch (connectionState) {
      case 'connected':
        return '接続中';
      case 'connecting':
        return '接続中...';
      case 'reconnecting':
        return '再接続中...';
      case 'error':
        return 'エラー';
      default:
        return '未接続';
    }
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md dark:shadow-gray-900/50 p-6 card-interactive">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-800 dark:text-gray-100">OBS接続設定</h3>
        <div className="flex items-center gap-2">
          <span
            className={`w-3 h-3 rounded-full ${getStatusBadgeClass()}`}
            aria-hidden="true"
          />
          <span className="text-sm text-gray-600 dark:text-gray-300">{getStatusLabel()}</span>
        </div>
      </div>

      {/* エラー表示 */}
      {translatedError && (
        <div
          className="mb-4 p-3 bg-red-100 dark:bg-red-950/50 border border-red-300 dark:border-red-700 rounded-md"
          role="alert"
          aria-live="assertive"
        >
          <div className="space-y-1">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-red-700 dark:text-red-200">
                {translatedError.message}
              </span>
              <button
                onClick={clearError}
                className="text-red-500 hover:text-red-700 dark:hover:text-red-300 text-sm"
                aria-label="エラーを閉じる"
              >
                x
              </button>
            </div>
            {translatedError.hint && (
              <p className="text-xs text-red-600 dark:text-red-200">{translatedError.hint}</p>
            )}
          </div>
        </div>
      )}

      {/* 接続フォーム */}
      <div className="space-y-4">
        {/* ホスト入力 */}
        <div>
          <label
            htmlFor="obs-host"
            className="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1"
          >
            ホスト
          </label>
          <input
            id="obs-host"
            type="text"
            value={host}
            onChange={(e) => setHost(e.target.value)}
            disabled={isConnected || isConnecting}
            placeholder="localhost"
            aria-describedby="obs-host-hint"
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100 dark:disabled:bg-gray-700 disabled:cursor-not-allowed bg-white dark:bg-gray-700 text-gray-800 dark:text-gray-100"
          />
          <p id="obs-host-hint" className="mt-1 text-xs text-gray-600 dark:text-gray-300">
            OBSが同じパソコンで動いている場合は「localhost」のままでOKです
          </p>
        </div>

        {/* ポート入力 */}
        <div>
          <label
            htmlFor="obs-port"
            className="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1"
          >
            ポート
          </label>
          <input
            id="obs-port"
            type="text"
            inputMode="numeric"
            pattern="[0-9]*"
            value={portInput}
            onChange={(e) => setPortInput(e.target.value)}
            disabled={isConnected || isConnecting}
            aria-invalid={!portValidation.valid && portInput !== ''}
            aria-describedby="obs-port-hint obs-port-error"
            className={`w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 disabled:bg-gray-100 dark:disabled:bg-gray-700 disabled:cursor-not-allowed bg-white dark:bg-gray-700 text-gray-800 dark:text-gray-100 ${
              !portValidation.valid && portInput !== ''
                ? 'border-red-300 dark:border-red-700 focus:ring-red-500'
                : 'border-gray-300 dark:border-gray-600 focus:ring-blue-500'
            }`}
          />
          {/* バリデーションエラー表示 */}
          {!portValidation.valid && portInput !== '' ? (
            <p id="obs-port-error" className="mt-1 text-xs text-red-600 dark:text-red-300">
              {portValidation.error}
            </p>
          ) : (
            <p id="obs-port-hint" className="mt-1 text-xs text-gray-600 dark:text-gray-300">
              OBSの設定 → WebSocketサーバーで確認できます（通常は4455）
            </p>
          )}
        </div>

        {/* パスワード入力 */}
        <div>
          <label
            htmlFor="obs-password"
            className="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1"
          >
            パスワード（オプション）
          </label>
          <div className="relative">
            <input
              id="obs-password"
              type={showPassword ? 'text' : 'password'}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              disabled={isConnected || isConnecting}
              placeholder="設定している場合のみ入力"
              className="w-full px-3 py-2 pr-10 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100 dark:disabled:bg-gray-700 disabled:cursor-not-allowed bg-white dark:bg-gray-700 text-gray-800 dark:text-gray-100"
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-600 dark:text-gray-300 hover:text-gray-700 dark:hover:text-gray-200"
              aria-label={showPassword ? 'パスワードを隠す' : 'パスワードを表示'}
            >
              {showPassword ? (
                <svg
                  className="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21"
                  />
                </svg>
              ) : (
                <svg
                  className="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                  />
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                  />
                </svg>
              )}
            </button>
          </div>
        </div>

        {/* パスワード保存チェックボックス */}
        <div className="flex items-center gap-2">
          <input
            id="save-password"
            type="checkbox"
            checked={savePassword}
            onChange={(e) => setSavePassword(e.target.checked)}
            disabled={isConnected || isConnecting}
            className="w-4 h-4 text-blue-600 bg-gray-100 dark:bg-gray-700 border-gray-300 dark:border-gray-600 rounded focus:ring-blue-500 dark:focus:ring-blue-600 focus:ring-2 disabled:opacity-50 disabled:cursor-not-allowed"
          />
          <label
            htmlFor="save-password"
            className="text-sm text-gray-700 dark:text-gray-200"
          >
            パスワードを保存する
          </label>
        </div>

        {/* 接続/切断ボタン */}
        <div className="pt-2">
          {isConnected ? (
            <button
              onClick={() => void handleDisconnect()}
              disabled={loading}
              className="w-full px-4 py-2 bg-red-500 text-white rounded-md hover:bg-red-600 hover:shadow-lg disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100 transition-all duration-200 hover:scale-105 active:scale-95"
            >
              {loading ? '切断中...' : '切断'}
            </button>
          ) : (
            <button
              onClick={() => void handleConnect()}
              disabled={loading || isConnecting || !canConnect}
              className="w-full px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 hover:shadow-lg disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100 transition-all duration-200 hover:scale-105 active:scale-95"
            >
              {isConnecting ? '接続中...' : '接続'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
