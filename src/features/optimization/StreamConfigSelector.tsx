import { useConfigStore } from '../../stores/configStore';
import type { StreamPlatform, StreamStyle } from '../../types/commands';

/**
 * 配信プラットフォームと配信スタイル選択コンポーネント
 *
 * 機能:
 * - プラットフォーム選択（YouTube, Twitch, ニコニコ, ツイキャス）
 * - 配信スタイル選択（ゲーム, 歌枠・音楽, 雑談, お絵描き）
 * - 選択状態を configStore に保存
 * - レスポンシブデザイン（モバイル2列、デスクトップ4列）
 */
export function StreamConfigSelector() {
  const { config, updateConfig } = useConfigStore();

  const platform = config?.platform || null;
  const streamStyle = config?.streamStyle || null;

  const handlePlatformChange = async (newPlatform: StreamPlatform) => {
    try {
      await updateConfig({ platform: newPlatform });
    } catch (error) {
      console.error('プラットフォーム設定の保存に失敗:', error);
    }
  };

  const handleStreamStyleChange = async (newStyle: StreamStyle) => {
    try {
      await updateConfig({ streamStyle: newStyle });
    } catch (error) {
      console.error('配信スタイル設定の保存に失敗:', error);
    }
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
      {/* ヘッダー */}
      <div className="border-b border-gray-200 dark:border-gray-700 px-6 py-4">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
          配信設定
        </h2>
        <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
          プラットフォームと配信スタイルを選択してください
        </p>
      </div>

      <div className="p-6 space-y-8">
        {/* プラットフォーム選択 */}
        <section>
          <h3 className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-4">
            配信プラットフォーム
          </h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <PlatformCard
              icon="📺"
              label="YouTube"
              specs="1080p60 / 51Mbps"
              features={['AV1対応', '高画質']}
              selected={platform === 'youtube'}
              onSelect={() => void handlePlatformChange('youtube')}
            />
            <PlatformCard
              icon="🎮"
              label="Twitch"
              specs="1080p60 / 6Mbps"
              features={['低遅延']}
              selected={platform === 'twitch'}
              onSelect={() => void handlePlatformChange('twitch')}
            />
            <PlatformCard
              icon="📹"
              label="ニコニコ"
              specs="720p30 / 6Mbps"
              features={['コメント機能']}
              selected={platform === 'niconico'}
              onSelect={() => void handlePlatformChange('niconico')}
            />
            <PlatformCard
              icon="📡"
              label="ツイキャス"
              specs="1080p60 / 60Mbps"
              features={['モバイル向け']}
              selected={platform === 'other'}
              onSelect={() => void handlePlatformChange('other')}
            />
          </div>
        </section>

        {/* 配信スタイル選択 */}
        <section>
          <h3 className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-4">
            配信スタイル
          </h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <StyleCard
              icon="🎮"
              label="ゲーム配信"
              description="高fps・高ビットレート"
              selected={streamStyle === 'game'}
              onSelect={() => void handleStreamStyleChange('game')}
            />
            <StyleCard
              icon="🎵"
              label="歌枠・音楽"
              description="音声品質重視"
              selected={streamStyle === 'music'}
              onSelect={() => void handleStreamStyleChange('music')}
            />
            <StyleCard
              icon="💬"
              label="雑談・トーク"
              description="低負荷設定"
              selected={streamStyle === 'talk'}
              onSelect={() => void handleStreamStyleChange('talk')}
            />
            <StyleCard
              icon="🎨"
              label="お絵描き"
              description="色精度重視"
              selected={streamStyle === 'art'}
              onSelect={() => void handleStreamStyleChange('art')}
            />
          </div>
        </section>
      </div>
    </div>
  );
}

// ========================================
// サブコンポーネント
// ========================================

/**
 * プラットフォームカード
 */
interface PlatformCardProps {
  icon: string;
  label: string;
  specs: string;
  features: string[];
  selected: boolean;
  onSelect: () => void;
}

function PlatformCard({
  icon,
  label,
  specs,
  features,
  selected,
  onSelect,
}: PlatformCardProps) {
  return (
    <button
      onClick={onSelect}
      className={`
        relative p-4 rounded-lg border-2 transition-all text-left
        ${
          selected
            ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 shadow-md'
            : 'border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 hover:border-gray-300 dark:hover:border-gray-600 hover:shadow-sm'
        }
      `}
      aria-pressed={selected}
      aria-label={`${label}を選択`}
    >
      {/* 選択チェックマーク */}
      {selected && (
        <div className="absolute top-2 right-2">
          <CheckIcon className="w-5 h-5 text-blue-500" />
        </div>
      )}

      {/* アイコン */}
      <div className="text-2xl mb-2">{icon}</div>

      {/* ラベル */}
      <div className="font-semibold text-gray-900 dark:text-gray-100 mb-1">{label}</div>

      {/* スペック */}
      <div className="text-xs text-gray-500 dark:text-gray-400 mb-2">{specs}</div>

      {/* 特徴タグ */}
      <div className="flex flex-wrap gap-1">
        {features.map((feature) => (
          <span
            key={feature}
            className="px-2 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-full"
          >
            {feature}
          </span>
        ))}
      </div>
    </button>
  );
}

/**
 * 配信スタイルカード
 */
interface StyleCardProps {
  icon: string;
  label: string;
  description: string;
  selected: boolean;
  onSelect: () => void;
}

function StyleCard({ icon, label, description, selected, onSelect }: StyleCardProps) {
  return (
    <button
      onClick={onSelect}
      className={`
        relative p-4 rounded-lg border-2 transition-all text-left
        ${
          selected
            ? 'border-purple-500 bg-purple-50 dark:bg-purple-900/20 shadow-md'
            : 'border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 hover:border-gray-300 dark:hover:border-gray-600 hover:shadow-sm'
        }
      `}
      aria-pressed={selected}
      aria-label={`${label}を選択`}
    >
      {/* 選択チェックマーク */}
      {selected && (
        <div className="absolute top-2 right-2">
          <CheckIcon className="w-5 h-5 text-purple-500" />
        </div>
      )}

      {/* アイコン */}
      <div className="text-2xl mb-2">{icon}</div>

      {/* ラベル */}
      <div className="font-semibold text-gray-900 dark:text-gray-100 mb-1">{label}</div>

      {/* 説明 */}
      <div className="text-xs text-gray-500 dark:text-gray-400">{description}</div>
    </button>
  );
}

/**
 * チェックアイコン
 */
interface CheckIconProps {
  className?: string;
}

function CheckIcon({ className = '' }: CheckIconProps) {
  return (
    <svg
      className={className}
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
      strokeWidth={2}
    >
      <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
    </svg>
  );
}
