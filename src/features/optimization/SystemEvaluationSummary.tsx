import { useState } from 'react';
import { ComponentTierCard } from './ComponentTierCard';
import {
  getOverallTierDescription,
  getOverallTierLabel,
  getTierLabel,
  getTierScore,
  overallTierToColorKey,
  TIER_COLORS,
} from './tierUtils';
import type {
  BottleneckFactor,
  SystemCapability,
} from '../../types/commands';

// ========================================
// 型定義
// ========================================

interface SystemEvaluationSummaryProps {
  /** システム能力評価データ */
  capability: SystemCapability;
  /** 折りたたみ初期状態 */
  defaultExpanded?: boolean;
  /** コンパクト表示モード */
  compact?: boolean;
}

// ========================================
// ボトルネック関連ユーティリティ
// ========================================

/**
 * ボトルネック要因の日本語ラベルを取得
 */
function getBottleneckLabel(bottleneck: BottleneckFactor): string | null {
  const labels: Record<BottleneckFactor, string | null> = {
    none: null,
    gpu: 'GPU',
    cpu: 'CPU',
    memory: 'メモリ',
  };
  return labels[bottleneck] ?? null;
}

/**
 * CPUティアから能力一覧を生成
 */
function getCpuCapabilities(cpuTier: string, cores: number): string[] {
  const capabilities: string[] = [];

  if (cores >= 16) {
    capabilities.push('x264 slow対応');
  } else if (cores >= 8) {
    capabilities.push('x264 medium対応');
  } else if (cores >= 6) {
    capabilities.push('x264 veryfast');
  } else {
    capabilities.push('GPUエンコード推奨');
  }

  if (cpuTier === 'highEnd' || cpuTier === 'upperMiddle') {
    capabilities.push('マルチタスク余裕');
  }

  return capabilities;
}

/**
 * メモリティアから能力一覧を生成
 */
function getMemoryCapabilities(_memoryTier: string, memoryGb: number): string[] {
  const capabilities: string[] = [];

  if (memoryGb >= 32) {
    capabilities.push('4K/1440p対応');
    capabilities.push('複数アプリ同時起動');
  } else if (memoryGb >= 16) {
    capabilities.push('1080p60安定');
    capabilities.push('ゲーム+配信+Discord');
  } else if (memoryGb >= 8) {
    capabilities.push('720p/1080p30');
  } else {
    capabilities.push('ブラウザ制限推奨');
  }

  return capabilities;
}

/**
 * GPUティアから能力一覧を生成
 */
function getGpuCapabilities(gpuTier: string, gpuName: string): string[] {
  const capabilities: string[] = [];
  const lowerName = gpuName.toLowerCase();

  // NVENC対応チェック
  if (lowerName.includes('rtx') || lowerName.includes('gtx 16') || lowerName.includes('gtx 20')) {
    capabilities.push('NVENC');
  }

  // AV1対応チェック（RTX 40シリーズ以上）
  if (lowerName.includes('rtx 40') || lowerName.includes('rtx 50')) {
    capabilities.push('AV1');
    capabilities.push('HEVC');
  } else if (lowerName.includes('rtx 30') || lowerName.includes('rtx 20')) {
    capabilities.push('HEVC');
  }

  // AMD対応チェック
  if (lowerName.includes('rx 7') || lowerName.includes('rx 6')) {
    capabilities.push('AMF');
    if (lowerName.includes('rx 7')) {
      capabilities.push('AV1');
    }
  }

  // Intel Arc対応チェック
  if (lowerName.includes('arc')) {
    capabilities.push('QSV');
    capabilities.push('AV1');
  }

  // ティアに応じた品質表示
  if (gpuTier === 'tierS' || gpuTier === 'tierA') {
    capabilities.push('高品質エンコード');
  }

  return capabilities;
}

// ========================================
// メインコンポーネント
// ========================================

/**
 * システム評価サマリーコンポーネント
 *
 * 機能:
 * - GPU/CPU/Memoryの3カード表示
 * - 総合評価ティア表示
 * - ボトルネック警告
 * - 折りたたみ可能
 * - レスポンシブ対応（1-3列）
 * - ダークモード対応
 *
 * 使用例:
 * ```tsx
 * <SystemEvaluationSummary
 *   capability={systemCapability}
 *   defaultExpanded={true}
 * />
 * ```
 */
