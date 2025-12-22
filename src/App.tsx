import { useEffect } from 'react';
import { ObsConnectionPanel } from './features/obs/ObsConnectionPanel';
import { ObsStatusIndicator } from './features/obs/ObsStatusIndicator';
import { ObsStreamControls } from './features/obs/ObsStreamControls';
import { ObsSceneSelector } from './features/obs/ObsSceneSelector';
import { MetricsPanel } from './features/monitor';
import { useObsStore } from './stores/obsStore';
import './App.css';

/**
 * OBS配信最適化ツール - メインアプリケーション
 *
 * レイアウト構成:
 * - ヘッダー: アプリケーション名
 * - 上部: OBS接続設定パネル
 * - 中央左: OBSステータス表示 + 配信・録画コントロール
 * - 中央右: システムメトリクス表示
 * - 下部: シーン選択パネル
 */
function App() {
  const { startPolling, subscribeToEvents } = useObsStore();

  useEffect(() => {
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

    setupSubscription();

    // クリーンアップ: ポーリングとイベント購読を停止
    return () => {
      isMounted = false;
      stopPolling();
      if (unsubscribe) {
        unsubscribe();
      }
    };
    // 依存配列を空にして、マウント時のみ実行
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <main className="min-h-screen bg-gray-100">
      {/* ヘッダー */}
      <header className="bg-white shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <h1 className="text-2xl font-bold text-gray-900">OBS配信最適化ツール</h1>
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
