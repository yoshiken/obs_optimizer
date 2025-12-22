import type { NetworkMetrics } from '../../../types';
import { MetricCard } from './MetricCard';
import { formatSpeed } from '../utils/formatters';

interface NetworkMetricsCardProps {
  metrics: NetworkMetrics;
  /** コンパクトモード */
  compact?: boolean;
}

/**
 * ネットワーク使用状況を表示するカード
 */
export function NetworkMetricsCard({
  metrics,
  compact = false,
}: NetworkMetricsCardProps) {
  const rowStyle: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    marginTop: '8px',
  };

  const iconStyle: React.CSSProperties = {
    fontSize: '18px',
    width: '24px',
    textAlign: 'center',
  };

  const labelStyle: React.CSSProperties = {
    fontSize: '12px',
    color: 'var(--text-muted, #9ca3af)',
    minWidth: '80px',
  };

  const valueStyle: React.CSSProperties = {
    fontSize: '16px',
    fontWeight: 600,
  };

  return (
    <MetricCard title="ネットワーク" severity="normal">
      {/* アップロード */}
      <div style={rowStyle}>
        <span style={{ ...iconStyle, color: '#3b82f6' }}>↑</span>
        {!compact && <span style={labelStyle}>アップロード</span>}
        <span style={valueStyle}>{formatSpeed(metrics.uploadBytesPerSec)}</span>
      </div>

      {/* ダウンロード */}
      <div style={rowStyle}>
        <span style={{ ...iconStyle, color: '#22c55e' }}>↓</span>
        {!compact && <span style={labelStyle}>ダウンロード</span>}
        <span style={valueStyle}>{formatSpeed(metrics.downloadBytesPerSec)}</span>
      </div>
    </MetricCard>
  );
}
