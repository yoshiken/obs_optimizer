import { useEffect } from 'react';
import { useMetricsStore } from '../../stores/metricsStore';
import { CpuMetricsCard } from './components/CpuMetricsCard';
import { MemoryMetricsCard } from './components/MemoryMetricsCard';
import { GpuMetricsCard } from './components/GpuMetricsCard';
import { NetworkMetricsCard } from './components/NetworkMetricsCard';
import { formatRelativeTime } from './utils/formatters';

interface DetailedMetricsPanelProps {
  /** ポーリング間隔（ミリ秒） */
  refreshInterval?: number;
  /** コンパクトモード（スペースを節約） */
  compactMode?: boolean;
  /** 各コアのCPU使用率を表示するか */
  showPerCoreUsage?: boolean;
  /** 追加のCSSクラス */
  className?: string;
}

/**
 * 詳細システムメトリクスパネル
 *
 * CPU、メモリ、GPU、ネットワークの詳細情報を表示する
 */
export function DetailedMetricsPanel({
  refreshInterval = 1000,
  compactMode = false,
  showPerCoreUsage = false,
  className = '',
}: DetailedMetricsPanelProps) {
  const {
    metrics,
    obsProcessMetrics,
    loading,
    error,
    lastUpdate,
    startPolling,
  } = useMetricsStore();

  useEffect(() => {
    const stopPolling = startPolling(refreshInterval);
    return stopPolling;
  }, [startPolling, refreshInterval]);

  const containerStyle: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
    padding: '16px',
  };

  const headerStyle: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '8px',
  };

  const titleStyle: React.CSSProperties = {
    fontSize: '18px',
    fontWeight: 600,
    color: 'var(--text-primary, #0f0f0f)',
  };

  const timestampStyle: React.CSSProperties = {
    fontSize: '12px',
    color: 'var(--text-muted, #9ca3af)',
  };

  const gridStyle: React.CSSProperties = {
    display: 'grid',
    gridTemplateColumns: compactMode ? '1fr' : 'repeat(auto-fit, minmax(280px, 1fr))',
    gap: '16px',
  };

  const obsInfoStyle: React.CSSProperties = {
    marginTop: '16px',
    padding: '12px',
    backgroundColor: 'var(--card-bg, #f9fafb)',
    borderRadius: '8px',
    fontSize: '14px',
  };

  if (error) {
    return (
      <div className={className} style={containerStyle}>
        <div style={{
          padding: '24px',
          backgroundColor: '#fef2f2',
          borderRadius: '8px',
          color: '#dc2626',
          textAlign: 'center',
        }}>
          <div style={{ fontWeight: 600, marginBottom: '8px' }}>エラーが発生しました</div>
          <div>{error}</div>
        </div>
      </div>
    );
  }

  if (loading && !metrics) {
    return (
      <div className={className} style={containerStyle}>
        <div style={headerStyle}>
          <span style={titleStyle}>システムメトリクス</span>
        </div>
        <div style={{ textAlign: 'center', padding: '40px', color: 'var(--text-muted, #9ca3af)' }}>
          読み込み中...
        </div>
      </div>
    );
  }

  if (!metrics) {
    return null;
  }

  return (
    <div className={className} style={containerStyle}>
      {/* ヘッダー */}
      <div style={headerStyle}>
        <span style={titleStyle}>システムメトリクス</span>
        {lastUpdate && (
          <span style={timestampStyle}>
            最終更新: {formatRelativeTime(lastUpdate)}
          </span>
        )}
      </div>

      {/* メトリクスグリッド */}
      <div style={gridStyle}>
        <CpuMetricsCard
          metrics={metrics.cpu}
          showPerCore={showPerCoreUsage}
          compact={compactMode}
        />
        <MemoryMetricsCard
          metrics={metrics.memory}
          compact={compactMode}
        />
        <GpuMetricsCard
          metrics={metrics.gpu}
          compact={compactMode}
        />
        <NetworkMetricsCard
          metrics={metrics.network}
          compact={compactMode}
        />
      </div>

      {/* OBSプロセス情報（検出時のみ表示） */}
      {obsProcessMetrics?.mainProcess && (
        <div style={obsInfoStyle}>
          <div style={{ fontWeight: 600, marginBottom: '8px' }}>
            OBS プロセス
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '12px' }}>
            <div>
              <div style={{ color: 'var(--text-muted, #9ca3af)', fontSize: '12px' }}>
                プロセス名
              </div>
              <div>{obsProcessMetrics.mainProcess.name}</div>
            </div>
            <div>
              <div style={{ color: 'var(--text-muted, #9ca3af)', fontSize: '12px' }}>
                CPU使用率
              </div>
              <div>{obsProcessMetrics.totalCpuUsage.toFixed(1)}%</div>
            </div>
            <div>
              <div style={{ color: 'var(--text-muted, #9ca3af)', fontSize: '12px' }}>
                メモリ使用量
              </div>
              <div>
                {(obsProcessMetrics.totalMemoryBytes / (1024 * 1024)).toFixed(0)} MB
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
