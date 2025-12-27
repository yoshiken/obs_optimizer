import type { CpuTier, EffectiveTier, MemoryTier } from '../../types/commands';
import {
  getTierLabel,
  getTierScore,
  normalizeToColorKey,
  TIER_COLORS,
} from './tierUtils';

// ========================================
// コンポーネントProps定義
// ========================================

interface ComponentTierCardProps {
  /** コンポーネント種別 */
  type: 'gpu' | 'cpu' | 'memory';
  /** コンポーネント名（表示用、例: "RTX 4080"） */
  name: string;
  /** ティア（GPU: EffectiveTier, CPU: CpuTier, Memory: MemoryTier） */
  tier: EffectiveTier | CpuTier | MemoryTier;
  /** 詳細情報（例: "12コア", "32GB"） */
  detail?: string;
  /** 能力一覧（例: ["AV1対応", "NVENC"]) */
  capabilities?: string[];
  /** コンパクト表示モード */
  compact?: boolean;
}

/**
 * コンポーネント別ティア表示カード
 *
 * 機能:
 * - GPU/CPU/Memoryの各コンポーネントをカードで表示
 * - ティア別カラーリング（TierS=emerald, TierA=blue, TierB=indigo, TierC=amber, TierD=red）
 * - プログレスバーによるティア可視化
 * - 能力リスト表示
 * - ダークモード対応
 *
 * 使用例:
 * ```tsx
 * <ComponentTierCard
 *   type="gpu"
 *   name="RTX 5080"
 *   tier="tierS"
 *   detail="AV1対応"
 *   capabilities={["NVENC", "AV1", "HEVC"]}
 * />
 * ```
 */
export function ComponentTierCard({
  type,
  name,
  tier,
  detail,
  capabilities = [],
  compact = false,
}: ComponentTierCardProps) {
  // ティアをTierColorKeyに正規化
  const colorKey = normalizeToColorKey(type, tier);
  const colors = TIER_COLORS[colorKey];
  const score = getTierScore(colorKey);
  const tierLabel = getTierLabel(colorKey);

  // アイコン取得
  const icon = getComponentIcon(type);

  return (
    <div
      className={`
        rounded-lg border p-4 transition-all duration-200 hover:shadow-md
        ${colors.bg} ${colors.bgDark}
        ${colors.border} ${colors.borderDark}
      `}
      role="article"
      aria-label={`${getComponentLabel(type)}評価: ${tierLabel}`}
    >
      {/* ヘッダー */}
      <div className="flex items-center gap-2 mb-2">
        <span className="text-xl" role="img" aria-hidden="true">
          {icon}
        </span>
        <span className={`text-sm font-medium ${colors.text} ${colors.textDark}`}>
          {getComponentLabel(type)}
        </span>
      </div>

      {/* 名称 */}
      <h4 className="text-base font-semibold text-gray-900 dark:text-gray-100 truncate" title={name}>
        {name}
      </h4>

      {/* 詳細情報 */}
      {detail && !compact && (
        <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">{detail}</p>
      )}

      {/* プログレスバー */}
      <div className="mt-3">
        <div className="flex items-center justify-between mb-1">
          <span className={`text-xs font-medium ${colors.text} ${colors.textDark}`}>
            {tierLabel}
          </span>
          <span className="text-xs text-gray-500 dark:text-gray-400">{score}%</span>
        </div>
        <div
          className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden"
          role="progressbar"
          aria-valuenow={score}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-label={`${getComponentLabel(type)}スコア: ${score}%`}
        >
          <div
            className={`h-full rounded-full transition-all duration-500 ${colors.bar} ${colors.barDark}`}
            style={{ width: `${score}%` }}
          />
        </div>
      </div>

      {/* 能力リスト */}
      {capabilities.length > 0 && !compact && (
        <div className="mt-3 flex flex-wrap gap-1">
          {capabilities.slice(0, 3).map((cap, index) => (
            <span
              key={index}
              className={`
                inline-block px-2 py-0.5 text-xs rounded-full
                bg-white/50 dark:bg-gray-800/50
                ${colors.text} ${colors.textDark}
              `}
            >
              {cap}
            </span>
          ))}
          {capabilities.length > 3 && (
            <span className="text-xs text-gray-500 dark:text-gray-400">
              +{capabilities.length - 3}
            </span>
          )}
        </div>
      )}
    </div>
  );
}

// ========================================
// ヘルパー関数
// ========================================

/**
 * コンポーネントタイプに応じたアイコンを取得
 */
function getComponentIcon(type: 'gpu' | 'cpu' | 'memory'): string {
  const icons: Record<string, string> = {
    gpu: '🎮',
    cpu: '🖥️',
    memory: '💾',
  };
  return icons[type] ?? '📊';
}

/**
 * コンポーネントタイプに応じたラベルを取得
 */
function getComponentLabel(type: 'gpu' | 'cpu' | 'memory'): string {
  const labels: Record<string, string> = {
    gpu: 'GPU',
    cpu: 'CPU',
    memory: 'メモリ',
  };
  return labels[type] ?? type.toUpperCase();
}
