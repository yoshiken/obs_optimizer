/**
 * オンボーディング Step 1: ようこそ画面
 */
export function Welcome() {
  return (
    <div className="text-center py-4">
      <div className="mb-4">
        <div className="w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center mx-auto mb-3">
          <svg
            className="w-8 h-8 text-blue-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        </div>
        <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100 mb-1">
          OBS配信最適化ツールへようこそ
        </h2>
        <p className="text-gray-600 dark:text-gray-400 text-sm">
          あなたの配信環境に最適な設定を簡単に見つけられます
        </p>
      </div>

      <div className="max-w-2xl mx-auto text-left space-y-3">
        <FeatureItem
          icon={
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          }
          title="自動診断"
          description="お使いのPCスペックと配信環境を自動で分析します"
        />
        <FeatureItem
          icon={
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M13 10V3L4 14h7v7l9-11h-7z"
              />
            </svg>
          }
          title="ワンクリック最適化"
          description="推奨設定をボタン一つで適用できます"
        />
        <FeatureItem
          icon={
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
              />
            </svg>
          }
          title="リアルタイム監視"
          description="配信中のCPU・メモリ使用率をモニタリングできます"
        />
      </div>

      <div className="mt-4 text-sm text-gray-500 dark:text-gray-400">
        所要時間: 約3分
      </div>
    </div>
  );
}

// ========================================
// サブコンポーネント
// ========================================

interface FeatureItemProps {
  icon: React.ReactNode;
  title: string;
  description: string;
}

function FeatureItem({ icon, title, description }: FeatureItemProps) {
  return (
    <div className="flex items-start gap-3 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
      <div className="flex-shrink-0 text-blue-600">{icon}</div>
      <div>
        <h3 className="font-semibold text-gray-900 dark:text-gray-100 text-sm">{title}</h3>
        <p className="text-xs text-gray-600 dark:text-gray-400">{description}</p>
      </div>
    </div>
  );
}
