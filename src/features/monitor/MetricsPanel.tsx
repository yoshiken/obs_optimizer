import { useEffect, useRef, useState } from 'react';
import { useMetricsStore } from '../../stores/metricsStore';
import {
  getCpuSeverity,
  getGpuSeverity,
  getMemorySeverity,
  getSeverityColorClass,
} from '../../utils/severity';

/** バイト数を人間が読みやすい形式に変換 */
function formatBytes(bytes: number): string {
  if (bytes === 0) {return '0 B';}

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

  // メトリクス変化検知用の状態
  const [cpuUpdated, setCpuUpdated] = useState(false);
  const [memoryUpdated, setMemoryUpdated] = useState(false);
  const [gpuUpdated, setGpuUpdated] = useState(false);
  const [networkUpdated, setNetworkUpdated] = useState(false);

  // 前回のメトリクス値を保持
  const prevMetrics = useRef(metrics);

  useEffect(() => {
    // メトリクスのポーリング開始（1秒間隔）
    // startPollingはZustandストアで安定した参照を持つため、依存配列に含めても問題ない
    const stopPolling = startPolling(1000);
    return stopPolling;
  }, [startPolling]);

  // メトリクス変化を検知してアニメーショントリガー
  useEffect(() => {
    if (!metrics || !prevMetrics.current) {
      prevMetrics.current = metrics;
      return;
    }

    // CPU変化検知
    if (metrics.cpu.usagePercent !== prevMetrics.current.cpu.usagePercent) {
      setCpuUpdated(true);
      setTimeout(() => setCpuUpdated(false), 300);
    }

    // メモリ変化検知
    if (metrics.memory.usagePercent !== prevMetrics.current.memory.usagePercent) {
      setMemoryUpdated(true);
      setTimeout(() => setMemoryUpdated(false), 300);
    }

    // GPU変化検知
    if (metrics.gpu && prevMetrics.current.gpu &&
        metrics.gpu.usagePercent !== prevMetrics.current.gpu.usagePercent) {
      setGpuUpdated(true);
      setTimeout(() => setGpuUpdated(false), 300);
    }

    // ネットワーク変化検知
    if (metrics.network.uploadBytesPerSec !== prevMetrics.current.network.uploadBytesPerSec ||
        metrics.network.downloadBytesPerSec !== prevMetrics.current.network.downloadBytesPerSec) {
      setNetworkUpdated(true);
      setTimeout(() => setNetworkUpdated(false), 300);
    }

    prevMetrics.current = metrics;
  }, [metrics]);

  if (error) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md dark:shadow-gray-900/50 p-6">
        <div className="p-3 bg-red-100 dark:bg-red-900/30 border border-red-300 dark:border-red-700 rounded-md">
          <span className="text-sm text-red-700 dark:text-red-300">エラー: {error}</span>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md dark:shadow-gray-900/50 p-6 card-interactive">
      <h3 className="text-lg font-semibold text-gray-800 dark:text-gray-100 mb-4">システムメトリクス</h3>
      {loading && !metrics ? (
        <div className="text-center py-8 text-gray-600 dark:text-gray-300">
          <p>読み込み中...</p>
        </div>
      ) : metrics ? (
        <div className="space-y-4" role="status" aria-live="polite">
          {/* CPU使用率 */}
          <div className="bg-gray-50 dark:bg-gray-700 rounded-md p-4 transition-all duration-200 hover:bg-gray-100 dark:hover:bg-gray-600 hover:shadow-md dark:hover:shadow-gray-900/50">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-200">CPU</span>
              <span className="text-xs text-gray-600 dark:text-gray-300">({metrics.cpu.coreCount}コア)</span>
            </div>
            <div className="mt-2">
              <span
                className={`text-2xl font-bold ${getSeverityColorClass(
                  getCpuSeverity(metrics.cpu.usagePercent)
                )} ${cpuUpdated ? 'metric-update' : ''}`}
              >
                {metrics.cpu.usagePercent.toFixed(1)}%
              </span>
            </div>
          </div>

          {/* メモリ使用率 */}
          <div className="bg-gray-50 dark:bg-gray-700 rounded-md p-4 transition-all duration-200 hover:bg-gray-100 dark:hover:bg-gray-600 hover:shadow-md dark:hover:shadow-gray-900/50">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-200">メモリ</span>
              <span
                className={`text-xs font-medium ${getSeverityColorClass(
                  getMemorySeverity(metrics.memory.usagePercent)
                )} ${memoryUpdated ? 'metric-update' : ''}`}
              >
                ({metrics.memory.usagePercent.toFixed(1)}%)
              </span>
            </div>
            <div className={`mt-2 text-sm text-gray-700 dark:text-gray-200 ${memoryUpdated ? 'metric-update' : ''}`}>
              {formatBytes(metrics.memory.usedBytes)} / {formatBytes(metrics.memory.totalBytes)}
            </div>
          </div>

          {/* GPU使用率（利用可能な場合） */}
          {metrics.gpu && (
            <div className="bg-gray-50 dark:bg-gray-700 rounded-md p-4 transition-all duration-200 hover:bg-gray-100 dark:hover:bg-gray-600 hover:shadow-md dark:hover:shadow-gray-900/50">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium text-gray-700 dark:text-gray-200">GPU</span>
                <span className="text-xs text-gray-600 dark:text-gray-300">{metrics.gpu.name}</span>
              </div>
              <div className="mt-2">
                <span
                  className={`text-2xl font-bold ${getSeverityColorClass(
                    getGpuSeverity(metrics.gpu.usagePercent)
                  )} ${gpuUpdated ? 'metric-update' : ''}`}
                >
                  {metrics.gpu.usagePercent.toFixed(1)}%
                </span>
              </div>
            </div>
          )}

          {/* ネットワーク */}
          <div className="bg-gray-50 dark:bg-gray-700 rounded-md p-4 transition-all duration-200 hover:bg-gray-100 dark:hover:bg-gray-600 hover:shadow-md dark:hover:shadow-gray-900/50">
            <span className="text-sm font-medium text-gray-700 dark:text-gray-200 block mb-2">ネットワーク</span>
            <div className="space-y-1">
              <div className="flex items-center justify-between text-sm text-gray-700 dark:text-gray-200">
                <span className="text-gray-600 dark:text-gray-300">↑ アップロード</span>
                <span className={`font-mono ${networkUpdated ? 'metric-update' : ''}`}>
                  {formatSpeed(metrics.network.uploadBytesPerSec)}
                </span>
              </div>
              <div className="flex items-center justify-between text-sm text-gray-700 dark:text-gray-200">
                <span className="text-gray-600 dark:text-gray-300">↓ ダウンロード</span>
                <span className={`font-mono ${networkUpdated ? 'metric-update' : ''}`}>
                  {formatSpeed(metrics.network.downloadBytesPerSec)}
                </span>
              </div>
            </div>
          </div>
        </div>
      ) : null}
    </div>
  );
}
