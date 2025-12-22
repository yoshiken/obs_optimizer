import type { GpuMetrics } from '../../../types';
import { MetricCard } from './MetricCard';
import { formatBytes, formatPercent } from '../utils/formatters';
import { getEncoderSeverity, getGpuSeverity, getSeverityColor } from '../utils/severity';

interface GpuMetricsCardProps {
  metrics: GpuMetrics | null;
  /** コンパクトモード */
  compact?: boolean;
}

/**
 * GPU使用状況を表示するカード
 */
export function GpuMetricsCard({
  metrics,
  compact = false,
}: GpuMetricsCardProps) {
  // GPUが検出されない場合
  if (!metrics) {
    return (
      <MetricCard title="GPU" severity="normal">
        <div style={{
          fontSize: '14px',
          color: 'var(--text-muted, #9ca3af)',
          fontStyle: 'italic'
        }}>
          GPUが検出されませんでした
        </div>
        <div style={{
          fontSize: '12px',
          color: 'var(--text-muted, #9ca3af)',
          marginTop: '8px'
        }}>
          NVIDIA GPUをお使いの場合、ドライバのインストールが必要です
        </div>
      </MetricCard>
    );
  }

  const gpuSeverity = getGpuSeverity(metrics.usagePercent);
  const encoderSeverity = getEncoderSeverity(metrics.encoderUsage);

  const barStyle: React.CSSProperties = {
    height: '8px',
    borderRadius: '4px',
    backgroundColor: '#e5e7eb',
    overflow: 'hidden',
  };

  const createBarFill = (usage: number, color: string): React.CSSProperties => ({
    height: '100%',
    width: `${Math.min(usage, 100)}%`,
    backgroundColor: color,
    transition: 'width 0.3s ease',
  });

  const detailStyle: React.CSSProperties = {
    display: 'grid',
    gridTemplateColumns: 'repeat(2, 1fr)',
    gap: '8px',
    marginTop: '12px',
    fontSize: '12px',
    color: 'var(--text-secondary, #6b7280)',
  };

  return (
    <MetricCard title="GPU" severity={gpuSeverity}>
      <div>
        <span>{formatPercent(metrics.usagePercent)}</span>
      </div>

      {/* GPU名称 */}
      <div style={{
        fontSize: '12px',
        color: 'var(--text-secondary, #6b7280)',
        marginTop: '4px',
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        whiteSpace: 'nowrap',
      }}>
        {metrics.name}
      </div>

      {/* GPU使用率バー */}
      <div style={{ marginTop: '8px', ...barStyle }}>
        <div style={createBarFill(metrics.usagePercent, getSeverityColor(gpuSeverity))} />
      </div>

      {!compact && (
        <>
          {/* エンコーダー使用率 */}
          <div style={{ marginTop: '12px' }}>
            <div style={{
              display: 'flex',
              justifyContent: 'space-between',
              fontSize: '12px',
              marginBottom: '4px',
            }}>
              <span style={{ color: 'var(--text-muted, #9ca3af)' }}>エンコーダー</span>
              <span>{formatPercent(metrics.encoderUsage)}</span>
            </div>
            <div style={barStyle}>
              <div style={createBarFill(metrics.encoderUsage, getSeverityColor(encoderSeverity))} />
            </div>
          </div>

          {/* VRAM情報 */}
          <div style={detailStyle}>
            <div>
              <div style={{ color: 'var(--text-muted, #9ca3af)' }}>VRAM使用中</div>
              <div style={{ fontWeight: 500 }}>{formatBytes(metrics.memoryUsedBytes)}</div>
            </div>
            <div>
              <div style={{ color: 'var(--text-muted, #9ca3af)' }}>VRAM合計</div>
              <div style={{ fontWeight: 500 }}>{formatBytes(metrics.memoryTotalBytes)}</div>
            </div>
          </div>
        </>
      )}
    </MetricCard>
  );
}
