import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { CpuMetricsCard } from './CpuMetricsCard';
import type { CpuMetrics } from '../../../types';

describe('CpuMetricsCard', () => {
  const mockMetrics: CpuMetrics = {
    usagePercent: 45.5,
    coreCount: 8,
    perCoreUsage: [40, 50, 45, 42, 48, 43, 47, 44],
    cpuName: 'Intel(R) Core(TM) i7-9700K CPU @ 3.60GHz',
  };

  it('CPU使用率とコア数を表示する', () => {
    render(<CpuMetricsCard metrics={mockMetrics} />);

    expect(screen.getByText(/45\.5%/)).toBeInTheDocument();
    expect(screen.getByText(/8コア/)).toBeInTheDocument();
  });

  it('デフォルトでCPU名を表示する', () => {
    render(<CpuMetricsCard metrics={mockMetrics} />);

    expect(screen.getByText('Intel(R) Core(TM) i7-9700K CPU @ 3.60GHz')).toBeInTheDocument();
  });

  it('showCpuName=falseの場合、CPU名を表示しない', () => {
    render(<CpuMetricsCard metrics={mockMetrics} showCpuName={false} />);

    expect(screen.queryByText('Intel(R) Core(TM) i7-9700K CPU @ 3.60GHz')).not.toBeInTheDocument();
  });

  it('compactモードの場合、CPU名を表示しない', () => {
    render(<CpuMetricsCard metrics={mockMetrics} compact={true} />);

    expect(screen.queryByText('Intel(R) Core(TM) i7-9700K CPU @ 3.60GHz')).not.toBeInTheDocument();
  });

  it('CPU名にtitle属性を設定する（ホバー時の全文表示用）', () => {
    render(<CpuMetricsCard metrics={mockMetrics} />);

    const cpuNameElement = screen.getByText('Intel(R) Core(TM) i7-9700K CPU @ 3.60GHz');
    expect(cpuNameElement).toHaveAttribute('title', 'Intel(R) Core(TM) i7-9700K CPU @ 3.60GHz');
  });

  it('showPerCore=trueの場合、各コアの使用率を表示する', () => {
    render(<CpuMetricsCard metrics={mockMetrics} showPerCore={true} />);

    expect(screen.getByText('コア別使用率')).toBeInTheDocument();
  });

  it('showPerCore=falseの場合、各コアの使用率を表示しない', () => {
    render(<CpuMetricsCard metrics={mockMetrics} showPerCore={false} />);

    expect(screen.queryByText('コア別使用率')).not.toBeInTheDocument();
  });
});
