import { useMemo, useState } from 'react';
import { type AlertSeverity, useAlertStore } from '../../stores/alertStore';

// ========================================
// 型定義
// ========================================

type FilterOption = 'all' | AlertSeverity;

// ========================================
// ヘルパー関数
// ========================================

/** 相対時間を表示（例: "3分前"） */
function getRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;

  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) {
    return `${days}日前`;
  }
  if (hours > 0) {
    return `${hours}時間前`;
  }
  if (minutes > 0) {
    return `${minutes}分前`;
  }
  return 'たった今';
}

/** 重要度ラベル */
function getSeverityLabel(severity: AlertSeverity): string {
  const labels: Record<AlertSeverity, string> = {
    critical: '重要',
    warning: '警告',
    info: '情報',
    tips: 'ヒント',
  };
  return labels[severity];
}

/** 重要度バッジのスタイル */
function getSeverityBadgeClass(severity: AlertSeverity): string {
  switch (severity) {
    case 'critical':
      return 'bg-red-100 text-red-800';
    case 'warning':
      return 'bg-yellow-100 text-yellow-800';
    case 'info':
      return 'bg-blue-100 text-blue-800';
    case 'tips':
      return 'bg-gray-100 text-gray-800';
  }
}

// ========================================
// コンポーネント
// ========================================

/**
 * 通知履歴パネル
 *
 * タイムライン表示、フィルタリング、既読/未読管理
 *
 * @example
 * <AlertHistory />
 */
export function AlertHistory() {
  const { alerts, dismissAlert, clearAll } = useAlertStore();
  const [filter, setFilter] = useState<FilterOption>('all');

  // フィルタリングされたアラート
  const filteredAlerts = useMemo(() => {
    if (filter === 'all') {
      return alerts;
    }
    return alerts.filter((alert) => alert.severity === filter);
  }, [alerts, filter]);

  const activeCount = alerts.filter((alert) => !alert.dismissed).length;
  const hasAlerts = alerts.length > 0;

  return (
    <div className="bg-white rounded-lg shadow-md p-6">
      {/* ヘッダー */}
      <div className="flex items-center justify-between mb-4">
        <div>
          <h3 className="text-lg font-semibold text-gray-800">通知履歴</h3>
          <p className="text-sm text-gray-500">
            {activeCount > 0 ? `${activeCount}件の未読通知` : '未読通知なし'}
          </p>
        </div>
        {hasAlerts && (
          <button
            onClick={clearAll}
            className="text-sm text-blue-600 hover:text-blue-800 font-medium"
            aria-label="すべてを既読にする"
          >
            すべて既読
          </button>
        )}
      </div>

      {/* フィルター */}
      <div className="flex gap-2 mb-4 border-b border-gray-200 pb-3">
        <FilterButton
          active={filter === 'all'}
          onClick={() => setFilter('all')}
          label="すべて"
          count={alerts.length}
        />
        <FilterButton
          active={filter === 'critical'}
          onClick={() => setFilter('critical')}
          label="重要"
          count={alerts.filter((a) => a.severity === 'critical').length}
        />
        <FilterButton
          active={filter === 'warning'}
          onClick={() => setFilter('warning')}
          label="警告"
          count={alerts.filter((a) => a.severity === 'warning').length}
        />
        <FilterButton
          active={filter === 'info'}
          onClick={() => setFilter('info')}
          label="情報"
          count={alerts.filter((a) => a.severity === 'info').length}
        />
        <FilterButton
          active={filter === 'tips'}
          onClick={() => setFilter('tips')}
          label="ヒント"
          count={alerts.filter((a) => a.severity === 'tips').length}
        />
      </div>

      {/* アラートリスト */}
      {filteredAlerts.length === 0 ? (
        <div className="text-center py-8 text-gray-500">
          {filter === 'all' ? '通知はありません' : `${getSeverityLabel(filter)}の通知はありません`}
        </div>
      ) : (
        <div className="space-y-3 max-h-96 overflow-y-auto">
          {filteredAlerts.map((alert) => (
            <div
              key={alert.id}
              className={`border rounded-lg p-4 transition-opacity ${
                alert.dismissed ? 'opacity-50 bg-gray-50' : 'bg-white'
              }`}
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  {/* バッジと時間 */}
                  <div className="flex items-center gap-2 mb-2">
                    <span
                      className={`px-2 py-1 text-xs font-medium rounded ${getSeverityBadgeClass(alert.severity)}`}
                    >
                      {getSeverityLabel(alert.severity)}
                    </span>
                    <span className="text-xs text-gray-500">{getRelativeTime(alert.timestamp)}</span>
                  </div>

                  {/* タイトルとメッセージ */}
                  <h4 className="text-sm font-semibold text-gray-800 mb-1">{alert.title}</h4>
                  <p className="text-sm text-gray-600">{alert.message}</p>
                </div>

                {/* 既読ボタン */}
                {!alert.dismissed && (
                  <button
                    onClick={() => dismissAlert(alert.id)}
                    className="ml-3 text-gray-400 hover:text-gray-600"
                    aria-label="既読にする"
                  >
                    <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                      <path
                        fillRule="evenodd"
                        d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                        clipRule="evenodd"
                      />
                    </svg>
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

// ========================================
// サブコンポーネント
// ========================================

interface FilterButtonProps {
  active: boolean;
  onClick: () => void;
  label: string;
  count: number;
}

function FilterButton({ active, onClick, label, count }: FilterButtonProps) {
  return (
    <button
      onClick={onClick}
      className={`px-3 py-1 text-sm font-medium rounded-md transition-colors ${
        active
          ? 'bg-blue-100 text-blue-700'
          : 'text-gray-600 hover:bg-gray-100'
      }`}
      aria-pressed={active}
    >
      {label} ({count})
    </button>
  );
}
