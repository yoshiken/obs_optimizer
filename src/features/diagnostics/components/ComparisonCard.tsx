import type { ObsSetting } from '../../../types/commands';

// ========================================
// 型定義
// ========================================

interface ComparisonCardProps {
  setting: ObsSetting;
  selected: boolean;
  onToggle: (key: string) => void;
}

// ========================================
// ヘルパー関数
// ========================================

/** 重要度ラベル */
function getPriorityLabel(priority: ObsSetting['priority']): string {
  const labels = {
    critical: '必須',
    recommended: '推奨',
    optional: '任意',
  };
  return labels[priority];
}

/** 重要度バッジのスタイル */
function getPriorityBadgeClass(priority: ObsSetting['priority']): string {
  switch (priority) {
    case 'critical':
      return 'bg-red-100 text-red-800';
    case 'recommended':
      return 'bg-yellow-100 text-yellow-800';
    case 'optional':
      return 'bg-blue-100 text-blue-800';
  }
}

/** 値を表示用にフォーマット */
function formatValue(value: string | number | boolean): string {
  if (typeof value === 'boolean') {
    return value ? 'オン' : 'オフ';
  }
  if (typeof value === 'number') {
    return value.toLocaleString();
  }
  return String(value);
}

// ========================================
// コンポーネント
// ========================================

/**
 * 設定の比較カード
 *
 * 現在値vs推奨値の比較を表示し、適用するかどうかを選択できる
 *
 * @example
 * <ComparisonCard
 *   setting={setting}
 *   selected={selected}
 *   onToggle={handleToggle}
 * />
 */
export function ComparisonCard({ setting, selected, onToggle }: ComparisonCardProps) {
  return (
    <div
      className={`
        border-2 rounded-lg p-4 transition-all
        ${selected ? 'border-blue-500 bg-blue-50' : 'border-gray-200 bg-white'}
      `}
    >
      {/* ヘッダー */}
      <div className="flex items-start justify-between mb-3">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <h3 className="font-semibold text-gray-900">{setting.displayName}</h3>
            <span
              className={`px-2 py-0.5 text-xs font-medium rounded ${getPriorityBadgeClass(setting.priority)}`}
            >
              {getPriorityLabel(setting.priority)}
            </span>
          </div>
          <p className="text-sm text-gray-600">{setting.reason}</p>
        </div>

        {/* チェックボックス */}
        <label className="flex items-center cursor-pointer ml-3">
          <input
            type="checkbox"
            checked={selected}
            onChange={() => onToggle(setting.key)}
            className="w-5 h-5 text-blue-500 border-gray-300 rounded focus:ring-blue-500"
            aria-label={`${setting.displayName}を適用`}
          />
        </label>
      </div>

      {/* Before/After比較 */}
      <div className="grid grid-cols-2 gap-3">
        {/* 現在の値 */}
        <div className="bg-gray-50 rounded-lg p-3 border border-gray-200">
          <div className="text-xs text-gray-500 mb-1">現在の値</div>
          <div className="text-sm font-medium text-gray-900">
            {formatValue(setting.currentValue)}
          </div>
        </div>

        {/* 推奨値 */}
        <div className="bg-green-50 rounded-lg p-3 border border-green-200">
          <div className="text-xs text-green-700 mb-1">推奨値</div>
          <div className="text-sm font-medium text-green-900">
            {formatValue(setting.recommendedValue)}
          </div>
        </div>
      </div>
    </div>
  );
}
