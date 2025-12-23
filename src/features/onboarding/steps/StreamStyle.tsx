import { useOnboardingStore } from '../../../stores/onboardingStore';
import type { StreamStyle } from '../../../types/commands';

/**
 * オンボーディング Step 3: 配信スタイル選択
 */
export function StreamStyleStep() {
  const { userPreferences, setUserPreferences } = useOnboardingStore();

  const handleSelect = (style: StreamStyle) => {
    setUserPreferences({ streamStyle: style });
  };

  return (
    <div className="space-y-6">
      <div className="text-center mb-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-2">配信スタイルを選択</h2>
        <p className="text-gray-600">
          主にどのような配信をされますか？最適な設定を提案します
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <StyleCard
          icon={
            <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
              />
            </svg>
          }
          title="雑談配信"
          description="トークメイン、Webカメラ使用"
          selected={userPreferences.streamStyle === 'talk'}
          onClick={() => handleSelect('talk')}
        />
        <StyleCard
          icon={
            <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
              />
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          }
          title="ゲーム配信"
          description="ゲーム画面がメイン、高FPS重視"
          selected={userPreferences.streamStyle === 'game'}
          onClick={() => handleSelect('game')}
        />
        <StyleCard
          icon={
            <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
              />
            </svg>
          }
          title="歌配信"
          description="高音質重視、映像はシンプル"
          selected={userPreferences.streamStyle === 'music'}
          onClick={() => handleSelect('music')}
        />
        <StyleCard
          icon={
            <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01"
              />
            </svg>
          }
          title="お絵描き配信"
          description="画面共有、細かい描画を美しく"
          selected={userPreferences.streamStyle === 'art'}
          onClick={() => handleSelect('art')}
        />
      </div>

      {userPreferences.streamStyle && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 text-center">
          <p className="text-sm text-blue-800">
            {getStyleDescription(userPreferences.streamStyle)}
          </p>
        </div>
      )}
    </div>
  );
}

// ========================================
// サブコンポーネント
// ========================================

interface StyleCardProps {
  icon: React.ReactNode;
  title: string;
  description: string;
  selected: boolean;
  onClick: () => void;
}

function StyleCard({ icon, title, description, selected, onClick }: StyleCardProps) {
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
      <div className={`mb-3 ${selected ? 'text-blue-600' : 'text-gray-600'}`}>{icon}</div>
      <h3 className="font-semibold text-gray-900 mb-1">{title}</h3>
      <p className="text-sm text-gray-600">{description}</p>
    </button>
  );
}

// ========================================
// ヘルパー関数
// ========================================

function getStyleDescription(style: StreamStyle): string {
  const descriptions: Record<StreamStyle, string> = {
    talk: '低ビットレートでも高品質な映像、音声優先の設定を提案します',
    game: '60FPS対応、動きの激しいシーンでも滑らかな映像設定を提案します',
    music: 'オーディオビットレート優先、音質を最大限に活かす設定を提案します',
    art: '高解像度対応、細かいディテールを再現する設定を提案します',
  };
  return descriptions[style];
}
