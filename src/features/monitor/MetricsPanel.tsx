import { useEffect } from 'react';
import { useMetricsStore } from '../../stores/metricsStore';
import {
  getCpuSeverity,
  getMemorySeverity,
  getGpuSeverity,
  getSeverityColorClass,
} from '../../utils/severity';

/** バイト数を人間が読みやすい形式に変換 */
function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';

  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const value = bytes / Math.pow(k, i);

  return value.toFixed(2) + ' ' + units[i];
}

/** 転送速度を人間が読みやすい形式に変換 */
function formatSpeed(bytesPerSec: number): string {
  return formatBytes(bytesPerSec) + '/s';
}

/**
 * システムメトリクスを表示するパネル（シンプル版）
 *
 * CPU、メモリ、ネットワークの基本情報を表示する
 */
export function MetricsPanel() {
  const { metrics, loading, error, startPolling } = useMetricsStore();

  useEffect(() => {
    // メトリクスのポーリング開始（1秒間隔）
    // startPollingはZustandストアで安定した参照を持つため、依存配列に含めても問題ない
    const stopPolling = startPolling(1000);
    return stopPolling;
  }, [startPolling]);

  if (error) {
    return (
      <div className="bg-white rounded-lg shadow-md p-6">
        <div className="p-3 bg-red-100 border border-red-300 rounded-md">
          <span className="text-sm text-red-700">エラー: {error}</span>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow-md p-6">
      <h3 className="text-lg font-semibold text-gray-800 mb-4">システムメトリクス</h3>
      {loading && !metrics ? (
        <div className="text-center py-8 text-gray-500">
          <p>読み込み中...</p>
        </div>
      ) : metrics ? (
        <div className="space-y-4" role="status" aria-live="polite">
          {/* CPU使用率 */}
          <div className="bg-gray-50 rounded-md p-4">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-700">CPU</span>
              <span className="text-xs text-gray-500">({metrics.cpu.coreCount}コア)</span>
            </div>
            <div className="mt-2">
              <span
                className={`text-2xl font-bold ${getSeverityColorClass(
                  getCpuSeverity(metrics.cpu.usagePercent)
                )}`}
              >
                {metrics.cpu.usagePercent.toFixed(1)}%
              </span>
            </div>
          </div>

          {/* メモリ使用率 */}
          <div className="bg-gray-50 rounded-md p-4">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-700">メモリ</span>
              <span
                className={`text-xs font-medium ${getSeverityColorClass(
                  getMemorySeverity(metrics.memory.usagePercent)
                )}`}
              >
                ({metrics.memory.usagePercent.toFixed(1)}%)
              </span>
            </div>
            <div className="mt-2 text-sm text-gray-700">
              {formatBytes(metrics.memory.usedBytes)} / {formatBytes(metrics.memory.totalBytes)}
            </div>
          </div>

          {/* GPU使用率（利用可能な場合） */}
          {metrics.gpu && (
            <div className="bg-gray-50 rounded-md p-4">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium text-gray-700">GPU</span>
                <span className="text-xs text-gray-500">{metrics.gpu.name}</span>
              </div>
              <div className="mt-2">
                <span
                  className={`text-2xl font-bold ${getSeverityColorClass(
                    getGpuSeverity(metrics.gpu.usagePercent)
                  )}`}
                >
                  {metrics.gpu.usagePercent.toFixed(1)}%
                </span>
              </div>
            </div>
          )}

          {/* ネットワーク */}
          <div className="bg-gray-50 rounded-md p-4">
            <span className="text-sm font-medium text-gray-700 block mb-2">ネットワーク</span>
            <div className="space-y-1">
              <div className="flex items-center justify-between text-sm text-gray-700">
                <span className="text-gray-500">↑ アップロード</span>
                <span className="font-mono">{formatSpeed(metrics.network.uploadBytesPerSec)}</span>
              </div>
              <div className="flex items-center justify-between text-sm text-gray-700">
                <span className="text-gray-500">↓ ダウンロード</span>
                <span className="font-mono">{formatSpeed(metrics.network.downloadBytesPerSec)}</span>
              </div>
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
}
