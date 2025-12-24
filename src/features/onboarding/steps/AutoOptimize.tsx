import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { OptimizationPreset, OptimizationResult } from '../../../types/commands';
import { useOnboardingStore } from '../../../stores/onboardingStore';

/**
 * オンボーディング Step 6: 推奨設定適用確認
 */
export function AutoOptimize() {
  const { userPreferences } = useOnboardingStore();
  const [selectedPreset, setSelectedPreset] = useState<OptimizationPreset>('medium');
  const [applying, setApplying] = useState(false);
  const [result, setResult] = useState<OptimizationResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleApply = async () => {
    setApplying(true);
    setError(null);

    try {
      const optimizationResult = await invoke<OptimizationResult>('apply_optimization', {
        params: { preset: selectedPreset },
      });
      setResult(optimizationResult);
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      setError(errorMessage);
    } finally {
      setApplying(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="text-center mb-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-2">推奨設定を適用</h2>
        <p className="text-gray-600">
          分析結果に基づいて最適な設定を適用します
        </p>
      </div>

      {/* ユーザー設定のサマリー */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <h3 className="font-semibold text-blue-900 mb-2">あなたの配信スタイル</h3>
        <div className="text-sm text-blue-800 space-y-1">
          <p>配信スタイル: {getStyleLabel(userPreferences.streamStyle)}</p>
          <p>プラットフォーム: {getPlatformLabel(userPreferences.platform)}</p>
        </div>
      </div>

      {/* プリセット選択 */}
      {!result && (
        <div className="space-y-3">
          <h3 className="font-semibold text-gray-900">最適化レベルを選択</h3>
          <div className="space-y-2">
            <PresetOption
              preset="low"
              label="軽量（低負荷優先）"
              description="PCへの負荷を最小限に、安定性重視"
              details={[
                '画質: 720p (1280x720)',
                'フレームレート: 30fps',
                '画質の目安: 3000kbps程度',
                '処理方法: CPUを使う（x264）',
              ]}
              selected={selectedPreset === 'low'}
              onClick={() => setSelectedPreset('low')}
            />
            <PresetOption
              preset="medium"
              label="標準（推奨）"
              description="バランスの取れた設定"
              details={[
                '画質: 1080p (1920x1080)',
                'フレームレート: 60fps',
                '画質の目安: 6000kbps程度',
                '処理方法: GPUを使う（NVENC/QuickSync）',
              ]}
              selected={selectedPreset === 'medium'}
              onClick={() => setSelectedPreset('medium')}
            />
            <PresetOption
              preset="high"
              label="高品質"
              description="画質優先、ハイスペックPC向け"
              details={[
                '画質: 1080p (1920x1080)',
                'フレームレート: 60fps',
                '画質の目安: 9000kbps程度',
                '処理方法: 高品質GPUエンコーディング',
                '注意: ハイスペックPC推奨',
              ]}
              selected={selectedPreset === 'high'}
              onClick={() => setSelectedPreset('high')}
            />
          </div>

          <button
            onClick={() => void handleApply()}
            disabled={applying}
            className="w-full mt-4 px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-semibold"
          >
            {applying ? '適用中...' : '設定を適用する'}
          </button>

          {/* スキップオプション */}
          <div className="text-center pt-2">
            <button className="text-gray-500 hover:text-gray-700 text-sm underline">
              あとで自分で設定する（スキップ）
            </button>
          </div>
        </div>
      )}

      {/* 適用結果 */}
      {result && (
        <div className="bg-green-50 border border-green-200 rounded-lg p-6">
          <div className="flex items-center justify-center gap-2 text-green-800 mb-4">
            <svg className="w-8 h-8" fill="currentColor" viewBox="0 0 20 20">
              <path
                fillRule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                clipRule="evenodd"
              />
            </svg>
            <span className="text-xl font-semibold">設定を適用しました！</span>
          </div>
          <div className="text-sm text-green-800 space-y-1 text-center">
            <p>{result.appliedCount}件の設定を適用しました</p>
            {result.failedCount > 0 && (
              <p className="text-yellow-700">{result.failedCount}件の設定は適用できませんでした</p>
            )}
          </div>
        </div>
      )}

      {/* エラー表示 */}
      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <p className="text-sm text-red-800">{error}</p>
        </div>
      )}
    </div>
  );
}

// ========================================
// サブコンポーネント
// ========================================

interface PresetOptionProps {
  preset: OptimizationPreset;
  label: string;
  description: string;
  details: string[];
  selected: boolean;
  onClick: () => void;
}

function PresetOption({ label, description, details, selected, onClick }: PresetOptionProps) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div
      className={`
        w-full rounded-lg border-2 transition-all
        ${
          selected
            ? 'border-blue-500 bg-blue-50'
            : 'border-gray-200 bg-white hover:border-gray-300'
        }
      `}
    >
      <button
        onClick={onClick}
        className="w-full p-4 text-left"
        aria-pressed={selected}
      >
        <div className="flex items-center gap-3">
          <div
            className={`w-5 h-5 rounded-full border-2 flex items-center justify-center flex-shrink-0 ${
              selected ? 'border-blue-500' : 'border-gray-300'
            }`}
          >
            {selected && <div className="w-3 h-3 rounded-full bg-blue-500" />}
          </div>
          <div className="flex-1">
            <h4 className="font-semibold text-gray-900">{label}</h4>
            <p className="text-sm text-gray-600">{description}</p>
          </div>
          <button
            onClick={(e) => {
              e.stopPropagation();
              setExpanded(!expanded);
            }}
            className="text-sm text-blue-600 hover:text-blue-800 px-2"
          >
            {expanded ? '閉じる' : '詳細'}
          </button>
        </div>
      </button>

      {expanded && (
        <div className="px-4 pb-4 border-t border-gray-200 mt-2 pt-3">
          <p className="text-xs font-medium text-gray-700 mb-2">この設定で適用される内容:</p>
          <ul className="space-y-1">
            {details.map((detail, index) => (
              <li key={index} className="text-xs text-gray-600 flex items-start gap-2">
                <span className="text-blue-500 mt-0.5">•</span>
                <span>{detail}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

// ========================================
// ヘルパー関数
// ========================================

function getStyleLabel(style: string | null): string {
  if (!style) {return '未選択';}
  const labels: Record<string, string> = {
    talk: '雑談配信',
    game: 'ゲーム配信',
    music: '歌配信',
    art: 'お絵描き配信',
  };
  return labels[style] || '未選択';
}

function getPlatformLabel(platform: string | null): string {
  if (!platform) {return '未選択';}
  const labels: Record<string, string> = {
    youtube: 'YouTube Live',
    twitch: 'Twitch',
    niconico: 'ニコニコ生放送',
    other: 'その他',
  };
  return labels[platform] || '未選択';
}
