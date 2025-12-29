import type { CpuMetrics } from '../../../types';
import { MetricCard } from './MetricCard';
import { formatPercent } from '../utils/formatters';
import { getCpuSeverity, getSeverityColor } from '../utils/severity';

interface CpuMetricsCardProps {
  metrics: CpuMetrics;
  /** 各コアの使用率を表示するか */
  showPerCore?: boolean;
  /** コンパクトモード */
  compact?: boolean;
  /** CPUモデル名を表示するか */
  showCpuName?: boolean;
}

/**
 * CPU使用率を表示するカード
 */
export function CpuMetricsCard({
  metrics,
  showPerCore = false,
  compact = false,
  showCpuName = true,
}: CpuMetricsCardProps) {
  const severity = getCpuSeverity(metrics.usagePercent);

  const barContainerStyle: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
    marginTop: '12px',
    fontSize: '12px',
  };

  const barStyle = (_usage: number): React.CSSProperties => ({
    height: '8px',
    borderRadius: '4px',
    backgroundColor: '#e5e7eb',
    overflow: 'hidden',
  });

  const barFillStyle = (usage: number): React.CSSProperties => ({
    height: '100%',
    width: `${Math.min(usage, 100)}%`,
    backgroundColor: getSeverityColor(getCpuSeverity(usage)),
    transition: 'width 0.3s ease',
  });

  return (
    <MetricCard title="CPU" severity={severity}>
      <div style={{ display: 'flex', alignItems: 'baseline', gap: '8px' }}>
        <span>{formatPercent(metrics.usagePercent, 1)}</span>
        <span style={{ fontSize: '14px', color: 'var(--text-secondary, #6b7280)' }}>
          {metrics.coreCount}コア
        </span>
      </div>

      {/* CPU名表示 */}
      {showCpuName && !compact && (
        <div
          style={{
            fontSize: '12px',
            color: 'var(--text-muted, #9ca3af)',
            marginTop: '4px',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
          title={metrics.cpuName}
        >
          {metrics.cpuName}
        </div>
      )}

      {/* メイン使用率バー */}
      <div style={{ marginTop: '8px', ...barStyle(metrics.usagePercent) }}>
        <div style={barFillStyle(metrics.usagePercent)} />
      </div>

      {/* 各コアの使用率 */}
      {showPerCore && !compact && metrics.perCoreUsage.length > 0 && (
        <div style={barContainerStyle}>
          <div style={{ fontWeight: 500, marginBottom: '4px' }}>コア別使用率</div>
          {metrics.perCoreUsage.map((usage, index) => (
            <div key={index} style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span style={{ minWidth: '40px', color: 'var(--text-muted, #9ca3af)' }}>
                #{index}
              </span>
              <div style={{ flex: 1, ...barStyle(usage) }}>
                <div style={barFillStyle(usage)} />
              </div>
              <span style={{ minWidth: '45px', textAlign: 'right' }}>
                {formatPercent(usage, 0)}
              </span>
            </div>
          ))}
        </div>
      )}
    </MetricCard>
  );
}
