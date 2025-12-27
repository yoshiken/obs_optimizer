import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useConfigStore } from '../../stores/configStore';
import { getEncoderDisplayLabel } from '../../utils/encoderLabels';
import { EncoderBadge } from '../../components/EncoderBadge';
import { SystemEvaluationSummary } from './SystemEvaluationSummary';
import type {
  AnalysisResult,
  AnalyzeSettingsRequest,
  KeyRecommendation,
  ObsSettings,
  StaticSettings,
  StreamingPlatform,
  StreamingStyle,
  SystemCapability,
  SystemInfo,
} from '../../types/commands';

// フロントエンド型からバックエンド型への変換
function convertPlatform(platform: string | null): StreamingPlatform | undefined {
  if (!platform) {
    return undefined;
  }
  const map: Record<string, StreamingPlatform> = {
    youtube: 'youTube',
    twitch: 'twitch',
    niconico: 'nicoNico',
    twitcasting: 'twitCasting',
    other: 'other',
  };
  return map[platform] ?? 'other';
}

function convertStyle(style: string | null): StreamingStyle | undefined {
  if (!style) {
    return undefined;
  }
  const map: Record<string, StreamingStyle> = {
    game: 'gaming',
    talk: 'talk',
    music: 'music',
    art: 'art',
  };
  return map[style] ?? 'gaming';
}

interface RecommendedSettingsPanelProps {
  /** 最適化適用後の設定更新トリガー（オプション） */
  refreshTrigger?: number;
}

/**
 * 推奨設定パネル
 *
 * 機能:
 * - PCスペック表示（折りたたみ可能）
 * - 現在値 vs 推奨値の比較テーブル
 * - 推奨理由リスト
 * - スコア表示（現在 → 推奨適用後）
 * - プラットフォーム/スタイル変更時のリアルタイム更新
 * - 最適化適用後の自動更新
 *
 * 使用するTauriコマンド:
 * - analyze_settings: 診断結果と推奨設定を取得
 * - get_obs_settings_command: 現在のOBS設定を取得
 */
