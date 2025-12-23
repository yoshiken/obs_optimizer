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
import { useAlertStore } from './stores/alertStore';
import { useAnalysisStore } from './stores/analysisStore';
import './App.css';

/**
 * OBS配信最適化ツール - メインアプリケーション
 *
 * 初回起動時:
 * - localStorageでオンボーディング完了をチェック
 * - 未完了の場合はOnboardingWizardを表示
 *
 * メイン画面のレイアウト構成:
 * - ヘッダー: アプリケーション名 + テーマ切り替え
 * - タブナビゲーション: ダッシュボード、問題分析、最適化、履歴、エクスポート
 * - タブコンテンツ: 各タブに対応する機能パネル
 */

type TabId = 'dashboard' | 'analysis' | 'optimization' | 'history' | 'export';

interface Tab {
  id: TabId;
  label: string;
  icon: string;
  badge?: number;
}

function App() {
  const { startPolling, subscribeToEvents } = useObsStore();
  const { config, loadConfig } = useConfigStore();
  const { completed: onboardingCompleted } = useOnboardingStore();
  const { getActiveAlerts } = useAlertStore();
  const { problems } = useAnalysisStore();
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<TabId>('dashboard');

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

  // タブ定義（バッジは動的に設定）
  const tabs: Tab[] = [
    {
      id: 'dashboard',
      label: 'ダッシュボード',
      icon: '📊',
    },
    {
      id: 'analysis',
      label: '問題分析',
      icon: '🔍',
      badge: getActiveAlerts().length + problems.length,
    },
    {
      id: 'optimization',
      label: '最適化',
      icon: '⚙️',
    },
    {
      id: 'history',
      label: '履歴',
      icon: '📈',
    },
    {
      id: 'export',
      label: 'エクスポート',
      icon: '📤',
    },
  ];

  // メイン画面
  return (
    <main className="min-h-screen bg-gray-100 dark:bg-gray-900">
      {/* ヘッダー */}
      <header className="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <div className="flex items-center justify-between">
            <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
              OBS配信最適化ツール
            </h1>
            <ThemeToggle />
          </div>
        </div>

        {/* タブナビゲーション */}
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <nav className="flex space-x-1" aria-label="Tabs">
            {tabs.map((tab) => {
              const isActive = activeTab === tab.id;
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`
                    relative px-4 py-3 text-sm font-medium rounded-t-lg transition-colors
                    ${
                      isActive
                        ? 'bg-gray-100 dark:bg-gray-900 text-blue-600 dark:text-blue-400 border-t-2 border-blue-600 dark:border-blue-400'
                        : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-800'
                    }
                  `}
                  aria-current={isActive ? 'page' : undefined}
                >
                  <span className="flex items-center gap-2">
                    <span>{tab.icon}</span>
                    <span>{tab.label}</span>
                    {tab.badge !== undefined && tab.badge > 0 && (
                      <span className="ml-1 px-2 py-0.5 text-xs font-semibold bg-red-600 text-white rounded-full">
                        {tab.badge}
                      </span>
                    )}
                  </span>
                </button>
              );
            })}
          </nav>
        </div>
      </header>

      {/* タブコンテンツ */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
        {activeTab === 'dashboard' && <DashboardTab />}
        {activeTab === 'analysis' && <AnalysisTab />}
        {activeTab === 'optimization' && <OptimizationTab />}
        {activeTab === 'history' && <HistoryTab />}
        {activeTab === 'export' && <ExportTab />}
      </div>
    </main>
  );
}

// ========================================
// タブコンテンツコンポーネント
// ========================================

/**
 * ダッシュボードタブ - OBS接続、ステータス、メトリクス監視
 */
function DashboardTab() {
  return (
    <div className="space-y-6">
      {/* OBS接続設定パネル */}
      <section>
        <ObsConnectionPanel />
      </section>

      {/* OBSコントロール と システムメトリクス */}
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

      {/* シーン選択 */}
      <section>
        <ObsSceneSelector />
      </section>
    </div>
  );
}

/**
 * 問題分析タブ - パフォーマンス問題の検出と診断レポート
 */
