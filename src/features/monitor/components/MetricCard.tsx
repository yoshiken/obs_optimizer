import type { ReactNode } from 'react';
import type { Severity } from '../utils/severity';
import { getSeverityColor } from '../utils/severity';

interface MetricCardProps {
  /** カードのタイトル */
  title: string;
  /** アイコン（オプション） */
  icon?: ReactNode;
  /** 重要度 */
  severity?: Severity;
  /** ローディング状態 */
  loading?: boolean;
  /** エラーメッセージ */
  error?: string | null;
  /** カード内のコンテンツ */
  children: ReactNode;
  /** 追加のCSSクラス */
  className?: string;
}

/**
 * メトリクス表示用の再利用可能なカードコンポーネント
 */
export function MetricCard({
  title,
  icon,
  severity = 'normal',
  loading = false,
  error,
  children,
  className = '',
}: MetricCardProps) {
  const borderColor = getSeverityColor(severity);

  const cardStyle: React.CSSProperties = {
    borderLeft: `4px solid ${borderColor}`,
    backgroundColor: 'var(--card-bg, #ffffff)',
    border: '1px solid var(--card-border, #e5e7eb)',
    borderRadius: '8px',
    padding: '16px',
    boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1)',
    transition: 'all 0.2s ease',
  };

  const headerStyle: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    marginBottom: '12px',
    fontSize: '14px',
    fontWeight: 600,
    color: 'var(--text-secondary, #6b7280)',
  };

  const contentStyle: React.CSSProperties = {
    fontSize: '24px',
    fontWeight: 700,
    color: 'var(--text-primary, #0f0f0f)',
  };

  if (error) {
    return (
      <div style={{ ...cardStyle, borderLeftColor: '#ef4444' }} className={className}>
        <div style={headerStyle}>
          {icon}
          <span>{title}</span>
        </div>
        <div style={{ color: '#ef4444', fontSize: '14px' }}>
          エラー: {error}
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div style={cardStyle} className={className}>
        <div style={headerStyle}>
          {icon}
          <span>{title}</span>
        </div>
        <div style={{ ...contentStyle, opacity: 0.5 }}>
          読み込み中...
        </div>
      </div>
    );
  }

  return (
    <div style={cardStyle} className={`card-interactive ${className}`}>
      <div style={headerStyle}>
        {icon}
        <span>{title}</span>
      </div>
      <div style={contentStyle}>
        {children}
      </div>
    </div>
  );
}