export function RecommendedSettingsPanel({ refreshTrigger }: RecommendedSettingsPanelProps = {}) {
  const { config } = useConfigStore();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [analysisResult, setAnalysisResult] = useState<AnalysisResult | null>(null);
  const [currentSettings, setCurrentSettings] = useState<ObsSettings | null>(null);
  const [showHardwareInfo, setShowHardwareInfo] = useState(false);
  const [showStaticSettings, setShowStaticSettings] = useState(false);

  // Phase 5-6: システム能力評価と静的設定
  // バックエンドからの拡張レスポンスが利用可能になるまでモック値を使用
  const [systemCapability, setSystemCapability] = useState<SystemCapability | null>(null);

  // configStoreからプラットフォーム・スタイルを取得
  const platform = config?.platform ?? null;
  const streamStyle = config?.streamStyle ?? null;

  // 選択状態を判定
  const isConfigured = platform !== null && streamStyle !== null;

  // データ取得（プラットフォーム・スタイル選択後、または最適化適用後に再取得）
  useEffect(() => {
    // 未選択時はデータ取得しない
    if (!isConfigured) {
      setLoading(false);
      return;
    }

    const fetchData = async () => {
      try {
        setLoading(true);
        setError(null);

        // リクエストパラメータを構築
        const request: AnalyzeSettingsRequest = {
          platform: convertPlatform(platform),
          style: convertStyle(streamStyle),
        };

        // 並列でデータ取得
        const [analysis, current] = await Promise.all([
          invoke<AnalysisResult>('analyze_settings', { request }),
          invoke<ObsSettings>('get_obs_settings_command'),
        ]);

        setAnalysisResult(analysis);
        setCurrentSettings(current);

        // Phase 5-6: SystemCapabilityを設定
        // バックエンドからの直接取得を優先、なければモック生成にフォールバック
        if (analysis.systemCapability) {
          setSystemCapability(analysis.systemCapability);
        } else {
          const mockCapability = generateSystemCapability(analysis.systemInfo);
          setSystemCapability(mockCapability);
        }
      } catch (err) {
        console.error('推奨設定の取得に失敗:', err);
        setError(err instanceof Error ? err.message : '推奨設定の取得に失敗しました');
      } finally {
        setLoading(false);
      }
    };

    void fetchData();
  }, [platform, streamStyle, refreshTrigger, isConfigured]);

  // 未選択時はガイダンスを表示
  if (!isConfigured) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <div className="text-center py-12" role="status" aria-live="polite">
          <div className="text-4xl mb-4">🎯</div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
            推奨設定を表示するには
          </h3>
          <p className="text-gray-600 dark:text-gray-400">
            {!platform && !streamStyle
              ? 'まず配信プラットフォームと配信スタイルを選択してください'
              : !platform
                ? '配信プラットフォームを選択してください'
                : '配信スタイルを選択してください'}
          </p>
        </div>
      </div>
    );
  }

  // ローディング状態
  if (loading) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <div className="flex items-center justify-center py-12">
          <div className="text-center">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4" />
            <p className="text-gray-600 dark:text-gray-400">推奨設定を分析中...</p>
          </div>
        </div>
      </div>
    );
  }

  // エラー状態
  if (error || !analysisResult || !currentSettings) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-6">
        <div className="flex items-center gap-3 text-red-600 dark:text-red-400">
          <span className="text-2xl">⚠️</span>
          <div>
            <h3 className="font-semibold">推奨設定の取得に失敗</h3>
            <p className="text-sm mt-1">{error || 'データの取得に失敗しました'}</p>
          </div>
        </div>
      </div>
    );
  }

  const { systemInfo, summary, qualityScore } = analysisResult;
  const { headline, recommendedPreset, keyRecommendations } = summary;

  // 推奨適用後のスコアを算出（qualityScoreベース + 20%改善を想定）
  const potentialScore = Math.min(100, Math.round(qualityScore * 1.2));

  // プラットフォーム・スタイルの表示名
  const platformLabels: Record<string, string> = {
    youtube: 'YouTube',
    twitch: 'Twitch',
    niconico: 'ニコニコ',
    twitcasting: 'ツイキャス',
    other: 'その他',
  };
  const styleLabels: Record<string, string> = {
    game: 'ゲーム配信',
    talk: 'トーク配信',
    music: '音楽配信',
    art: 'お絵かき配信',
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
      {/* 選択状態の表示 */}
      <div className="bg-blue-50 dark:bg-blue-900/20 px-6 py-3 border-b border-blue-200 dark:border-blue-800">
        <div className="flex items-center gap-2 text-sm">
          <span className="text-blue-600 dark:text-blue-400">選択中:</span>
          <span className="font-medium text-gray-900 dark:text-gray-100">
            {platformLabels[platform] ?? platform} × {styleLabels[streamStyle] ?? streamStyle}
          </span>
          <span className="text-gray-600 dark:text-gray-400">に最適化された設定</span>
        </div>
      </div>

      {/* ヘッダー */}
      <div className="border-b border-gray-200 dark:border-gray-700 px-6 py-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              推奨設定
            </h2>
            <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">{headline}</p>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-sm text-gray-600 dark:text-gray-400">推奨プリセット:</span>
            <span className="px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 rounded-md text-sm font-medium">
              {recommendedPreset.toUpperCase()}
            </span>
          </div>
        </div>
      </div>

      <div className="p-6 space-y-6">
        {/* スコア表示 */}
        <ScoreComparison currentScore={qualityScore} potentialScore={potentialScore} />

        {/* Phase 5-6: システム評価サマリー（3カード表示） */}
        {systemCapability && (
          <SystemEvaluationSummary
            capability={systemCapability}
            defaultExpanded={true}
          />
        )}

        {/* PCスペック（折りたたみ可能）- レガシー表示 */}
        {!systemCapability && (
          <HardwareInfoSection
            systemInfo={systemInfo}
            isExpanded={showHardwareInfo}
            onToggle={() => setShowHardwareInfo(!showHardwareInfo)}
          />
        )}

        {/* 設定比較テーブル */}
        <SettingsComparisonTable
          current={currentSettings}
          recommendations={keyRecommendations}
        />

        {/* 推奨理由リスト */}
        <RecommendationReasons recommendations={keyRecommendations} />

        {/* Phase 5-6: 静的設定（ベストプラクティス）セクション */}
        <StaticSettingsSection
          isExpanded={showStaticSettings}
          onToggle={() => setShowStaticSettings(!showStaticSettings)}
        />
      </div>
    </div>
  );
}

