import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useConfigStore } from '../../stores/configStore';
import { getEncoderDisplayLabel } from '../../utils/encoderLabels';
import { EncoderBadge } from '../../components/EncoderBadge';
import type {
  AnalysisResult,
  AnalyzeSettingsRequest,
  KeyRecommendation,
  ObsSettings,
  StreamingPlatform,
  StreamingStyle,
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

        {/* PCスペック（折りたたみ可能） */}
        <HardwareInfoSection
          systemInfo={systemInfo}
          isExpanded={showHardwareInfo}
          onToggle={() => setShowHardwareInfo(!showHardwareInfo)}
        />

        {/* 設定比較テーブル */}
        <SettingsComparisonTable
          current={currentSettings}
          recommendations={keyRecommendations}
        />

        {/* 推奨理由リスト */}
        <RecommendationReasons recommendations={keyRecommendations} />
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
