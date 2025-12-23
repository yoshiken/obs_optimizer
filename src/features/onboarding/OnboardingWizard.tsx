import { useEffect } from 'react';
import { REQUIRED_STEPS, TOTAL_STEPS, useOnboardingStore } from '../../stores/onboardingStore';
import { useConfigStore } from '../../stores/configStore';
import { useObsStore } from '../../stores/obsStore';
import { Welcome } from './steps/Welcome';
import { ObsConnection } from './steps/ObsConnection';
import { StreamStyleStep } from './steps/StreamStyle';
import { PlatformStep } from './steps/Platform';
import { Analysis } from './steps/Analysis';
import { AutoOptimize } from './steps/AutoOptimize';
import { Complete } from './steps/Complete';

/**
 * オンボーディングウィザード
 *
 * 7ステップのセットアップフロー
 */
export function OnboardingWizard() {
  const { currentStep, userPreferences, nextStep, prevStep, setStep, completeOnboarding } =
    useOnboardingStore();
  const { updateConfig } = useConfigStore();
  const { connectionState } = useObsStore();

  // OBS接続状態によって自動で次のステップに進む（Step 2のみ）
  useEffect(() => {
    if (currentStep === 2 && connectionState === 'connected') {
      // 少し遅延させてユーザーが接続成功メッセージを見れるようにする
      const timer = setTimeout(() => {
        // ユーザーがまだStep 2にいる場合のみ進む
        if (currentStep === 2) {
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
        return <ObsConnection />;
      case 3:
        return <StreamStyleStep />;
      case 4:
        return <PlatformStep />;
      case 5:
        return <Analysis />;
      case 6:
        return <AutoOptimize />;
      case 7:
        return <Complete />;
      default:
        return <Welcome />;
    }
  };

  // 次へボタンの有効状態
  const canGoNext = () => {
    switch (currentStep) {
      case 2:
        // OBS接続は必須
        return connectionState === 'connected';
      case 3:
        // 配信スタイルの選択は任意
        return true;
      case 4:
        // プラットフォームの選択は任意
        return true;
      case 5:
        // 分析は自動実行されるので常に進める
        return true;
      default:
        return true;
    }
  };

  // 完了処理
  const handleComplete = async () => {
    try {
      // オンボーディング完了フラグとユーザー設定を保存
      await updateConfig({
        onboardingCompleted: true,
        streamStyle: userPreferences.streamStyle,
        platform: userPreferences.platform,
      });
      completeOnboarding();
    } catch (e) {
      // エラーはストアで処理される
      console.error('設定の保存に失敗しました:', e);
    }
  };

  return (
    <div className="min-h-screen bg-gray-100 flex items-center justify-center p-4">
      <div className="max-w-4xl w-full bg-white rounded-lg shadow-xl">
        {/* プログレスバー */}
        <div className="px-8 pt-8 pb-4">
          <div className="flex items-center justify-between mb-2">
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
                    w-10 h-10 rounded-full flex items-center justify-center font-semibold text-sm
                    transition-colors
                    ${
                      step === currentStep
                        ? 'bg-blue-500 text-white'
                        : step < currentStep
                        ? 'bg-green-500 text-white cursor-pointer hover:bg-green-600'
                        : 'bg-gray-200 text-gray-500 cursor-not-allowed'
                    }
                  `}
                  aria-label={`ステップ ${step}`}
                  aria-current={step === currentStep ? 'step' : undefined}
                >
                  {step < currentStep ? '✓' : step}
                </button>
                {step < TOTAL_STEPS && (
                  <div
                    className={`flex-1 h-1 mx-2 ${
                      step < currentStep ? 'bg-green-500' : 'bg-gray-200'
                    }`}
                  />
                )}
              </div>
            ))}
          </div>
          <div className="text-center text-sm text-gray-600">
            ステップ {currentStep} / {TOTAL_STEPS}
          </div>
        </div>

        {/* ステップコンテンツ */}
        <div className="px-8 py-6 min-h-[400px]">{renderStep()}</div>

        {/* ナビゲーションボタン */}
        <div className="px-8 pb-8 flex items-center justify-between">
          <button
            onClick={prevStep}
            disabled={currentStep === 1}
            className="px-6 py-2 text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            戻る
          </button>

          {currentStep < TOTAL_STEPS ? (
            <button
              onClick={nextStep}
              disabled={!canGoNext()}
              className="px-6 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              次へ
            </button>
          ) : (
            <button
              onClick={() => void handleComplete()}
              className="px-6 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600 transition-colors"
            >
              完了
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
