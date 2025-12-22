import { useMemo } from 'react';
import type { TimeSeriesDataPoint } from '../../stores/metricsStore';

interface MetricsChartProps {
  /** 表示するデータ */
  data: TimeSeriesDataPoint[];
  /** チャートのラベル */
  label: string;
  /** ラインの色 */
  color?: string;
  /** 最大値（指定しない場合はデータから自動計算） */
  maxValue?: number;
  /** 最小値 */
  minValue?: number;
  /** チャートの高さ（ピクセル） */
  height?: number;
  /** 単位（ツールチップ用） */
  unit?: string;
  /** グリッド線を表示するか */
  showGrid?: boolean;
  /** 現在値を表示するか */
  showCurrentValue?: boolean;
}

/**
 * SVGベースのシンプルな時系列チャート
 *
 * 外部ライブラリなしで軽量に描画する
 */
export function MetricsChart({
  data,
  label,
  color = '#3b82f6',
  maxValue,
  minValue = 0,
  height = 100,
  unit = '',
  showGrid = true,
  showCurrentValue = true,
}: MetricsChartProps) {
  const width = 300; // 固定幅（親要素でスケール）
  const padding = useMemo(() => ({ top: 10, right: 10, bottom: 20, left: 40 }), []);

  // グラフ描画領域
  const chartWidth = width - padding.left - padding.right;
  const chartHeight = height - padding.top - padding.bottom;

  // データからパスを生成
  const { path, currentValue, calculatedMax } = useMemo(() => {
    if (data.length === 0) {
      return { path: '', currentValue: 0, calculatedMax: 100 };
    }

    // 最大値を計算
    const dataMax = Math.max(...data.map(d => d.value));
    const effectiveMax = maxValue ?? Math.max(dataMax * 1.1, 10);

    // 最新の値
    const current = data[data.length - 1].value;

    // SVGパスを生成
    const points = data.map((point, index) => {
      const x = padding.left + (index / Math.max(data.length - 1, 1)) * chartWidth;
      const normalizedValue = (point.value - minValue) / (effectiveMax - minValue);
      const y = padding.top + chartHeight - (normalizedValue * chartHeight);
      return `${x},${y}`;
    });

    const pathData = `M ${points.join(' L ')}`;

    return {
      path: pathData,
      currentValue: current,
      calculatedMax: effectiveMax,
    };
  }, [data, maxValue, minValue, chartWidth, chartHeight, padding]);

  // グリッド線のY座標
  const gridLines = useMemo(() => {
    const lines = [];
    const steps = 4;
    for (let i = 0; i <= steps; i++) {
      const y = padding.top + (i / steps) * chartHeight;
      const value = calculatedMax - (i / steps) * (calculatedMax - minValue);
      lines.push({ y, value });
    }
    return lines;
  }, [calculatedMax, minValue, chartHeight, padding.top]);

  const containerStyle: React.CSSProperties = {
    position: 'relative',
    width: '100%',
  };

  const headerStyle: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '8px',
    fontSize: '14px',
  };

  const labelStyle: React.CSSProperties = {
    fontWeight: 500,
    color: 'var(--text-primary, #0f0f0f)',
  };

  const valueStyle: React.CSSProperties = {
    fontWeight: 600,
    color,
  };

  return (
    <div style={containerStyle}>
      <div style={headerStyle}>
        <span style={labelStyle}>{label}</span>
        {showCurrentValue && data.length > 0 && (
          <span style={valueStyle}>
            {currentValue.toFixed(1)}{unit}
          </span>
        )}
      </div>

      <svg
        width="100%"
        height={height}
        viewBox={`0 0 ${width} ${height}`}
        preserveAspectRatio="none"
        style={{ display: 'block' }}
      >
        {/* グリッド線 */}
        {showGrid && gridLines.map((line, index) => (
          <g key={index}>
            <line
              x1={padding.left}
              y1={line.y}
              x2={width - padding.right}
              y2={line.y}
              stroke="#e5e7eb"
              strokeWidth="1"
              strokeDasharray={index === gridLines.length - 1 ? 'none' : '2,2'}
            />
            <text
              x={padding.left - 5}
              y={line.y + 4}
              textAnchor="end"
              fontSize="10"
              fill="#9ca3af"
            >
              {line.value.toFixed(0)}
            </text>
          </g>
        ))}

        {/* データライン */}
        {data.length > 1 && (
          <path
            d={path}
            fill="none"
            stroke={color}
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        )}

        {/* 塗りつぶし（オプション） */}
        {data.length > 1 && (
          <path
            d={`${path} L ${padding.left + chartWidth},${padding.top + chartHeight} L ${padding.left},${padding.top + chartHeight} Z`}
            fill={color}
            fillOpacity="0.1"
          />
        )}

        {/* データポイント（最新のみ） */}
        {data.length > 0 && (
          <circle
            cx={padding.left + chartWidth}
            cy={padding.top + chartHeight - ((currentValue - minValue) / (calculatedMax - minValue)) * chartHeight}
            r="4"
            fill={color}
          />
        )}
      </svg>

      {/* データがない場合のメッセージ */}
      {data.length === 0 && (
        <div style={{
          position: 'absolute',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          color: 'var(--text-muted, #9ca3af)',
          fontSize: '12px',
        }}>
          データなし
        </div>
      )}
    </div>
  );
}
