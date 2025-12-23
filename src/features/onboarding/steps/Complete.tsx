/**
 * オンボーディング Step 4: 完了画面
 */
export function Complete() {
  return (
    <div className="text-center py-4 animate-fade-in">
      <div className="mb-4">
        <div className="w-16 h-16 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center mx-auto mb-3 animate-scale-in">
          <svg
            className="w-8 h-8 text-green-600"
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path
              fillRule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
              clipRule="evenodd"
            />
          </svg>
        </div>
        <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100 mb-1">
          セットアップ完了！
        </h2>
        <p className="text-sm text-gray-600 dark:text-gray-400">
          OBS配信最適化ツールを今すぐ使い始められます
        </p>
      </div>

      <div className="max-w-2xl mx-auto space-y-4">
        {/* 主要機能の紹介 */}
        <div className="bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 border border-blue-200 dark:border-blue-700 rounded-lg p-4 text-left">
          <h3 className="font-semibold text-blue-900 dark:text-blue-200 mb-3 flex items-center gap-2 text-sm">
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            今すぐ使える機能
          </h3>
          <div className="grid grid-cols-2 gap-3">
            <FeatureCard icon="📊" title="リアルタイム監視" description="CPU・メモリ・GPU使用率を常時監視" />
            <FeatureCard icon="🎬" title="ワンクリック操作" description="配信・録画をアプリから直接制御" />
            <FeatureCard icon="🎯" title="シーン切り替え" description="複数のシーンを素早く切り替え" />
            <FeatureCard icon="📈" title="パフォーマンス分析" description="配信品質を数値で確認" />
          </div>
        </div>

        {/* クイックスタートガイド */}
        <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-700 rounded-lg p-4 text-left">
          <h3 className="font-semibold text-amber-900 dark:text-amber-200 mb-2 flex items-center gap-2 text-sm">
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
              <path d="M11 3a1 1 0 10-2 0v1a1 1 0 102 0V3zM15.657 5.757a1 1 0 00-1.414-1.414l-.707.707a1 1 0 001.414 1.414l.707-.707zM18 10a1 1 0 01-1 1h-1a1 1 0 110-2h1a1 1 0 011 1zM5.05 6.464A1 1 0 106.464 5.05l-.707-.707a1 1 0 00-1.414 1.414l.707.707zM5 10a1 1 0 01-1 1H3a1 1 0 110-2h1a1 1 0 011 1zM8 16v-1h4v1a2 2 0 11-4 0zM12 14c.015-.34.208-.646.477-.859a4 4 0 10-4.954 0c.27.213.462.519.476.859h4.002z" />
            </svg>
            次のステップ
          </h3>
          <ol className="space-y-2 text-xs text-amber-900 dark:text-amber-200">
            <li className="flex items-center gap-2">
              <span className="flex-shrink-0 w-5 h-5 bg-amber-200 dark:bg-amber-700 rounded-full flex items-center justify-center font-semibold text-xs">1</span>
              <span>メイン画面でOBSのステータスを確認</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="flex-shrink-0 w-5 h-5 bg-amber-200 dark:bg-amber-700 rounded-full flex items-center justify-center font-semibold text-xs">2</span>
              <span>システムメトリクスパネルでリソース使用状況をチェック</span>
            </li>
            <li className="flex items-center gap-2">
              <span className="flex-shrink-0 w-5 h-5 bg-amber-200 dark:bg-amber-700 rounded-full flex items-center justify-center font-semibold text-xs">3</span>
              <span>テスト配信で動作を確認してみましょう</span>
            </li>
          </ol>
        </div>

        {/* 完了ボタンへの誘導 */}
        <div className="text-center pt-2">
          <p className="text-gray-600 dark:text-gray-400 text-xs">
            下の「完了してメイン画面へ」ボタンをクリックして始めましょう！
          </p>
        </div>
      </div>
    </div>
  );
}

// ========================================
// サブコンポーネント
// ========================================

interface FeatureCardProps {
  icon: string;
  title: string;
  description: string;
}

function FeatureCard({ icon, title, description }: FeatureCardProps) {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg p-3 border border-blue-100 dark:border-blue-800">
      <div className="text-lg mb-1">{icon}</div>
      <h4 className="font-semibold text-gray-900 dark:text-gray-100 text-xs mb-0.5">{title}</h4>
      <p className="text-xs text-gray-600 dark:text-gray-400">{description}</p>
    </div>
  );
}