// ========================================
// サブコンポーネント
// ========================================

/**
 * スコア比較表示
 */
interface ScoreComparisonProps {
  currentScore: number;
  potentialScore: number;
}

function ScoreComparison({ currentScore, potentialScore }: ScoreComparisonProps) {
  const improvement = potentialScore - currentScore;

  return (
    <div className="bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-950 dark:to-indigo-950 rounded-lg p-4 border border-blue-200 dark:border-blue-800">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <div className="text-center">
            <div className="text-sm text-gray-600 dark:text-gray-400 mb-1">現在</div>
            <div className="text-3xl font-bold text-gray-900 dark:text-gray-100">
              {currentScore}
            </div>
          </div>
          <div className="text-2xl text-gray-400 dark:text-gray-600">→</div>
          <div className="text-center">
            <div className="text-sm text-gray-600 dark:text-gray-400 mb-1">推奨適用後</div>
            <div className="text-3xl font-bold text-blue-600 dark:text-blue-400">
              {potentialScore}
            </div>
          </div>
        </div>
        <div className="text-right">
          <div className="text-sm text-gray-600 dark:text-gray-400 mb-1">改善予測</div>
          <div className="text-2xl font-bold text-green-600 dark:text-green-400">
            +{improvement}
          </div>
        </div>
      </div>
    </div>
  );
}

/**
 * PCスペック情報（折りたたみ可能）
 */
interface HardwareInfoSectionProps {
  systemInfo: SystemInfo;
  isExpanded: boolean;
  onToggle: () => void;
}

function HardwareInfoSection({ systemInfo, isExpanded, onToggle }: HardwareInfoSectionProps) {
  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
      <button
        onClick={onToggle}
        className="w-full px-4 py-3 bg-gray-50 dark:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors flex items-center justify-between"
        aria-expanded={isExpanded}
      >
        <span className="font-medium text-gray-900 dark:text-gray-100">PCスペック</span>
        <span className="text-gray-500 dark:text-gray-400 text-lg">
          {isExpanded ? '▲' : '▼'}
        </span>
      </button>

      {isExpanded && (
        <div className="px-4 py-3 space-y-2 bg-white dark:bg-gray-800">
          <InfoRow label="CPU" value={systemInfo.cpuModel} />
          <InfoRow
            label="GPU"
            value={systemInfo.gpuModel || '取得できませんでした'}
            valueClassName={!systemInfo.gpuModel ? 'text-gray-500 dark:text-gray-500' : ''}
          />
          <InfoRow
            label="メモリ"
            value={`${formatMemory(systemInfo.totalMemoryMb)} (利用可能: ${formatMemory(
              systemInfo.availableMemoryMb
            )})`}
          />
        </div>
      )}
    </div>
  );
}

/**
 * 設定比較テーブル
 */
interface SettingsComparisonTableProps {
  current: ObsSettings;
  recommendations: KeyRecommendation[];
}

