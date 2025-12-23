import { useCallback, useEffect, useRef, useState } from 'react';
import { useObsStore } from '../../stores/obsStore';
import { ConfirmDialog } from '../../components/common/ConfirmDialog';

/**
 * 時間コード（ミリ秒）を HH:MM:SS 形式に変換
 */
function formatTimecode(ms: number | null): string {
  if (ms === null || ms < 0) {return '00:00:00';}

  const totalSeconds = Math.floor(ms / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return `${hours.toString().padStart(2, '0')}:${minutes
    .toString()
    .padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
}

/**
 * OBS配信・録画コントロール
 *
 * 配信と録画の開始/停止ボタン、経過時間の表示
 */
export function ObsStreamControls() {
  const {
    connectionState,
    status,
    loading,
    startStreaming,
    stopStreaming,
    startRecording,
    stopRecording,
  } = useObsStore();

  // 録画停止後の出力パスを表示するための状態
  const [lastRecordingPath, setLastRecordingPath] = useState<string | null>(null);
  const [showRecordingPathNotice, setShowRecordingPathNotice] = useState(false);
  // 通知タイマーのref（メモリリーク防止用）
  const noticeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // 確認ダイアログの状態
  const [showStreamingStopDialog, setShowStreamingStopDialog] = useState(false);
  const [showRecordingStopDialog, setShowRecordingStopDialog] = useState(false);

  // クリーンアップ: コンポーネントアンマウント時にタイマーをクリア
  useEffect(() => {
    return () => {
      if (noticeTimerRef.current) {
        clearTimeout(noticeTimerRef.current);
      }
    };
  }, []);

  const isConnected = connectionState === 'connected';
  const isStreaming = status?.streaming ?? false;
  const isRecording = status?.recording ?? false;

  // 配信開始/停止ハンドラ
  const handleStreamingToggle = useCallback(() => {
    if (isStreaming) {
      // 配信停止時は確認ダイアログを表示
      setShowStreamingStopDialog(true);
    } else {
      // 配信開始はそのまま実行
      startStreaming().catch(() => {
        // エラーはストアで処理される
      });
    }
  }, [isStreaming, startStreaming]);

  // 配信停止の確認後の処理
  const handleConfirmStreamingStop = useCallback(async () => {
    setShowStreamingStopDialog(false);
    try {
      await stopStreaming();
    } catch {
      // エラーはストアで処理される
    }
  }, [stopStreaming]);

  // 録画開始/停止ハンドラ
  const handleRecordingToggle = useCallback(() => {
    if (isRecording) {
      // 録画停止時は確認ダイアログを表示
      setShowRecordingStopDialog(true);
    } else {
      // 録画開始はそのまま実行
      startRecording().then(() => {
        setLastRecordingPath(null);
        setShowRecordingPathNotice(false);
        // 録画開始時もタイマーをクリア
        if (noticeTimerRef.current) {
          clearTimeout(noticeTimerRef.current);
          noticeTimerRef.current = null;
        }
      }).catch(() => {
        // エラーはストアで処理される
      });
    }
  }, [isRecording, startRecording]);

  // 録画停止の確認後の処理
  const handleConfirmRecordingStop = useCallback(async () => {
    setShowRecordingStopDialog(false);
    try {
      const outputPath = await stopRecording();
      setLastRecordingPath(outputPath);
      setShowRecordingPathNotice(true);
      // 既存のタイマーをクリアして新しいタイマーを設定
      if (noticeTimerRef.current) {
        clearTimeout(noticeTimerRef.current);
      }
      // 5秒後に通知を非表示
      noticeTimerRef.current = setTimeout(() => {
        setShowRecordingPathNotice(false);
        noticeTimerRef.current = null;
      }, 5000);
    } catch {
      // エラーはストアで処理される
    }
  }, [stopRecording]);

  if (!isConnected) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md dark:shadow-gray-900/50 p-6">
        <h3 className="text-lg font-semibold text-gray-800 dark:text-gray-100 mb-4">配信・録画コントロール</h3>
        <div className="text-center py-8 text-gray-600 dark:text-gray-300">
          <p>OBSに接続されていません</p>
        </div>
      </div>
    );
  }

  return (
    <>
      {/* 配信停止確認ダイアログ */}
      <ConfirmDialog
        isOpen={showStreamingStopDialog}
        title="配信を停止しますか？"
        message="配信を停止すると、視聴者への配信が終了します。よろしいですか？"
        confirmText="停止する"
        cancelText="キャンセル"
        confirmVariant="danger"
        onConfirm={() => void handleConfirmStreamingStop()}
        onCancel={() => setShowStreamingStopDialog(false)}
      />

      {/* 録画停止確認ダイアログ */}
      <ConfirmDialog
        isOpen={showRecordingStopDialog}
        title="録画を停止しますか？"
        message="録画を停止すると、ファイルが保存されます。よろしいですか？"
        confirmText="停止する"
        cancelText="キャンセル"
        confirmVariant="danger"
        onConfirm={() => void handleConfirmRecordingStop()}
        onCancel={() => setShowRecordingStopDialog(false)}
      />

      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md dark:shadow-gray-900/50 p-6 card-interactive">
        <h3 className="text-lg font-semibold text-gray-800 dark:text-gray-100 mb-4">配信・録画コントロール</h3>

      {/* 録画完了通知 */}
      {showRecordingPathNotice && lastRecordingPath && (
        <div className="mb-4 p-3 bg-green-100 dark:bg-green-900/30 border border-green-300 dark:border-green-700 rounded-md">
          <div className="flex items-center justify-between">
            <div>
              <span className="text-sm text-green-700 dark:text-green-300 font-medium">録画が保存されました</span>
              <p className="text-xs text-green-600 dark:text-green-300 mt-1 font-mono truncate" title={lastRecordingPath}>
                {lastRecordingPath}
              </p>
            </div>
            <button
              onClick={() => setShowRecordingPathNotice(false)}
              className="text-green-500 hover:text-green-700 dark:hover:text-green-300 text-sm"
              aria-label="通知を閉じる"
            >
              x
            </button>
          </div>
        </div>
      )}

      <div className="grid grid-cols-2 gap-6">
        {/* 配信コントロール */}
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-gray-700 dark:text-gray-200">配信</span>
            {isStreaming && (
              <span className="flex items-center gap-1 text-xs text-red-600 dark:text-red-300">
                <span className="w-2 h-2 bg-red-500 rounded-full animate-pulse" />
                LIVE
              </span>
            )}
          </div>

          {/* 配信時間 */}
          <div className="text-center py-3 bg-gray-50 dark:bg-gray-700 rounded-md">
            <span className="text-3xl font-mono font-bold text-gray-800 dark:text-gray-100">
              {formatTimecode(status?.streamTimecode ?? null)}
            </span>
          </div>

          {/* 配信ボタン */}
          <button
            onClick={handleStreamingToggle}
            disabled={loading}
            className={`w-full px-4 py-3 rounded-md text-white font-medium transition-all duration-200 hover:scale-105 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100 ${
              isStreaming
                ? 'bg-red-500 hover:bg-red-600 hover:shadow-lg'
                : 'bg-green-500 hover:bg-green-600 hover:shadow-lg'
            }`}
          >
            {loading ? (
              <span className="flex items-center justify-center gap-2">
                <svg
                  className="w-5 h-5 animate-spin"
                  fill="none"
                  viewBox="0 0 24 24"
                >
                  <circle
                    className="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    strokeWidth="4"
                  />
                  <path
                    className="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  />
                </svg>
                処理中...
              </span>
            ) : isStreaming ? (
              <span className="flex items-center justify-center gap-2">
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fillRule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zM8 7a1 1 0 00-1 1v4a1 1 0 001 1h4a1 1 0 001-1V8a1 1 0 00-1-1H8z"
                    clipRule="evenodd"
                  />
                </svg>
                配信を停止
              </span>
            ) : (
              <span className="flex items-center justify-center gap-2">
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fillRule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z"
                    clipRule="evenodd"
                  />
                </svg>
                配信を開始
              </span>
            )}
          </button>
        </div>

        {/* 録画コントロール */}
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-gray-700 dark:text-gray-200">録画</span>
            {isRecording && (
              <span className="flex items-center gap-1 text-xs text-red-600 dark:text-red-300">
                <span className="w-2 h-2 bg-red-500 rounded-full animate-pulse" />
                REC
              </span>
            )}
          </div>

          {/* 録画時間 */}
          <div className="text-center py-3 bg-gray-50 dark:bg-gray-700 rounded-md">
            <span className="text-3xl font-mono font-bold text-gray-800 dark:text-gray-100">
              {formatTimecode(status?.recordTimecode ?? null)}
            </span>
          </div>

          {/* 録画ボタン */}
          <button
            onClick={handleRecordingToggle}
            disabled={loading}
            className={`w-full px-4 py-3 rounded-md text-white font-medium transition-all duration-200 hover:scale-105 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100 ${
              isRecording
                ? 'bg-red-500 hover:bg-red-600 hover:shadow-lg'
                : 'bg-blue-500 hover:bg-blue-600 hover:shadow-lg'
            }`}
          >
            {loading ? (
              <span className="flex items-center justify-center gap-2">
                <svg
                  className="w-5 h-5 animate-spin"
                  fill="none"
                  viewBox="0 0 24 24"
                >
                  <circle
                    className="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    strokeWidth="4"
                  />
                  <path
                    className="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  />
                </svg>
                処理中...
              </span>
            ) : isRecording ? (
              <span className="flex items-center justify-center gap-2">
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fillRule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zM8 7a1 1 0 00-1 1v4a1 1 0 001 1h4a1 1 0 001-1V8a1 1 0 00-1-1H8z"
                    clipRule="evenodd"
                  />
                </svg>
                録画を停止
              </span>
            ) : (
              <span className="flex items-center justify-center gap-2">
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M10 18a8 8 0 100-16 8 8 0 000 16zM9 9a1 1 0 112 0v2a1 1 0 11-2 0V9z" />
                  <path d="M10 4a6 6 0 100 12 6 6 0 000-12zm0 10a4 4 0 110-8 4 4 0 010 8z" />
                </svg>
                録画を開始
              </span>
            )}
          </button>
        </div>
      </div>
      </div>
    </>
  );
}
