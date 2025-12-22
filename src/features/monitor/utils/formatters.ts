// 表示フォーマット用のユーティリティ関数

/** バイト数を人間が読みやすい形式に変換 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) {return '0 B';}

  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const value = bytes / Math.pow(k, i);

  return value.toFixed(2) + ' ' + units[i];
}

/** 転送速度を人間が読みやすい形式に変換 */
export function formatSpeed(bytesPerSec: number): string {
  return formatBytes(bytesPerSec) + '/s';
}

/** パーセント値をフォーマット */
export function formatPercent(value: number, decimals: number = 1): string {
  return value.toFixed(decimals) + '%';
}

/** タイムスタンプを相対時間に変換 */
export function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;

  if (diff < 1000) {return '今';}
  if (diff < 60000) {return `${Math.floor(diff / 1000)}秒前`;}
  if (diff < 3600000) {return `${Math.floor(diff / 60000)}分前`;}

  return `${Math.floor(diff / 3600000)}時間前`;
}
