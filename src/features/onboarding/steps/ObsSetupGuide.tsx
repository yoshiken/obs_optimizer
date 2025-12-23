import { useState } from 'react';

/**
 * オンボーディング Step 2: OBS WebSocket設定ガイド
 *
 * スクリーンショット説明付きでWebSocket設定方法を案内
 */
export function ObsSetupGuide() {
  const [currentGuideStep, setCurrentGuideStep] = useState(1);
  const totalGuideSteps = 5;

  return (
    <div className="space-y-4">
      <div className="text-center mb-3">
        <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100 mb-1">
          OBS WebSocket設定
        </h2>
        <p className="text-gray-600 dark:text-gray-400 text-sm">
          OBSのWebSocketサーバーを有効にして、このツールから接続できるようにします
        </p>
      </div>

      {/* ガイドステップインジケーター */}
      <div className="flex items-center justify-center gap-2 mb-3">
        {Array.from({ length: totalGuideSteps }, (_, i) => i + 1).map((step) => (
          <button
            key={step}
            onClick={() => setCurrentGuideStep(step)}
            className={`w-2.5 h-2.5 rounded-full transition-colors ${
              step === currentGuideStep
                ? 'bg-blue-600'
                : step < currentGuideStep
                ? 'bg-blue-300'
                : 'bg-gray-300 dark:bg-gray-600'
            }`}
            aria-label={`ガイドステップ ${step}`}
          />
        ))}
      </div>

      {/* ガイドコンテンツ */}
      <div className="bg-white dark:bg-gray-800 border-2 border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
        {/* 説明エリア */}
        <div className="p-4 bg-gray-50 dark:bg-gray-700 border-b border-gray-200 dark:border-gray-600">
          <div className="flex items-start gap-3">
            <div className="flex-shrink-0 w-7 h-7 bg-blue-600 text-white rounded-full flex items-center justify-center font-bold text-sm">
              {currentGuideStep}
            </div>
            <div className="flex-1">
              <h3 className="font-semibold text-gray-900 dark:text-gray-100 text-sm mb-0.5">
                {getGuideStepTitle(currentGuideStep)}
              </h3>
              <p className="text-xs text-gray-700 dark:text-gray-300">
                {getGuideStepDescription(currentGuideStep)}
              </p>
            </div>
          </div>
        </div>

        {/* スクリーンショット説明エリア */}
        <div className="p-4">
          <div className="bg-gray-100 dark:bg-gray-700 rounded-lg p-4 flex items-center justify-center">
            <div className="text-center">
              {/* 実際のスクリーンショットの代わりにプレースホルダー */}
              <div className="mb-2">
                {getGuideStepIllustration(currentGuideStep)}
              </div>
              <p className="text-xs text-gray-600 dark:text-gray-400 max-w-md mx-auto">
                {getGuideStepNote(currentGuideStep)}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* ナビゲーションボタン */}
      <div className="flex items-center justify-between pt-2">
        <button
          onClick={() => setCurrentGuideStep(Math.max(1, currentGuideStep - 1))}
          disabled={currentGuideStep === 1}
          className="px-3 py-1.5 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors text-sm"
        >
          前のステップ
        </button>

        {currentGuideStep < totalGuideSteps ? (
          <button
            onClick={() => setCurrentGuideStep(Math.min(totalGuideSteps, currentGuideStep + 1))}
            className="px-3 py-1.5 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors text-sm"
          >
            次のステップ
          </button>
        ) : (
          <div className="text-sm text-green-600 dark:text-green-400 font-semibold">
            設定完了！次へ進んでください
          </div>
        )}
      </div>

      {/* 重要な注意事項 */}
      <div className="bg-yellow-50 dark:bg-yellow-900/30 border border-yellow-200 dark:border-yellow-700 rounded-lg p-3">
        <div className="flex items-start gap-2">
          <svg
            className="w-4 h-4 text-yellow-600 flex-shrink-0 mt-0.5"
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path
              fillRule="evenodd"
              d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
              clipRule="evenodd"
            />
          </svg>
          <div className="text-xs text-yellow-800 dark:text-yellow-200">
            <span className="font-semibold">重要: </span>
            OBSのバージョンが28.0.0以降であることを確認してください。
          </div>
        </div>
      </div>
    </div>
  );
}

// ========================================
// ヘルパー関数
// ========================================

function getGuideStepTitle(step: number): string {
  switch (step) {
    case 1:
      return 'OBSを起動';
    case 2:
      return 'メニューから設定を開く';
    case 3:
      return 'WebSocketサーバー設定を開く';
    case 4:
      return 'WebSocketサーバーを有効化';
    case 5:
      return '接続情報を確認';
    default:
      return '';
  }
}

function getGuideStepDescription(step: number): string {
  switch (step) {
    case 1:
      return 'OBS Studio を起動してください。まだインストールしていない場合は、公式サイトからダウンロードできます。';
    case 2:
      return '上部メニューバーから「ツール」をクリックします。';
    case 3:
      return 'ツールメニューから「WebSocketサーバー設定」を選択します。';
    case 4:
      return '「WebSocketサーバーを有効にする」にチェックを入れます。';
    case 5:
      return 'サーバーポート（デフォルト: 4455）とパスワード（設定されている場合）を確認します。';
    default:
      return '';
  }
}

function getGuideStepIllustration(step: number): React.ReactNode {
  // 実際のスクリーンショットの代わりにSVGアイコンで表現
  const iconClass = "w-16 h-16 text-blue-600 mx-auto";
  switch (step) {
    case 1:
      return (
        <svg className={iconClass} fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
            d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
        </svg>
      );
    case 2:
      return (
        <svg className={iconClass} fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
        </svg>
      );
    case 3:
      return (
        <svg className={iconClass} fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
            d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
      );
    case 4:
      return (
        <svg className={iconClass} fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      );
    case 5:
      return (
        <svg className={iconClass} fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      );
    default:
      return null;
  }
}

function getGuideStepNote(step: number): string {
  switch (step) {
    case 1:
      return 'OBS Studio のアイコンをダブルクリックして起動します';
    case 2:
      return '画面上部のメニューバーに「ツール」があります';
    case 3:
      return 'WebSocketサーバー設定のダイアログが開きます';
    case 4:
      return 'チェックを入れるとWebSocketサーバーが起動します';
    case 5:
      return 'この情報を次のステップで使用します';
    default:
      return '';
  }
}