function SettingsComparisonTable({
  current,
  recommendations,
}: SettingsComparisonTableProps) {
  // 現在値を取得
  const currentResolution = `${current.video.outputWidth}x${current.video.outputHeight}`;
  const currentFps = current.video.fpsDenominator !== 0
    ? Math.round((current.video.fpsNumerator / current.video.fpsDenominator) * 10) / 10
    : 0;
  const currentBitrate = `${Math.round(current.output.bitrateKbps / 1000)} Mbps`;
  // エンコーダーIDをユーザー向けラベルに変換
  const currentEncoder = getEncoderDisplayLabel(current.output.encoder);

  // 推奨値を取得（keyRecommendationsから該当するものを探す）
  const getRecommendedValue = (label: string): string => {
    const rec = recommendations.find((r) =>
      r.label.toLowerCase().includes(label.toLowerCase())
    );
    return rec ? rec.value : '-';
  };

  // 推奨エンコーダー情報を取得
  const recommendedEncoderValue = getRecommendedValue('エンコーダー');

  const rows: Array<{
    label: string;
    current: string;
    recommended: string;
    isEncoder: boolean;
    currentRawId?: string;
  }> = [
    {
      label: '解像度',
      current: currentResolution,
      recommended: getRecommendedValue('解像度'),
      isEncoder: false,
    },
    {
      label: 'FPS',
      current: `${currentFps} fps`,
      recommended: getRecommendedValue('fps'),
      isEncoder: false,
    },
    {
      label: 'ビットレート',
      current: currentBitrate,
      recommended: getRecommendedValue('ビットレート'),
      isEncoder: false,
    },
    {
      label: 'エンコーダー',
      current: currentEncoder,
      recommended: recommendedEncoderValue,
      isEncoder: true,
      currentRawId: current.output.encoder,
    },
  ];

  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
      <div className="bg-gray-50 dark:bg-gray-900 px-4 py-3">
        <h3 className="font-medium text-gray-900 dark:text-gray-100">設定比較</h3>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead className="bg-gray-100 dark:bg-gray-800">
            <tr>
              <th className="px-4 py-2 text-left text-sm font-medium text-gray-700 dark:text-gray-300">
                項目
              </th>
              <th className="px-4 py-2 text-left text-sm font-medium text-gray-700 dark:text-gray-300">
                現在値
              </th>
              <th className="px-4 py-2 text-left text-sm font-medium text-gray-700 dark:text-gray-300">
                推奨値
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
            {rows.map((row) => {
              const isDifferent = row.current !== row.recommended && row.recommended !== '-';
              return (
                <tr
                  key={row.label}
                  className={
                    isDifferent
                      ? 'bg-yellow-50 dark:bg-yellow-950/20'
                      : 'bg-white dark:bg-gray-800'
                  }
                >
                  <td className="px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100">
                    {row.label}
                  </td>
                  <td className="px-4 py-3 text-sm text-gray-700 dark:text-gray-300">
                    {row.isEncoder && row.currentRawId ? (
                      <EncoderBadge encoderId={row.currentRawId} showDetails />
                    ) : (
                      row.current
                    )}
                  </td>
                  <td className="px-4 py-3 text-sm font-medium text-blue-600 dark:text-blue-400">
                    {row.recommended}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

/**
 * 推奨理由リスト
 */
interface RecommendationReasonsProps {
  recommendations: KeyRecommendation[];
}

function RecommendationReasons({ recommendations }: RecommendationReasonsProps) {
  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
      <div className="bg-gray-50 dark:bg-gray-900 px-4 py-3">
        <h3 className="font-medium text-gray-900 dark:text-gray-100">推奨理由</h3>
      </div>
      <div className="p-4">
        <ul className="space-y-3">
          {recommendations.map((rec, index) => (
            <li key={index} className="flex gap-3">
              <span className="text-blue-600 dark:text-blue-400 flex-shrink-0 mt-0.5">✓</span>
              <div className="flex-1">
                <div className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  {rec.label}: {rec.value}
                </div>
                <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                  {rec.reasonSimple}
                </div>
              </div>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

// ========================================
// ユーティリティコンポーネント
// ========================================

interface InfoRowProps {
  label: string;
  value: string;
  valueClassName?: string;
}

function InfoRow({ label, value, valueClassName = '' }: InfoRowProps) {
  return (
    <div className="flex justify-between items-center">
      <span className="text-sm text-gray-600 dark:text-gray-400">{label}:</span>
      <span className={`text-sm text-gray-900 dark:text-gray-100 ${valueClassName}`}>
        {value}
      </span>
    </div>
  );
}

// ========================================
// ユーティリティ関数
// ========================================

function formatMemory(mb: number): string {
  if (mb >= 1024) {
    return `${Math.round((mb / 1024) * 10) / 10} GB`;
  }
  return `${Math.round(mb)} MB`;
}

// ========================================
// Phase 5-6: SystemCapability生成ヘルパー
// ========================================

/**
 * SystemInfoからSystemCapabilityを生成
 * 将来的にはバックエンドから直接取得する予定
 */
function generateSystemCapability(systemInfo: SystemInfo): SystemCapability {
  const gpuName = systemInfo.gpuModel ?? 'Unknown GPU';
  const memoryGb = systemInfo.totalMemoryMb / 1024;

  // GPUティアを推定
  const gpuTier = estimateGpuTier(gpuName);

  // CPUコア数を推定（cpuModelからは取得できないためデフォルト値）
  const cpuCores = 8; // デフォルト値
  const cpuTier = estimateCpuTier(cpuCores);

  // メモリティアを判定
  const memoryTier = estimateMemoryTier(memoryGb);

  // 各ティアのスコアを計算
  const gpuScore = getTierScore(gpuTier);
  const cpuScore = getCpuTierScore(cpuTier);
  const memScore = getMemoryTierScore(memoryTier);

  // 最小スコアで総合評価を決定
  const minScore = Math.min(gpuScore, cpuScore, memScore);
  const overallTier = scoreToOverallTier(minScore);

  // ボトルネックを判定
  const bottleneck = determineBottleneck(gpuScore, cpuScore, memScore);

  // 説明文を生成
  const description = generateDescription(overallTier, bottleneck);

  return {
    gpuTier,
    gpuName,
    cpuTier,
    cpuCores,
    memoryTier,
    memoryGb,
    overallTier,
    bottleneck,
    description,
  };
}

/**
 * 説明文を生成
 */
function generateDescription(tier: SystemCapability['overallTier'], bottleneck: SystemCapability['bottleneck']): string {
  const tierDescriptions: Record<SystemCapability['overallTier'], string> = {
    ultra: '1440p 60fpsで余裕を持って配信可能です',
    high: '1080p 60fpsで高品質な配信が可能です',
    medium: '1080p 60fpsで安定した配信が可能です',
    low: '720p 60fpsで基本的な配信が可能です',
    minimal: '720p 30fpsで軽量設定で配信可能です',
  };

  const bottleneckNote: Record<SystemCapability['bottleneck'], string> = {
    none: '',
    gpu: '（GPU性能が制限要因）',
    cpu: '（CPU性能が制限要因）',
    memory: '（メモリ容量が制限要因）',
  };

  return tierDescriptions[tier] + bottleneckNote[bottleneck];
}

/**
 * GPU名からティアを推定
 */
function estimateGpuTier(gpuName: string): SystemCapability['gpuTier'] {
  const lowerName = gpuName.toLowerCase();

  // RTX 50シリーズ / RTX 4090
  if (lowerName.includes('rtx 50') || lowerName.includes('rtx 4090')) {
    return 'tierS';
  }
  // RTX 4080 / RTX 4070 Ti / RTX 3090
  if (lowerName.includes('rtx 4080') || lowerName.includes('rtx 4070 ti') || lowerName.includes('rtx 3090')) {
    return 'tierA';
  }
  // RTX 4070 / RTX 3080 / RTX 3070
  if (lowerName.includes('rtx 4070') || lowerName.includes('rtx 3080') || lowerName.includes('rtx 3070')) {
    return 'tierB';
  }
  // RTX 4060 / RTX 3060 / RTX 2070
  if (lowerName.includes('rtx 4060') || lowerName.includes('rtx 3060') || lowerName.includes('rtx 2070')) {
    return 'tierC';
  }
  // GTX / その他
  return 'tierD';
}

/**
 * CPUコア数からティアを推定
 */
function estimateCpuTier(cores: number): SystemCapability['cpuTier'] {
  if (cores >= 16) {return 'highEnd';}
  if (cores >= 8) {return 'upperMiddle';}
  if (cores >= 6) {return 'middle';}
  return 'entry';
}

/**
 * メモリ容量からティアを推定
 */
function estimateMemoryTier(memoryGb: number): SystemCapability['memoryTier'] {
  if (memoryGb >= 32) {return 'abundant';}
  if (memoryGb >= 16) {return 'adequate';}
  if (memoryGb >= 8) {return 'standard';}
  return 'entry';
}

/**
 * GPUティアのスコアを取得
 */
function getTierScore(tier: SystemCapability['gpuTier']): number {
  const scores: Record<SystemCapability['gpuTier'], number> = {
    tierS: 100,
    tierA: 83,
    tierB: 67,
    tierC: 50,
    tierD: 33,
    tierE: 17,
  };
  return scores[tier] ?? 50;
}

/**
 * CPUティアのスコアを取得
 */
function getCpuTierScore(tier: SystemCapability['cpuTier']): number {
  const scores: Record<SystemCapability['cpuTier'], number> = {
    highEnd: 100,
    upperMiddle: 80,
    middle: 60,
    entry: 40,
  };
  return scores[tier] ?? 50;
}

/**
 * メモリティアのスコアを取得
 */
function getMemoryTierScore(tier: SystemCapability['memoryTier']): number {
  const scores: Record<SystemCapability['memoryTier'], number> = {
    abundant: 100,
    adequate: 80,
    standard: 60,
    entry: 40,
  };
  return scores[tier] ?? 50;
}

/**
 * スコアから総合ティアを決定
 */
function scoreToOverallTier(score: number): SystemCapability['overallTier'] {
  if (score >= 90) {return 'ultra';}
  if (score >= 70) {return 'high';}
  if (score >= 50) {return 'medium';}
  if (score >= 30) {return 'low';}
  return 'minimal';
}

/**
 * ボトルネック要因を判定
 */
function determineBottleneck(
  gpuScore: number,
  cpuScore: number,
  memScore: number
): SystemCapability['bottleneck'] {
  const minScore = Math.min(gpuScore, cpuScore, memScore);
  const threshold = 20; // 他と20ポイント以上差がある場合にボトルネック

  if (gpuScore === minScore && gpuScore < cpuScore - threshold && gpuScore < memScore - threshold) {
    return 'gpu';
  }
  if (cpuScore === minScore && cpuScore < gpuScore - threshold && cpuScore < memScore - threshold) {
    return 'cpu';
  }
  if (memScore === minScore && memScore < gpuScore - threshold && memScore < cpuScore - threshold) {
    return 'memory';
  }
  return 'none';
}

// ========================================
// Phase 5-6: 静的設定セクション
// ========================================

interface StaticSettingsSectionProps {
  isExpanded: boolean;
  onToggle: () => void;
}

/**
 * 静的設定（ベストプラクティス）セクション
 * スペックに依存しない固定推奨値を表示
 */
function StaticSettingsSection({ isExpanded, onToggle }: StaticSettingsSectionProps) {
  // デフォルトの静的設定
  const staticSettings: StaticSettings = {
    sampleRate: 48000,
    audioBitrate: 160,
    keyframeInterval: 2,
    rateControl: 'CBR',
    colorFormat: 'NV12',
    colorSpace: '709',
    colorRange: 'Partial',
    profile: 'high',
    bFrames: 2,
    lookAhead: false,
    psychoVisualTuning: true,
  };

  const settingGroups = [
    {
      title: '音声設定',
      icon: '🎵',
      items: [
        { label: 'サンプルレート', value: `${staticSettings.sampleRate} Hz`, reason: 'Windowsデフォルトと一致、リサンプル回避' },
        { label: '音声ビットレート', value: `${staticSettings.audioBitrate} kbps`, reason: '配信音声として十分な品質' },
      ],
    },
    {
      title: '映像基本設定',
      icon: '🎬',
      items: [
        { label: 'キーフレーム間隔', value: `${staticSettings.keyframeInterval}秒`, reason: '配信プラットフォームの推奨値' },
        { label: 'レートコントロール', value: staticSettings.rateControl, reason: '安定したビットレートで配信' },
        { label: 'プロファイル', value: staticSettings.profile, reason: '高品質エンコードに対応' },
      ],
    },
    {
      title: 'カラー設定',
      icon: '🎨',
      items: [
        { label: 'カラーフォーマット', value: staticSettings.colorFormat, reason: 'GPU処理に最適化' },
        { label: 'カラースペース', value: `Rec.${staticSettings.colorSpace}`, reason: 'HD/SDR配信の標準規格' },
        { label: 'カラーレンジ', value: staticSettings.colorRange, reason: '互換性を重視' },
      ],
    },
    {
      title: 'エンコーダー詳細',
      icon: '⚙️',
      items: [
        { label: 'Bフレーム', value: `${staticSettings.bFrames}`, reason: '圧縮効率と遅延のバランス' },
        { label: '先読み (Look-Ahead)', value: staticSettings.lookAhead ? '有効' : '無効', reason: '低遅延配信向け' },
        { label: 'Psycho Visual Tuning', value: staticSettings.psychoVisualTuning ? '有効' : '無効', reason: '知覚的品質を向上' },
      ],
    },
  ];

  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
      <button
        onClick={onToggle}
        className="w-full px-4 py-3 bg-gray-50 dark:bg-gray-900 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors flex items-center justify-between"
        aria-expanded={isExpanded}
        aria-controls="static-settings-content"
      >
        <div className="flex items-center gap-2">
          <span className="font-medium text-gray-900 dark:text-gray-100">ベストプラクティス設定</span>
          <span className="px-2 py-0.5 text-xs font-medium rounded-full bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-300">
            固定推奨
          </span>
        </div>
        <span className="text-gray-500 dark:text-gray-400 text-lg" aria-hidden="true">
          {isExpanded ? '▲' : '▼'}
        </span>
      </button>

      {isExpanded && (
        <div
          id="static-settings-content"
          className="p-4 bg-white dark:bg-gray-800 space-y-4"
        >
          <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
            以下の設定はPCスペックに関係なく、すべての配信者に推奨される固定値です。
          </p>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {settingGroups.map((group) => (
              <div
                key={group.title}
                className="border border-gray-200 dark:border-gray-700 rounded-lg p-3"
              >
                <div className="flex items-center gap-2 mb-3">
                  <span role="img" aria-hidden="true">{group.icon}</span>
                  <h4 className="font-medium text-gray-900 dark:text-gray-100">{group.title}</h4>
                </div>
                <ul className="space-y-2">
                  {group.items.map((item) => (
                    <li
                      key={item.label}
                      className="text-sm"
                    >
                      <div className="flex justify-between items-center">
                        <span className="text-gray-600 dark:text-gray-400">{item.label}</span>
                        <span className="font-medium text-gray-900 dark:text-gray-100">{item.value}</span>
                      </div>
                      <p className="text-xs text-gray-500 dark:text-gray-500 mt-0.5">{item.reason}</p>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
