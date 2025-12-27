export { OneClickApply } from './OneClickApply';
export { BackupRestore } from './BackupRestore';
export { ComponentTierCard } from './ComponentTierCard';
export { SystemEvaluationSummary } from './SystemEvaluationSummary';
export { RecommendedSettingsPanel } from './RecommendedSettingsPanel';

// ティア関連ユーティリティ
export {
  cpuTierToColorKey,
  getOverallTierDescription,
  getOverallTierLabel,
  getTierLabel,
  getTierScore,
  memoryTierToColorKey,
  normalizeToColorKey,
  overallTierToColorKey,
  TIER_COLORS,
} from './tierUtils';
export type { TierColorKey, TierColors } from './tierUtils';
