import { useOnboardingStore } from '../../../stores/onboardingStore';
import type { StreamPlatform } from '../../../types/commands';

/**
 * オンボーディング Step 4: プラットフォーム選択
 */
export function PlatformStep() {
  const { userPreferences, setUserPreferences } = useOnboardingStore();

  const handleSelect = (platform: StreamPlatform) => {
    setUserPreferences({ platform });
  };

  return (
    <div className="space-y-6">
      <div className="text-center mb-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-2">配信プラットフォームを選択</h2>
        <p className="text-gray-600">
          主にどのプラットフォームで配信されますか？
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <PlatformCard
          icon="🎥"
          title="YouTube Live"
          description="高画質・高ビットレート推奨"
          selected={userPreferences.platform === 'youtube'}
          onClick={() => handleSelect('youtube')}
        />
        <PlatformCard
          icon="🎮"
          title="Twitch"
          description="低遅延・品質バランス重視"
          selected={userPreferences.platform === 'twitch'}
          onClick={() => handleSelect('twitch')}
        />
        <PlatformCard
          icon="📺"
          title="ニコニコ生放送"
          description="低ビットレート最適化"
          selected={userPreferences.platform === 'niconico'}
          onClick={() => handleSelect('niconico')}
        />
        <PlatformCard
          icon="🌐"
          title="その他"
          description="汎用設定"
          selected={userPreferences.platform === 'other'}
          onClick={() => handleSelect('other')}
        />
      </div>

      {userPreferences.platform && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 text-center">
          <p className="text-sm text-blue-800">
            {getPlatformDescription(userPreferences.platform)}
          </p>
        </div>
      )}
    </div>
  );
}

// ========================================
// サブコンポーネント
// ========================================

interface PlatformCardProps {
  icon: string;
  title: string;
  description: string;
  selected: boolean;
  onClick: () => void;
}

function PlatformCard({ icon, title, description, selected, onClick }: PlatformCardProps) {
  return (
    <button
      onClick={onClick}
      className={`
        p-6 rounded-lg border-2 transition-all text-left
        ${
          selected
            ? 'border-blue-500 bg-blue-50 shadow-md'
            : 'border-gray-200 bg-white hover:border-gray-300 hover:shadow'
        }
      `}
      aria-pressed={selected}
    >
      <div className="text-4xl mb-3">{icon}</div>
      <h3 className="font-semibold text-gray-900 mb-1">{title}</h3>
      <p className="text-sm text-gray-600">{description}</p>
    </button>
  );
}

// ========================================
// ヘルパー関数
// ========================================

function getPlatformDescription(platform: StreamPlatform): string {
  const descriptions: Record<StreamPlatform, string> = {
    youtube: 'ビットレート上限が高く、高画質配信に最適な設定を提案します',
    twitch: 'Twitch推奨のバランス型設定を提案します',
    niconico: 'ビットレート制限に対応した高効率エンコード設定を提案します',
    other: '汎用性の高いバランス型設定を提案します',
  };
  return descriptions[platform];
}