function AnalysisTab() {
  return (
    <div className="space-y-6">
      {/* プレースホルダー: ProblemDashboard */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
          問題分析ダッシュボード
        </h2>
        <div className="text-gray-600 dark:text-gray-400">
          <p className="mb-4">パフォーマンス問題を検出し、解決策を提案します。</p>
          <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded p-4">
            <p className="text-sm text-blue-800 dark:text-blue-300">
              実装予定: ProblemDashboard コンポーネント
            </p>
            <p className="text-sm text-blue-700 dark:text-blue-400 mt-2">
              機能: CPU/GPU/ネットワークの問題検出、重要度別の問題表示、推奨アクション
            </p>
          </div>
        </div>
      </div>

      {/* プレースホルダー: DiagnosticReport */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
          診断レポート
        </h2>
        <div className="text-gray-600 dark:text-gray-400">
          <p className="mb-4">詳細な診断結果とパフォーマンス評価を表示します。</p>
          <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded p-4">
            <p className="text-sm text-blue-800 dark:text-blue-300">
              実装予定: DiagnosticReport コンポーネント
            </p>
            <p className="text-sm text-blue-700 dark:text-blue-400 mt-2">
              機能: システム情報、パフォーマンススコア、問題の詳細分析
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

/**
 * 最適化タブ - ワンクリック最適化とプロファイル管理
 */
function OptimizationTab() {
  return (
    <div className="space-y-6">
      {/* プレースホルダー: OneClickApply */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
          ワンクリック最適化
        </h2>
        <div className="text-gray-600 dark:text-gray-400">
          <p className="mb-4">システムに最適な設定を自動で適用します。</p>
          <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded p-4">
            <p className="text-sm text-blue-800 dark:text-blue-300">
              実装予定: OneClickApply コンポーネント
            </p>
            <p className="text-sm text-blue-700 dark:text-blue-400 mt-2">
              機能: プリセット選択（低/中/高/最高）、推奨設定のプレビュー、適用ボタン
            </p>
          </div>
        </div>
      </div>

      {/* プレースホルダー: ProfileList / ProfileEditor */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
          プロファイル管理
        </h2>
        <div className="text-gray-600 dark:text-gray-400">
          <p className="mb-4">配信スタイル別の設定プロファイルを保存・管理します。</p>
          <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded p-4">
            <p className="text-sm text-blue-800 dark:text-blue-300">
              実装予定: ProfileList / ProfileEditor コンポーネント
            </p>
            <p className="text-sm text-blue-700 dark:text-blue-400 mt-2">
              機能: プロファイル一覧、新規作成、編集、削除、適用
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

/**
 * 履歴タブ - セッション履歴とメトリクスの時系列表示
 */
function HistoryTab() {
  return (
    <div className="space-y-6">
      {/* プレースホルダー: SessionHistory */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
          セッション履歴
        </h2>
        <div className="text-gray-600 dark:text-gray-400">
          <p className="mb-4">過去の配信セッションのパフォーマンスを確認できます。</p>
          <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded p-4">
            <p className="text-sm text-blue-800 dark:text-blue-300">
              実装予定: SessionHistory コンポーネント
            </p>
            <p className="text-sm text-blue-700 dark:text-blue-400 mt-2">
              機能: セッション一覧、平均CPU/GPU使用率、フレームドロップ数、品質スコア
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

/**
 * エクスポートタブ - データのエクスポート機能
 */
function ExportTab() {
  return (
    <div className="space-y-6">
      {/* プレースホルダー: ExportPanel */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
          データエクスポート
        </h2>
        <div className="text-gray-600 dark:text-gray-400">
          <p className="mb-4">セッションデータや診断レポートをエクスポートします。</p>
          <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded p-4">
            <p className="text-sm text-blue-800 dark:text-blue-300">
              実装予定: ExportPanel コンポーネント
            </p>
            <p className="text-sm text-blue-700 dark:text-blue-400 mt-2">
              機能: JSON/CSV形式エクスポート、診断レポート生成、セッション選択
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
