import { useEffect } from 'react';
import { useHistoryStore } from '../../stores/historyStore';

/**
 * 時系列グラフ
 * - CPU/GPU/フレームドロップ率の推移
 * - ズーム・パン機能
 *
 * NOTE: Rechartsがインストールされるまでのプレースホルダー実装
 * REQ-006の依存関係が解決されたら、実際のグラフコンポーネントに置き換える
 */
export function MetricsChart() {
  const { selectedSessionIds, metricsData, loadMetrics, isLoading } = useHistoryStore();

  useEffect(() => {
    // 選択されたセッションのメトリクスを読み込み
    if (selectedSessionIds.length > 0) {
      const sessionId = selectedSessionIds[0];
      const now = Date.now();
      const oneHourAgo = now - 60 * 60 * 1000; // 1時間前
      void loadMetrics(sessionId, oneHourAgo, now);
    }
  }, [selectedSessionIds, loadMetrics]);

  if (selectedSessionIds.length === 0) {
    return (
      <div className="max-w-6xl mx-auto p-6">
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-8 text-center">
          <p className="text-blue-700 text-lg font-semibold">メトリクスグラフ</p>
          <p className="text-blue-600 text-sm mt-2">
            グラフを表示するにはセッションを選択してください
          </p>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="max-w-6xl mx-auto p-6">
        <div className="text-center py-12" role="status" aria-live="polite">
          <div className="inline-block animate-spin rounded-full h-12 w-12 border-4 border-gray-300 border-t-blue-600" />
          <p className="mt-4 text-gray-600">メトリクスを読み込み中...</p>
        </div>
      </div>
    );
  }

  if (metricsData.length === 0) {
    return (
      <div className="max-w-6xl mx-auto p-6">
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-8 text-center">
          <p className="text-yellow-700 text-lg font-semibold">データなし</p>
          <p className="text-yellow-600 text-sm mt-2">
            選択されたセッションのメトリクスデータが見つかりません
          </p>
        </div>
      </div>
    );
  }

  // 統計情報を計算
  const cpuValues = metricsData.map((m) => m.system.cpu.usagePercent);
  const gpuValues = metricsData.map((m) => m.system.gpu?.usagePercent ?? 0);
  const frameDrops = metricsData.map((m) => m.obs.outputDroppedFrames ?? 0);

  const avgCpu = cpuValues.reduce((a, b) => a + b, 0) / cpuValues.length;
  const avgGpu = gpuValues.reduce((a, b) => a + b, 0) / gpuValues.length;
  const maxCpu = Math.max(...cpuValues);
  const maxGpu = Math.max(...gpuValues);
  const totalDrops = frameDrops[frameDrops.length - 1] - frameDrops[0];

  return (
    <div className="max-w-6xl mx-auto p-6">
      <h1 className="text-2xl font-bold text-gray-900 mb-6">メトリクス推移グラフ</h1>

      {/* プレースホルダーメッセージ */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-6 mb-6">
        <p className="text-blue-700 font-semibold mb-2">グラフ表示準備中</p>
        <p className="text-sm text-blue-600">
          Rechartsライブラリのインストール待ちです（REQ-006）。
          <br />
          現在は統計情報のみ表示しています。
        </p>
      </div>

      {/* 統計情報 */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <div className="bg-white border border-gray-200 rounded-lg p-4">
          <h3 className="text-sm font-semibold text-gray-600 mb-2">CPU使用率</h3>
          <div className="space-y-1">
            <p className="text-sm text-gray-700">
              平均: <span className="font-bold">{avgCpu.toFixed(1)}%</span>
            </p>
            <p className="text-sm text-gray-700">
              最大: <span className="font-bold">{maxCpu.toFixed(1)}%</span>
            </p>
          </div>
        </div>

        <div className="bg-white border border-gray-200 rounded-lg p-4">
          <h3 className="text-sm font-semibold text-gray-600 mb-2">GPU使用率</h3>
          <div className="space-y-1">
            <p className="text-sm text-gray-700">
              平均: <span className="font-bold">{avgGpu.toFixed(1)}%</span>
            </p>
            <p className="text-sm text-gray-700">
              最大: <span className="font-bold">{maxGpu.toFixed(1)}%</span>
            </p>
          </div>
        </div>

        <div className="bg-white border border-gray-200 rounded-lg p-4">
          <h3 className="text-sm font-semibold text-gray-600 mb-2">フレームドロップ</h3>
          <div className="space-y-1">
            <p className="text-sm text-gray-700">
              合計: <span className="font-bold">{totalDrops}</span>
            </p>
            <p className="text-sm text-gray-700">
              データポイント: <span className="font-bold">{metricsData.length}</span>
            </p>
          </div>
        </div>
      </div>

      {/* データテーブル（プレースホルダー） */}
      <div className="bg-white border border-gray-200 rounded-lg overflow-hidden">
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  時刻
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  CPU (%)
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  GPU (%)
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  ドロップフレーム
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  ビットレート (Mbps)
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {metricsData.slice(0, 20).map((metric, index) => (
                <tr key={index} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {new Date(metric.timestamp).toLocaleTimeString('ja-JP')}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {metric.system.cpu.usagePercent.toFixed(1)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {metric.system.gpu?.usagePercent.toFixed(1) ?? 'N/A'}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {metric.obs.outputDroppedFrames ?? 0}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {metric.obs.streamBitrate
                      ? (metric.obs.streamBitrate / 1000).toFixed(1)
                      : 'N/A'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        {metricsData.length > 20 && (
          <div className="bg-gray-50 px-6 py-3 text-sm text-gray-600 text-center">
            {metricsData.length - 20} 件のデータが省略されています
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * TODO: Rechartsインストール後に実装する内容
 *
 * import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
 *
 * - 時系列折れ線グラフ
 * - 複数メトリクスの同時表示
 * - ズーム機能（ReferenceArea使用）
 * - パン機能（ドラッグでスクロール）
 * - ツールチップで詳細表示
 * - 凡例クリックで系列の表示/非表示切り替え
 */
