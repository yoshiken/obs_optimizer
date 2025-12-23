import { useEffect, useState } from 'react';
import { ObsConnectionPanel } from './features/obs/ObsConnectionPanel';
import { ObsStatusIndicator } from './features/obs/ObsStatusIndicator';
import { ObsStreamControls } from './features/obs/ObsStreamControls';
import { ObsSceneSelector } from './features/obs/ObsSceneSelector';
import { MetricsPanel } from './features/monitor';
import { ThemeToggle } from './components/ThemeToggle';
import { OnboardingWizard } from './features/onboarding/OnboardingWizard';
import { useObsStore } from './stores/obsStore';
import { useConfigStore } from './stores/configStore';
import { useOnboardingStore } from './stores/onboardingStore';
import './App.css';

/**
 * OBS配信最適化ツール - メインアプリケーション
 *
 * 初回起動時:
 * - localStorageでオンボーディング完了をチェック
 * - 未完了の場合はOnboardingWizardを表示
 *
 * メイン画面のレイアウト構成:
 * - ヘッダー: アプリケーション名
 * - 上部: OBS接続設定パネル
 * - 中央左: OBSステータス表示 + 配信・録画コントロール
 * - 中央右: システムメトリクス表示
 * - 下部: シーン選択パネル
 */
function App() {
  const { startPolling, subscribeToEvents } = useObsStore();
  const { config, loadConfig } = useConfigStore();
  const { completed: onboardingCompleted } = useOnboardingStore();
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  // 初回起動時: 設定を読み込んでオンボーディング状態をチェック
  useEffect(() => {
    const checkOnboarding = async () => {
      try {
        await loadConfig();
        setIsLoading(false);
      } catch (error) {
        console.error('設定の読み込みに失敗しました:', error);
        setIsLoading(false);
      }
    };

    void checkOnboarding();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // 設定が読み込まれたらオンボーディング状態を判定
  useEffect(() => {
    if (!isLoading && config) {
      // localStorageとストアの両方をチェック
      const localStorageCompleted = localStorage.getItem('onboardingCompleted') === 'true';
      const shouldShowOnboarding =
        !config.onboardingCompleted && !onboardingCompleted && !localStorageCompleted;
      setShowOnboarding(shouldShowOnboarding);
    }
  }, [isLoading, config, onboardingCompleted]);

  // オンボーディング完了時: localStorageに保存してメイン画面を表示
  useEffect(() => {
    if (onboardingCompleted) {
      localStorage.setItem('onboardingCompleted', 'true');
      setShowOnboarding(false);
    }
  }, [onboardingCompleted]);

  // メイン画面表示時のみOBSポーリングとイベント購読を開始
  useEffect(() => {
    if (!showOnboarding && !isLoading) {
      // OBSステータスのポーリング開始（1秒間隔）
      const stopPolling = startPolling(1000);

      // OBSイベントの購読開始
      let unsubscribe: (() => void) | undefined;
      let isMounted = true;

      // 非同期処理のレースコンディション対策
      const setupSubscription = async () => {
        try {
          const unsub = await subscribeToEvents();
          if (isMounted) {
            unsubscribe = unsub;
          } else {
            // コンポーネントが既にアンマウントされている場合は即座にクリーンアップ
            unsub();
          }
        } catch {
          // 購読エラーは無視（ストアでエラー処理される）
        }
      };

      void setupSubscription();

      // クリーンアップ: ポーリングとイベント購読を停止
      return () => {
        isMounted = false;
        stopPolling();
        if (unsubscribe) {
          unsubscribe();
        }
      };
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [showOnboarding, isLoading]);

  // ローディング画面
  if (isLoading) {
    return (
      <div className="min-h-screen bg-gray-100 dark:bg-gray-900 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-blue-600 mx-auto mb-4" />
          <p className="text-gray-600 dark:text-gray-400">読み込み中...</p>
        </div>
      </div>
    );
  }

  // オンボーディング画面
  if (showOnboarding) {
    return <OnboardingWizard />;
  }

  // メイン画面
  return (
    <main className="min-h-screen bg-gray-100 dark:bg-gray-900">
      {/* ヘッダー */}
      <header className="bg-white dark:bg-gray-800 shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <div className="flex items-center justify-between">
            <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
              OBS配信最適化ツール
            </h1>
            <ThemeToggle />
          </div>
        </div>
      </header>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-6">
        {/* 上部: OBS接続設定パネル */}
        <section>
          <ObsConnectionPanel />
        </section>

        {/* 中央: OBSコントロール と システムメトリクス */}
        <section className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* 左カラム: OBSステータスとコントロール */}
          <div className="space-y-6">
            <ObsStatusIndicator />
            <ObsStreamControls />
          </div>

          {/* 右カラム: システムメトリクス */}
          <div>
            <MetricsPanel />
          </div>
        </section>

        {/* 下部: シーン選択 */}
        <section>
          <ObsSceneSelector />
        </section>
      </div>
    </main>
  );
}

export default App;
