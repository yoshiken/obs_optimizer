/**
 * セベリティレベル
 */
export type SeverityLevel = 'normal' | 'warning' | 'critical';

/**
 * セベリティ設定
 */
export interface SeverityThresholds {
  warning: number;
  critical: number;
}

/**
 * メトリクス種別ごとのセベリティ閾値
 */
export const SEVERITY_THRESHOLDS = {
  cpu: {
    warning: 70,
    critical: 90,
  },
  memory: {
    warning: 80,
    critical: 95,
  },
  gpu: {
    warning: 85,
    critical: 95,
  },
} as const;

/**
 * 使用率パーセンテージからセベリティレベルを判定
 */
export function getSeverityLevel(
  value: number,
  thresholds: SeverityThresholds
): SeverityLevel {
  if (value >= thresholds.critical) {
    return 'critical';
  }
  if (value >= thresholds.warning) {
    return 'warning';
  }
  return 'normal';
}

/**
 * セベリティレベルに応じたTailwind CSSクラス名を取得
 */
export function getSeverityColorClass(severity: SeverityLevel): string {
  switch (severity) {
    case 'critical':
      return 'text-red-600 dark:text-red-300';
    case 'warning':
      return 'text-yellow-600 dark:text-yellow-300';
    case 'normal':
    default:
      return 'text-gray-800 dark:text-gray-100';
  }
}

/**
 * CPU使用率のセベリティ判定
 */
export function getCpuSeverity(usagePercent: number): SeverityLevel {
  return getSeverityLevel(usagePercent, SEVERITY_THRESHOLDS.cpu);
}

/**
 * メモリ使用率のセベリティ判定
 */
export function getMemorySeverity(usagePercent: number): SeverityLevel {
  return getSeverityLevel(usagePercent, SEVERITY_THRESHOLDS.memory);
}

/**
 * GPU使用率のセベリティ判定
 */
export function getGpuSeverity(usagePercent: number): SeverityLevel {
  return getSeverityLevel(usagePercent, SEVERITY_THRESHOLDS.gpu);
}