export function SystemEvaluationSummary({
  capability,
  defaultExpanded = true,
  compact = false,
}: SystemEvaluationSummaryProps) {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);

  const {
    gpuTier,
    gpuName,
    cpuTier,
    cpuCores,
    memoryTier,
    memoryGb,
    overallTier,
    bottleneck,
  } = capability;

  // 総合評価のカラーキー
  const overallColorKey = overallTierToColorKey(overallTier);
  const overallColors = TIER_COLORS[overallColorKey];
  const overallScore = getTierScore(overallColorKey);
  const bottleneckLabel = getBottleneckLabel(bottleneck);

  // 各コンポーネントの能力一覧を生成
  const gpuCapabilities = getGpuCapabilities(gpuTier, gpuName);
  const cpuCapabilities = getCpuCapabilities(cpuTier, cpuCores);
  const memoryCapabilities = getMemoryCapabilities(memoryTier, memoryGb);

  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
      {/* ヘッダー（折りたたみボタン） */}
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full px-4 py-3 bg-gray-50 dark:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors flex items-center justify-between"
        aria-expanded={isExpanded}
        aria-controls="system-evaluation-content"
      >
        <div className="flex items-center gap-3">
          <span className="font-medium text-gray-900 dark:text-gray-100">システム評価</span>
          {/* コンパクトビューでは総合ティアをヘッダーに表示 */}
          <span
            className={`
              px-2 py-0.5 text-xs font-medium rounded-full
              ${overallColors.bg} ${overallColors.bgDark}
              ${overallColors.text} ${overallColors.textDark}
            `}
          >
            {getOverallTierLabel(overallTier)}
          </span>
          {/* ボトルネック警告 */}
          {bottleneckLabel && (
            <span className="flex items-center gap-1 px-2 py-0.5 text-xs font-medium rounded-full bg-amber-100 dark:bg-amber-900 text-amber-700 dark:text-amber-300">
              <span aria-hidden="true">⚠️</span>
              {bottleneckLabel}がボトルネック
            </span>
          )}
        </div>
        <span className="text-gray-500 dark:text-gray-400 text-lg" aria-hidden="true">
          {isExpanded ? '▲' : '▼'}
        </span>
      </button>

      {/* コンテンツ（展開時のみ表示） */}
      {isExpanded && (
        <div
          id="system-evaluation-content"
          className="p-4 bg-white dark:bg-gray-800 space-y-4"
        >
          {/* 3カードグリッド */}
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {/* GPUカード */}
            <ComponentTierCard
              type="gpu"
              name={gpuName || 'GPU情報なし'}
              tier={gpuTier}
              capabilities={gpuCapabilities}
              compact={compact}
            />

            {/* CPUカード */}
            <ComponentTierCard
              type="cpu"
              name={`${cpuCores}コア CPU`}
              tier={cpuTier}
              detail={`${cpuCores}コア`}
              capabilities={cpuCapabilities}
              compact={compact}
            />

            {/* メモリカード */}
            <ComponentTierCard
              type="memory"
              name={`${Math.round(memoryGb)}GB RAM`}
              tier={memoryTier}
              detail={`${Math.round(memoryGb)}GB`}
              capabilities={memoryCapabilities}
              compact={compact}
            />
          </div>

          {/* 総合評価バー */}
          <div
            className={`
              rounded-lg p-4 border
              ${overallColors.bg} ${overallColors.bgDark}
              ${overallColors.border} ${overallColors.borderDark}
            `}
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                <span className="text-lg" role="img" aria-hidden="true">
                  📊
                </span>
                <span className={`font-semibold ${overallColors.text} ${overallColors.textDark}`}>
                  総合評価: {getTierLabel(overallColorKey)}
                </span>
              </div>
              <span className={`text-sm ${overallColors.text} ${overallColors.textDark}`}>
                {getOverallTierLabel(overallTier)}
              </span>
            </div>

            {/* プログレスバー */}
            <div
              className="w-full h-3 bg-white/50 dark:bg-gray-700/50 rounded-full overflow-hidden mb-2"
              role="progressbar"
              aria-valuenow={overallScore}
              aria-valuemin={0}
              aria-valuemax={100}
              aria-label={`総合スコア: ${overallScore}%`}
            >
              <div
                className={`h-full rounded-full transition-all duration-700 ${overallColors.bar} ${overallColors.barDark}`}
                style={{ width: `${overallScore}%` }}
              />
            </div>

            {/* 説明文 */}
            <p className="text-sm text-gray-600 dark:text-gray-400">
              {getOverallTierDescription(overallTier)}
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
