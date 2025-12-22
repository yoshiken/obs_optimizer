// メトリクスの重要度判定用ユーティリティ

export type Severity = 'normal' | 'warning' | 'critical';

/** CPU使用率から重要度を計算 */
export function getCpuSeverity(usagePercent: number): Severity {
  if (usagePercent >= 90) {return 'critical';}
  if (usagePercent >= 70) {return 'warning';}
  return 'normal';
}

/** メモリ使用率から重要度を計算 */
export function getMemorySeverity(usagePercent: number): Severity {
  if (usagePercent >= 90) {return 'critical';}
  if (usagePercent >= 75) {return 'warning';}
  return 'normal';
}

/** GPU使用率から重要度を計算 */
export function getGpuSeverity(usagePercent: number): Severity {
  if (usagePercent >= 95) {return 'critical';}
  if (usagePercent >= 80) {return 'warning';}
  return 'normal';
}

/** エンコーダー使用率から重要度を計算 */
export function getEncoderSeverity(usagePercent: number): Severity {
  if (usagePercent >= 90) {return 'critical';}
  if (usagePercent >= 70) {return 'warning';}
  return 'normal';
}

/** 重要度に対応するCSSクラスを取得 */
export function getSeverityClass(severity: Severity): string {
  switch (severity) {
    case 'critical':
      return 'severity-critical';
    case 'warning':
      return 'severity-warning';
    default:
      return 'severity-normal';
  }
}

/** 重要度に対応する色を取得 */
export function getSeverityColor(severity: Severity): string {
  switch (severity) {
    case 'critical':
      return '#ef4444'; // 赤
    case 'warning':
      return '#f59e0b'; // 黄
    default:
      return '#22c55e'; // 緑
  }
}
