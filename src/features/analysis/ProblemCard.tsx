import type { ProblemReport } from '../../types/commands';

interface ProblemCardProps {
  problem: ProblemReport;
}

/**
 * 個別の問題表示カード
 * - 問題のタイトル・説明
 * - 推奨アクション一覧
 * - 発生時刻
 */
export function ProblemCard({ problem }: ProblemCardProps) {
  // 重要度に応じた色
  const severityColors = {
    critical: 'border-red-500 bg-red-50',
    warning: 'border-yellow-500 bg-yellow-50',
    info: 'border-blue-500 bg-blue-50',
    tips: 'border-green-500 bg-green-50',
  };

  const severityLabels = {
    critical: '重大',
    warning: '警告',
    info: '情報',
    tips: 'ヒント',
  };

  const categoryLabels = {
    encoding: 'エンコーディング',
    network: 'ネットワーク',
    resource: 'リソース',
    settings: '設定',
  };

  const detectedTime = new Date(problem.detectedAt).toLocaleString('ja-JP');

  return (
    <div
      className={`border-l-4 p-4 rounded-lg shadow-sm ${severityColors[problem.severity]}`}
      role="article"
      aria-labelledby={`problem-title-${problem.id}`}
    >
      {/* ヘッダー */}
      <div className="flex items-start justify-between mb-2">
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-1">
            <span
              className="px-2 py-0.5 text-xs font-semibold rounded"
              style={{
                backgroundColor:
                  problem.severity === 'critical'
                    ? '#dc2626'
                    : problem.severity === 'warning'
                      ? '#f59e0b'
                      : problem.severity === 'info'
                        ? '#3b82f6'
                        : '#10b981',
                color: 'white',
              }}
              role="status"
              aria-label={`重要度: ${severityLabels[problem.severity]}`}
            >
              {severityLabels[problem.severity]}
            </span>
            <span className="text-xs text-gray-600">{categoryLabels[problem.category]}</span>
          </div>
          <h3 id={`problem-title-${problem.id}`} className="text-lg font-semibold text-gray-900">
            {problem.title}
          </h3>
        </div>
        <time className="text-xs text-gray-500 whitespace-nowrap ml-4" dateTime={new Date(problem.detectedAt).toISOString()}>
          {detectedTime}
        </time>
      </div>

      {/* 説明 */}
      <p className="text-sm text-gray-700 mb-3">{problem.description}</p>

      {/* 推奨アクション */}
      {problem.suggestedActions.length > 0 && (
        <div className="mt-3">
          <h4 className="text-sm font-semibold text-gray-900 mb-2">推奨アクション:</h4>
          <ul className="list-disc list-inside space-y-1" role="list">
            {problem.suggestedActions.map((action, index) => (
              <li key={index} className="text-sm text-gray-700">
                {action}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* 影響を受けるメトリクス */}
      <div className="mt-3 text-xs text-gray-500">
        <span>関連メトリクス: </span>
        <span className="font-mono">{problem.affectedMetric}</span>
      </div>
    </div>
  );
}
