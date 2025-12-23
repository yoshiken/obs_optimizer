import type { MemoryMetrics } from '../../../types';
import { MetricCard } from './MetricCard';
import { formatBytes, formatPercent } from '../utils/formatters';
import { getMemorySeverity, getSeverityColor } from '../utils/severity';

interface MemoryMetricsCardProps {
  metrics: MemoryMetrics;
  /** コンパクトモード */
  compact?: boolean;
}

/**
 * メモリ使用状況を表示するカード
 */
export function MemoryMetricsCard({
  metrics,
  compact = false,
}: MemoryMetricsCardProps) {
  const severity = getMemorySeverity(metrics.usagePercent);

  const barStyle: React.CSSProperties = {
    height: '8px',
    borderRadius: '4px',
    backgroundColor: '#e5e7eb',
    overflow: 'hidden',
    marginTop: '8px',
  };

  const barFillStyle: React.CSSProperties = {
    height: '100%',
    width: `${Math.min(metrics.usagePercent, 100)}%`,
    backgroundColor: getSeverityColor(severity),
    transition: 'width 0.3s ease',
  };

  const detailStyle: React.CSSProperties = {
    display: 'grid',
    gridTemplateColumns: 'repeat(2, 1fr)',
    gap: '8px',
    marginTop: '12px',
    fontSize: '12px',
    color: 'var(--text-secondary, #6b7280)',
  };

  return (
    <MetricCard title="メモリ" severity={severity}>
      <div style={{ display: 'flex', alignItems: 'baseline', gap: '8px' }}>
        <span>{formatPercent(metrics.usagePercent, 1)}</span>
      </div>

      {/* 使用率バー */}
      <div style={barStyle}>
        <div style={barFillStyle} />
      </div>

      {/* 詳細情報 */}
      {!compact && (
        <div style={detailStyle}>
          <div>
            <div style={{ color: 'var(--text-muted, #9ca3af)' }}>使用中</div>
            <div style={{ fontWeight: 500 }}>{formatBytes(metrics.usedBytes)}</div>
          </div>
          <div>
            <div style={{ color: 'var(--text-muted, #9ca3af)' }}>合計</div>
            <div style={{ fontWeight: 500 }}>{formatBytes(metrics.totalBytes)}</div>
          </div>
          <div>
            <div style={{ color: 'var(--text-muted, #9ca3af)' }}>利用可能</div>
            <div style={{ fontWeight: 500 }}>{formatBytes(metrics.availableBytes)}</div>
          </div>
        </div>
      )}
    </MetricCard>
  );
}
