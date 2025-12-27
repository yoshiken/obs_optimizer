// ========================================
// ティア関連ユーティリティ
// Phase 5-6: SystemCapability UI用
// ========================================

import type { CpuTier, EffectiveTier, MemoryTier, OverallTier } from '../../types/commands';

// ========================================
// 型定義
// ========================================

export type TierColorKey = 'tierS' | 'tierA' | 'tierB' | 'tierC' | 'tierD';

export interface TierColors {
  bg: string;
  bgDark: string;
  text: string;
  textDark: string;
  border: string;
  borderDark: string;
  bar: string;
  barDark: string;
}

// ========================================
// カラー定義
// ========================================

export const TIER_COLORS: Record<TierColorKey, TierColors> = {
  tierS: {
    bg: 'bg-emerald-50',
    bgDark: 'dark:bg-emerald-950',
    text: 'text-emerald-700',
    textDark: 'dark:text-emerald-300',
    border: 'border-emerald-200',
    borderDark: 'dark:border-emerald-800',
    bar: 'bg-emerald-500',
    barDark: 'dark:bg-emerald-400',
  },
  tierA: {
    bg: 'bg-blue-50',
    bgDark: 'dark:bg-blue-950',
    text: 'text-blue-700',
    textDark: 'dark:text-blue-300',
    border: 'border-blue-200',
    borderDark: 'dark:border-blue-800',
    bar: 'bg-blue-500',
    barDark: 'dark:bg-blue-400',
  },
  tierB: {
    bg: 'bg-indigo-50',
    bgDark: 'dark:bg-indigo-950',
    text: 'text-indigo-700',
    textDark: 'dark:text-indigo-300',
    border: 'border-indigo-200',
    borderDark: 'dark:border-indigo-800',
    bar: 'bg-indigo-500',
    barDark: 'dark:bg-indigo-400',
  },
  tierC: {
    bg: 'bg-amber-50',
    bgDark: 'dark:bg-amber-950',
    text: 'text-amber-700',
    textDark: 'dark:text-amber-300',
    border: 'border-amber-200',
    borderDark: 'dark:border-amber-800',
    bar: 'bg-amber-500',
    barDark: 'dark:bg-amber-400',
  },
  tierD: {
    bg: 'bg-red-50',
    bgDark: 'dark:bg-red-950',
    text: 'text-red-700',
    textDark: 'dark:text-red-300',
    border: 'border-red-200',
    borderDark: 'dark:border-red-800',
    bar: 'bg-red-500',
    barDark: 'dark:bg-red-400',
  },
};

// ========================================
// ティア変換ユーティリティ
// ========================================

/**
 * CPUティアをTierColorKeyに変換
 */
export function cpuTierToColorKey(tier: CpuTier): TierColorKey {
  const mapping: Record<CpuTier, TierColorKey> = {
    highEnd: 'tierS',
    upperMiddle: 'tierA',
    middle: 'tierB',
    entry: 'tierD',
  };
  return mapping[tier] ?? 'tierC';
}

/**
 * メモリティアをTierColorKeyに変換
 */
export function memoryTierToColorKey(tier: MemoryTier): TierColorKey {
  const mapping: Record<MemoryTier, TierColorKey> = {
    abundant: 'tierS',
    adequate: 'tierA',
    standard: 'tierB',
    entry: 'tierD',
  };
  return mapping[tier] ?? 'tierC';
}

/**
 * OverallTierをTierColorKeyに変換
 */
export function overallTierToColorKey(tier: OverallTier): TierColorKey {
  const mapping: Record<OverallTier, TierColorKey> = {
    ultra: 'tierS',
    high: 'tierA',
    medium: 'tierB',
    low: 'tierC',
    minimal: 'tierD',
  };
  return mapping[tier] ?? 'tierC';
}

/**
 * ティアスコアを取得（プログレスバー用）
 */
export function getTierScore(tier: TierColorKey): number {
  const scores: Record<TierColorKey, number> = {
    tierS: 100,
    tierA: 80,
    tierB: 60,
    tierC: 40,
    tierD: 20,
  };
  return scores[tier] ?? 50;
}

/**
 * ティアラベルを取得
 */
export function getTierLabel(tier: TierColorKey): string {
  const labels: Record<TierColorKey, string> = {
    tierS: 'Tier S',
    tierA: 'Tier A',
    tierB: 'Tier B',
    tierC: 'Tier C',
    tierD: 'Tier D',
  };
  return labels[tier] ?? 'Unknown';
}

/**
 * OverallTierの日本語ラベルを取得
 */
export function getOverallTierLabel(tier: OverallTier): string {
  const labels: Record<OverallTier, string> = {
    ultra: '最高性能',
    high: '高性能',
    medium: '標準',
    low: '軽量',
    minimal: 'エントリー',
  };
  return labels[tier] ?? tier;
}

/**
 * OverallTierに応じた説明文を取得
 */
export function getOverallTierDescription(tier: OverallTier): string {
  const descriptions: Record<OverallTier, string> = {
    ultra: '1440p60配信やAV1エンコードに対応した最高性能です',
    high: '高品質1080p60配信が安定して可能なスペックです',
    medium: '1080p60配信が可能な標準的なスペックです',
    low: '720p60または1080p30配信が推奨されるスペックです',
    minimal: '720p30配信が推奨される軽量スペックです',
  };
  return descriptions[tier] ?? '設定を分析中...';
}

/**
 * 各コンポーネントタイプのティアをTierColorKeyに正規化
 */
export function normalizeToColorKey(
  type: 'gpu' | 'cpu' | 'memory',
  tier: EffectiveTier | CpuTier | MemoryTier
): TierColorKey {
  switch (type) {
    case 'gpu':
      // EffectiveTierはすでにTierColorKey互換
      return tier as TierColorKey;
    case 'cpu':
      return cpuTierToColorKey(tier as CpuTier);
    case 'memory':
      return memoryTierToColorKey(tier as MemoryTier);
    default:
      return 'tierC';
  }
}
