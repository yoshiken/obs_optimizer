import { getEncoderInfo } from '../utils/encoderLabels';

interface EncoderBadgeProps {
  /** エンコーダーID (例: "jim_av1_nvenc", "obs_x264") */
  encoderId: string;
  /** 詳細情報を表示するか */
  showDetails?: boolean;
  /** カスタムクラス名 */
  className?: string;
}

/**
 * エンコーダー情報を視覚的にわかりやすく表示するバッジコンポーネント
 *
 * 機能:
 * - エンコーダー種類に応じた色分け (GPU: 青, CPU: グレー)
 * - コーデック情報の表示 (H.264, HEVC, AV1)
 * - ダークモード対応
 */
export function EncoderBadge({ encoderId, showDetails = false, className = '' }: EncoderBadgeProps) {
  const info = getEncoderInfo(encoderId);

  // エンコーダー種類に応じた色設定
  const colorClasses = getColorClasses(info.type);

  return (
    <div className={`inline-flex items-center gap-2 ${className}`}>
      {/* メインバッジ */}
      <span
        className={`px-3 py-1 rounded-full text-sm font-medium ${colorClasses.badge}`}
        title={`種類: ${info.type === 'gpu' ? 'GPU' : 'CPU'}, コーデック: ${info.codec}`}
      >
        {info.label}
      </span>

      {/* 詳細情報（オプション） */}
      {showDetails && (
        <div className="flex items-center gap-1">
          {/* コーデックバッジ */}
          <span
            className={`px-2 py-0.5 rounded text-xs font-medium ${colorClasses.codec}`}
            title={`コーデック: ${info.codec}`}
          >
            {info.codec}
          </span>

          {/* 種類アイコン */}
          <span className="text-xs text-gray-500 dark:text-gray-400" title={info.type === 'gpu' ? 'GPUエンコード' : 'CPUエンコード'}>
            {info.type === 'gpu' ? '⚡' : '💻'}
          </span>
        </div>
      )}
    </div>
  );
}

/**
 * エンコーダー種類に応じた色クラスを取得
 */
function getColorClasses(type: 'gpu' | 'cpu' | 'unknown'): {
  badge: string;
  codec: string;
} {
  switch (type) {
    case 'gpu':
      return {
        badge: 'bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300',
        codec: 'bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-400',
      };
    case 'cpu':
      return {
        badge: 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300',
        codec: 'bg-gray-50 dark:bg-gray-800 text-gray-600 dark:text-gray-400',
      };
    case 'unknown':
    default:
      return {
        badge: 'bg-yellow-100 dark:bg-yellow-900 text-yellow-700 dark:text-yellow-300',
        codec: 'bg-yellow-50 dark:bg-yellow-950 text-yellow-600 dark:text-yellow-400',
      };
  }
}
