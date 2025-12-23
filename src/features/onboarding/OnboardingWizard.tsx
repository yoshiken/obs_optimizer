import { useEffect } from 'react';
import { REQUIRED_STEPS, TOTAL_STEPS, useOnboardingStore } from '../../stores/onboardingStore';
import { useConfigStore } from '../../stores/configStore';
import { useObsStore } from '../../stores/obsStore';
import { Welcome } from './steps/Welcome';
import { ObsSetupGuide } from './steps/ObsSetupGuide';
import { ConnectionTest } from './steps/ConnectionTest';
import { Complete } from './steps/Complete';

/**
 * オンボーディングウィザード
 *
 * 4ステップのシンプルなセットアップフロー:
 * 1. ウェルカム画面
 * 2. OBS WebSocket設定ガイド
 * 3. 接続テスト
 * 4. 完了画面
 */
export function OnboardingWizard() {
  const { currentStep, nextStep, prevStep, setStep, completeOnboarding } =
    useOnboardingStore();
  const { updateConfig } = useConfigStore();
  const { connectionState } = useObsStore();

  // OBS接続状態によって自動で次のステップに進む（Step 3のみ）
  useEffect(() => {
    if (currentStep === 3 && connectionState === 'connected') {
      // 少し遅延させてユーザーが接続成功メッセージを見れるようにする
      const timer = setTimeout(() => {
        // ユーザーがまだStep 3にいる場合のみ進む
        if (currentStep === 3) {
          nextStep();
        }
      }, 2000);
      return () => clearTimeout(timer);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [connectionState, currentStep]);

  // ステップコンポーネントのマッピング
  const renderStep = () => {
    switch (currentStep) {
      case 1:
        return <Welcome />;
      case 2:
        return <ObsSetupGuide />;
      case 3:
        return <ConnectionTest />;
      case 4:
        return <Complete />;
      default:
        return <Welcome />;
    }
  };

  // 次へボタンの有効状態
  const canGoNext = () => {
    switch (currentStep) {
      case 1:
        // ウェルカム画面は常に進める
        return true;
      case 2:
        // 設定ガイドは常に進める（説明を読むだけ）
        return true;
      case 3:
        // OBS接続テストは接続成功が必須
        return connectionState === 'connected';
      case 4:
        // 完了画面
        return true;
      default:
        return true;
    }
  };

  // 完了処理
  const handleComplete = async () => {
    try {
      // オンボーディング完了フラグを保存
      await updateConfig({
        onboardingCompleted: true,
      });
      completeOnboarding();
    } catch (e) {
      // エラーはストアで処理される
      console.error('設定の保存に失敗しました:', e);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-white to-indigo-50 flex items-center justify-center p-4">
      <div className="max-w-4xl w-full bg-white rounded-xl shadow-2xl overflow-hidden transition-all duration-300">
        {/* プログレスバー */}
        <div className="px-8 pt-8 pb-4 bg-gradient-to-r from-blue-500 to-indigo-600">
          <div className="flex items-center justify-between mb-4">
            {Array.from({ length: TOTAL_STEPS }, (_, i) => i + 1).map((step) => (
              <div key={step} className="flex-1 flex items-center">
                <button
                  onClick={() => {
                    // 完了済みのステップにはジャンプできる（ただし必須ステップを超えない）
                    if (step < currentStep || !REQUIRED_STEPS.includes(step)) {
                      setStep(step);
                    }
                  }}
                  disabled={step > currentStep}
                  className={`
                    w-12 h-12 rounded-full flex items-center justify-center font-bold text-sm
                    transition-all duration-300 transform
                    ${
                      step === currentStep
                        ? 'bg-white text-blue-600 scale-110 shadow-lg'
                        : step < currentStep
                        ? 'bg-green-400 text-white cursor-pointer hover:bg-green-500 hover:scale-105'
                        : 'bg-blue-300 bg-opacity-50 text-blue-200 cursor-not-allowed'
                    }
                  `}
                  aria-label={`ステップ ${step}`}
                  aria-current={step === currentStep ? 'step' : undefined}
                >
                  {step < currentStep ? '✓' : step}
                </button>
                {step < TOTAL_STEPS && (
                  <div
                    className={`flex-1 h-1.5 mx-3 rounded-full transition-all duration-300 ${
                      step < currentStep ? 'bg-green-400' : 'bg-blue-300 bg-opacity-50'
                    }`}
                  />
                )}
              </div>
            ))}
          </div>
          <div className="text-center">
            <div className="text-white text-sm font-medium mb-1">
              {getStepTitle(currentStep)}
            </div>
            <div className="text-blue-100 text-xs">
              ステップ {currentStep} / {TOTAL_STEPS}
            </div>
          </div>
        </div>

        {/* ステップコンテンツ */}
        <div className="px-8 py-8 min-h-[500px] bg-white">
          <div className="animate-fade-in">{renderStep()}</div>
        </div>

        {/* ナビゲーションボタン */}
        <div className="px-8 py-6 bg-gray-50 border-t border-gray-200 flex items-center justify-between">
          <button
            onClick={prevStep}
            disabled={currentStep === 1}
            className="px-6 py-3 text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-all font-medium shadow-sm hover:shadow"
          >
            <span className="flex items-center gap-2">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M15 19l-7-7 7-7"
                />
              </svg>
              戻る
            </span>
          </button>

          {currentStep < TOTAL_STEPS ? (
            <button
              onClick={nextStep}
              disabled={!canGoNext()}
              className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-all font-medium shadow-lg hover:shadow-xl disabled:shadow-none"
            >
              <span className="flex items-center gap-2">
                次へ
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M9 5l7 7-7 7"
                  />
                </svg>
              </span>
            </button>
          ) : (
            <button
              onClick={() => void handleComplete()}
              className="px-8 py-3 bg-gradient-to-r from-green-500 to-green-600 text-white rounded-lg hover:from-green-600 hover:to-green-700 transition-all font-bold shadow-lg hover:shadow-xl"
            >
              <span className="flex items-center gap-2">
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fillRule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                    clipRule="evenodd"
                  />
                </svg>
                完了してメイン画面へ
              </span>
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

// ========================================
// ヘルパー関数
// ========================================

function getStepTitle(step: number): string {
  switch (step) {
    case 1:
      return 'ようこそ';
    case 2:
      return 'OBS WebSocket設定';
    case 3:
      return '接続テスト';
    case 4:
      return 'セットアップ完了';
    default:
      return '';
  }
}
