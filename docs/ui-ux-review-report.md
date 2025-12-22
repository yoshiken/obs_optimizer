# OBS配信最適化ツール UI/UXレビューレポート

**レビュー日時:** 2025-12-22
**レビュー対象:** /home/yskn/git/obs_optimizer/src/
**評価基準:** ユーザビリティ、アクセシビリティ、視覚デザイン、フィードバック、レスポンシブ、一貫性

---

## 1. エグゼクティブサマリー

### 1.1 総合評価

| 観点 | スコア | コメント |
|------|-------|---------|
| ユーザビリティ | 7/10 | 基本操作は直感的だが、初心者向けガイドが不足 |
| アクセシビリティ | 5/10 | 改善余地が大きい（ARIA、キーボード操作） |
| 視覚デザイン | 8/10 | Tailwindを活用し統一感あり、色彩は要改善 |
| フィードバック | 6/10 | ローディング状態は良好、成功通知が不足 |
| レスポンシブ | 7/10 | 基本的な対応あり、タブレット・モバイルは未検証 |
| 一貫性 | 8/10 | コンポーネント間でスタイルが統一されている |

### 1.2 クリティカルな問題（即時対応推奨）

1. **アクセシビリティ欠如**: スクリーンリーダー対応が不十分（WCAG AA準拠未達）
2. **初心者UX**: 専門用語が多く、ペルソナ「あかり」のニーズを満たしていない
3. **エラー時の対処法不明**: エラーメッセージに具体的な解決策がない
4. **配信中モード未実装**: 配信中のゲーム優先動作が実装されていない

---

## 2. コンポーネント別評価

### 2.1 App.tsx（メインレイアウト）

**ファイル:** `/home/yskn/git/obs_optimizer/src/App.tsx`

#### Good
- レイアウト構造が明確で、セクション分けが適切
- レスポンシブグリッド（`lg:grid-cols-2`）で画面サイズに対応
- ヘッダーが固定で、アプリケーション名が常に視認可能

#### Needs Improvement

| 優先度 | 問題 | 現状 | 改善案 |
|-------|------|------|-------|
| **High** | ナビゲーション不在 | ヘッダーにタイトルのみ | タブナビゲーション追加（ダッシュボード、診断、設定） |
| **High** | オンボーディング不在 | 初回起動時にガイドなし | チュートリアルモーダル or ツールチップツアー |
| **Medium** | グランスビュー未実装 | requirements_v2.mdで定義された健康状態サマリーがない | ヘッダー直下に「配信状態: 良好」バッジ追加 |
| **Low** | ページタイトル不足 | `<main>`にラベルなし | `aria-label="メインコンテンツ"`追加 |

**推奨UI改善:**
```tsx
{/* ヘッダーにナビゲーション追加 */}
<header className="bg-white shadow-sm">
  <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
    <div className="flex items-center justify-between py-4">
      <h1 className="text-2xl font-bold text-gray-900">OBS配信最適化ツール</h1>
      <nav role="navigation" aria-label="メインナビゲーション">
        <ul className="flex gap-4">
          <li><button className="px-4 py-2 rounded-md hover:bg-gray-100">ダッシュボード</button></li>
          <li><button className="px-4 py-2 rounded-md hover:bg-gray-100">設定</button></li>
        </ul>
      </nav>
    </div>
  </div>
</header>

{/* グランスビュー追加 */}
<section className="bg-gradient-to-r from-green-50 to-blue-50 py-4">
  <div className="max-w-7xl mx-auto px-4 flex items-center gap-3">
    <span className="w-4 h-4 bg-green-500 rounded-full" aria-hidden="true" />
    <span className="text-lg font-semibold">配信状態: 良好</span>
    <span className="text-sm text-gray-600">問題は検出されていません</span>
  </div>
</section>
```

---

### 2.2 ObsConnectionPanel.tsx（OBS接続設定）

**ファイル:** `/home/yskn/git/obs_optimizer/src/features/obs/ObsConnectionPanel.tsx`

#### Good
- 入力バリデーションが優れている（ポート範囲チェック、リアルタイムエラー表示）
- パスワード表示/非表示トグルでセキュリティと使いやすさを両立
- 接続状態が色付きバッジで視覚的にわかりやすい
- `disabled`状態が適切に管理されている

#### Needs Improvement

| 優先度 | 問題 | 現状 | 改善案 |
|-------|------|------|-------|
| **High** | 初心者向け説明不足 | ホスト・ポート・パスワードの意味が不明 | 各入力欄の下に「ヒント」を追加 |
| **Medium** | エラーメッセージが技術的 | エラーがそのまま表示される可能性 | エラーメッセージを翻訳（例: "Connection refused" → "OBSに接続できませんでした。OBSが起動しているか確認してください"） |
| **Medium** | キーボード操作不完全 | Enterキーで接続できない | フォーム内Enterキーで`handleConnect`実行 |
| **Low** | アクセシビリティ | エラーメッセージに`role="alert"`がない | エラー表示時に`role="alert" aria-live="assertive"`追加 |

**推奨UI改善:**
```tsx
{/* ホスト入力にヒント追加 */}
<div>
  <label htmlFor="obs-host" className="block text-sm font-medium text-gray-700 mb-1">
    ホスト
  </label>
  <input ... />
  <p className="mt-1 text-xs text-gray-500">
    💡 OBSが同じパソコンで動いている場合は「localhost」のままでOKです
  </p>
</div>

{/* エラー表示にARIA追加 */}
{error && (
  <div
    role="alert"
    aria-live="assertive"
    className="mb-4 p-3 bg-red-100 border border-red-300 rounded-md"
  >
    <div className="flex items-center justify-between">
      <div>
        <span className="text-sm text-red-700 font-medium">{translateError(error)}</span>
        <p className="text-xs text-red-600 mt-1">
          💡 OBSが起動しているか確認してください
        </p>
      </div>
      <button ... >x</button>
    </div>
  </div>
)}
```

---

### 2.3 ObsStatusIndicator.tsx（OBSステータス表示）

**ファイル:** `/home/yskn/git/obs_optimizer/src/features/obs/ObsStatusIndicator.tsx`

#### Good
- 配信/録画中の`animate-pulse`で視覚的にライブ感を表現
- 時間コードとビットレートの数値表示が正確
- パフォーマンス統計（FPS、ドロップフレーム）が目立つ配置
- 仮想カメラ有効時のインジケーターが親切

#### Needs Improvement

| 優先度 | 問題 | 現状 | 改善案 |
|-------|------|------|-------|
| **High** | 初心者に専門用語が難しい | 「FPS」「レンダードロップ」「出力ドロップ」がそのまま | ツールチップで説明追加（例: FPS → "1秒あたりのコマ数。通常30〜60が目安"） |
| **High** | ドロップフレーム発生時の対処法不明 | 黄色・赤色で表示するだけ | 「問題を解決する」リンク or ボタン追加 |
| **Medium** | ARIA不足 | `aria-live`でステータス変化を通知していない | メトリクス部分に`role="status" aria-live="polite"`追加 |
| **Low** | OBSバージョン表示が小さい | 文字サイズ`text-xs`で視認性低い | ホバーで詳細表示 or 「詳細」ボタン追加 |

**推奨UI改善:**
```tsx
{/* ツールチップコンポーネント例 */}
<div className="text-center relative group">
  <div className="text-2xl font-bold text-gray-800">
    {status?.fps?.toFixed(1) ?? '--'}
  </div>
  <div className="text-xs text-gray-500">FPS</div>
  {/* ツールチップ */}
  <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-2 bg-gray-900 text-white text-xs rounded-md opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap z-10">
    1秒あたりのコマ数。通常30〜60が目安
  </div>
</div>

{/* ドロップフレーム警告 */}
{(status?.outputDroppedFrames ?? 0) > 0 && (
  <div className="mt-4 p-3 bg-yellow-50 border border-yellow-300 rounded-md">
    <p className="text-sm text-yellow-800">
      映像が少しカクついています
    </p>
    <button className="mt-2 text-sm text-blue-600 hover:underline">
      → 対処法を見る
    </button>
  </div>
)}
```

---

### 2.4 ObsStreamControls.tsx（配信・録画コントロール）

**ファイル:** `/home/yskn/git/obs_optimizer/src/features/obs/ObsStreamControls.tsx`

#### Good
- 配信/録画ボタンが視覚的に大きく、操作しやすい
- ローディング状態（スピナー）が明確
- 録画完了時の通知（出力パス表示）が親切
- アイコン+テキストで色覚多様性に配慮

#### Needs Improvement

| 優先度 | 問題 | 現状 | 改善案 |
|-------|------|------|-------|
| **Critical** | 誤操作防止がない | 配信中にクリック1回で停止可能 | 確認モーダル追加（「本当に配信を停止しますか?」） |
| **High** | 成功通知がない | 録画開始時にフィードバックなし | トースト通知で「録画を開始しました」表示 |
| **Medium** | キーボードショートカット未実装 | requirements_v2.mdで定義された`Ctrl+R`等が未実装 | グローバルショートカット追加 |
| **Medium** | 時間コードが単調 | ただ表示されるだけ | 1時間超過時に色を変える等、視覚的なフィードバック |
| **Low** | 録画パス通知が消える | 5秒で自動消去 | 「クリップボードにコピー」ボタン追加 |

**推奨UI改善:**
```tsx
{/* 確認モーダル */}
const handleStreamingToggle = useCallback(async () => {
  if (isStreaming) {
    // 停止時は確認ダイアログ
    const confirmed = await showConfirmDialog({
      title: '配信を停止しますか？',
      message: '視聴者への配信が終了します。',
      confirmText: '停止する',
      cancelText: 'キャンセル',
      severity: 'warning'
    });
    if (!confirmed) return;
  }

  try {
    if (isStreaming) {
      await stopStreaming();
      showToast('配信を停止しました', 'success');
    } else {
      await startStreaming();
      showToast('配信を開始しました', 'success');
    }
  } catch { ... }
}, [isStreaming, startStreaming, stopStreaming]);

{/* 録画パス通知改善 */}
{showRecordingPathNotice && lastRecordingPath && (
  <div className="mb-4 p-3 bg-green-100 border border-green-300 rounded-md">
    <div className="flex items-center justify-between">
      <div className="flex-1">
        <span className="text-sm text-green-700 font-medium">録画が保存されました</span>
        <p className="text-xs text-green-600 mt-1 font-mono truncate" title={lastRecordingPath}>
          {lastRecordingPath}
        </p>
      </div>
      <div className="flex gap-2">
        <button
          onClick={() => {
            navigator.clipboard.writeText(lastRecordingPath);
            showToast('パスをコピーしました', 'success');
          }}
          className="text-sm text-green-600 hover:underline"
        >
          コピー
        </button>
        <button
          onClick={() => setShowRecordingPathNotice(false)}
          className="text-green-500 hover:text-green-700 text-sm"
          aria-label="通知を閉じる"
        >
          x
        </button>
      </div>
    </div>
  </div>
)}
```

---

### 2.5 ObsSceneSelector.tsx（シーン選択）

**ファイル:** `/home/yskn/git/obs_optimizer/src/features/obs/ObsSceneSelector.tsx`

#### Good
- モバイル用ドロップダウンとデスクトップ用グリッド表示でレスポンシブ対応
- アクティブシーンの視覚的強調が明確
- `disabled`状態で誤操作を防止

#### Needs Improvement

| 優先度 | 問題 | 現状 | 改善案 |
|-------|------|------|-------|
| **Medium** | シーン変更の成功通知なし | シーンが変わったかわかりづらい | トースト通知で「シーンを切り替えました: {シーン名}」 |
| **Medium** | シーン変更が即座 | 配信中の誤操作リスク | 配信中モード時は確認ダイアログ追加 |
| **Low** | シーンアイコンが単調 | 全シーンに同じアイコン | シーン名に応じたアイコン（ゲーム、雑談等） |
| **Low** | キーボード操作 | 矢印キーでシーン選択できない | 矢印キー + Enterで選択可能に |

**推奨UI改善:**
```tsx
const handleSceneChange = useCallback(
  async (sceneName: string) => {
    if (sceneName === currentScene) return;

    // 配信中は確認ダイアログ
    if (status?.streaming) {
      const confirmed = await showConfirmDialog({
        title: 'シーンを切り替えますか？',
        message: '配信中に映像が切り替わります。',
        confirmText: '切り替える',
        cancelText: 'キャンセル',
      });
      if (!confirmed) return;
    }

    try {
      await setCurrentScene(sceneName);
      showToast(`シーンを切り替えました: ${sceneName}`, 'success');
    } catch (e) {
      showToast('シーンの切り替えに失敗しました', 'error');
      console.error('Scene change failed:', e);
    }
  },
  [currentScene, setCurrentScene, status?.streaming]
);
```

---

### 2.6 MetricsPanel.tsx（システムメトリクス）

**ファイル:** `/home/yskn/git/obs_optimizer/src/features/monitor/MetricsPanel.tsx`

#### Good
- 数値フォーマット（`formatBytes`, `formatSpeed`）が統一されている
- CPU・メモリ・GPU・ネットワークの基本情報がコンパクトに表示

#### Needs Improvement

| 優先度 | 問題 | 現状 | 改善案 |
|-------|------|------|-------|
| **High** | ユーザー向け用語に未翻訳 | 「CPU」「メモリ」「GPU」のまま | requirements_v2.mdの用語翻訳表に基づき、初心者モード時は「パソコンの処理」「作業用記憶」「画像処理」に変更 |
| **High** | 深刻度の色分けなし | 90%でも緑色表示 | requirements_v2.mdのセベリティ閾値に基づき、黄色・赤色で警告 |
| **Medium** | トレンド情報なし | 現在値のみ | 小さなグラフ or 矢印で増減トレンド表示 |
| **Low** | ARIA不足 | 動的な値変化を通知していない | `role="status" aria-live="polite"`追加 |

**推奨UI改善:**
```tsx
{/* CPU使用率にセベリティ適用 */}
<div className="bg-gray-50 rounded-md p-4" role="status" aria-live="polite">
  <div className="flex items-center justify-between">
    <span className="text-sm font-medium text-gray-700">
      {easyMode ? 'パソコンの処理' : 'CPU'}
    </span>
    <span className="text-xs text-gray-500">({metrics.cpu.coreCount}コア)</span>
  </div>
  <div className="mt-2 flex items-baseline gap-2">
    <span
      className={`text-2xl font-bold ${
        metrics.cpu.usagePercent > 90 ? 'text-red-600' :
        metrics.cpu.usagePercent > 70 ? 'text-yellow-600' :
        'text-gray-800'
      }`}
    >
      {metrics.cpu.usagePercent.toFixed(1)}%
    </span>
    {/* トレンド矢印 */}
    {trend > 0 && <span className="text-red-500 text-lg">↑</span>}
    {trend < 0 && <span className="text-green-500 text-lg">↓</span>}
  </div>
  {/* プログレスバー */}
  <div className="mt-2 h-2 bg-gray-200 rounded-full overflow-hidden">
    <div
      className={`h-full transition-all duration-300 ${
        metrics.cpu.usagePercent > 90 ? 'bg-red-500' :
        metrics.cpu.usagePercent > 70 ? 'bg-yellow-500' :
        'bg-green-500'
      }`}
      style={{ width: `${metrics.cpu.usagePercent}%` }}
    />
  </div>
</div>
```

---

### 2.7 DetailedMetricsPanel.tsx（詳細メトリクス）

**ファイル:** `/home/yskn/git/obs_optimizer/src/features/monitor/DetailedMetricsPanel.tsx`

#### Good
- インラインスタイルでCSS変数を活用し、ダークモード対応の準備ができている
- OBSプロセス情報の表示が有益
- `refreshInterval`と`compactMode`のカスタマイズ性が高い

#### Needs Improvement

| 優先度 | 問題 | 現状 | 改善案 |
|-------|------|------|-------|
| **Medium** | CSS変数が未定義 | `var(--text-primary, #0f0f0f)`等が実際には定義されていない可能性 | グローバルCSSに変数を定義 or Tailwindに統合 |
| **Medium** | エラー表示が簡素 | 赤背景にテキストのみ | エラーの種類に応じて再試行ボタン or 設定へのリンク追加 |
| **Low** | 最終更新時刻の曖昧さ | 「2秒前」だけでは不正確 | ホバーで絶対時刻（ISO8601）表示 |

---

### 2.8 メトリクスカードコンポーネント

**ファイル:** `/home/yskn/git/obs_optimizer/src/features/monitor/components/`

#### CpuMetricsCard.tsx
**Good:**
- コア別使用率の視覚化が詳細
- セベリティに基づく色分け実装済み

**Needs Improvement:**
- コア数が多い場合（16コア等）に表示が縦長になりすぎる → 折りたたみ or スクロール

#### MemoryMetricsCard.tsx
**Good:**
- プログレスバーでメモリ使用率が直感的

**Needs Improvement:**
- スワップメモリ情報がない → Windows環境でページファイル使用量も表示

#### GpuMetricsCard.tsx
**Good:**
- GPU未検出時のフィードバックメッセージが親切

**Needs Improvement:**
- エンコーダー使用率が0%の場合も表示 → 「使用していません」と明示

#### NetworkMetricsCard.tsx
**Good:**
- アップロード・ダウンロードのアイコンが視覚的

**Needs Improvement:**
- 配信ビットレートと比較していない → 「配信に十分な速度です」等の評価追加

---

## 3. 横断的な問題点

### 3.1 アクセシビリティ（WCAG 2.1 AA準拠未達）

| 問題 | 現状 | 改善優先度 |
|------|------|-----------|
| **ARIA属性不足** | メトリクス更新時に`aria-live`がない | Critical |
| **キーボード操作** | Tabキーでフォーカス可能だが、Enterキー動作が不完全 | High |
| **スクリーンリーダー** | グラフやバーに代替テキストがない | High |
| **フォーカスインジケーター** | デフォルトの細いアウトラインのみ | Medium |
| **color-only依存** | ドロップフレームの警告が色のみ | Medium |

**推奨対応:**
1. 全インタラクティブ要素に`aria-label`追加
2. 動的コンテンツに`role="status"`と`aria-live="polite"`
3. 警告・エラーに`role="alert"`と`aria-live="assertive"`
4. フォーカススタイルをグローバルCSSで定義
```css
*:focus-visible {
  outline: 3px solid #3b82f6;
  outline-offset: 2px;
}
```

### 3.2 初心者UX（ペルソナ「あかり」対応不足）

| ニーズ | 現状 | 改善案 |
|--------|------|-------|
| ワンクリック最適化 | 未実装 | 「おまかせ設定」ボタン追加 |
| 専門用語回避 | そのまま表示 | 「初心者モード」トグル追加、用語翻訳 |
| 具体的な対処法 | エラーメッセージのみ | 「この問題を解決する」リンク追加 |

**推奨実装:**
```tsx
// 初心者モードトグル
const [easyMode, setEasyMode] = useState(true);

{/* ヘッダーにトグル追加 */}
<div className="flex items-center gap-2">
  <label htmlFor="easy-mode" className="text-sm text-gray-700">
    初心者モード
  </label>
  <input
    id="easy-mode"
    type="checkbox"
    checked={easyMode}
    onChange={(e) => setEasyMode(e.target.checked)}
    className="toggle"
  />
</div>

{/* 表示内容を切り替え */}
<span className="text-sm font-medium text-gray-700">
  {easyMode ? 'パソコンの処理' : 'CPU使用率'}
</span>
```

### 3.3 配信中モード未実装

requirements_v2.mdで定義された「配信中モード」が未実装。

**必要な機能:**
- 通知頻度の抑制（Critical/Warningのみ）
- ウィンドウ常に背面（ゲーム優先）
- UI更新頻度の低減（1秒→2秒）
- 誤操作防止（確認ダイアログ追加）

**推奨実装:**
```tsx
// 配信中モードの自動検出
useEffect(() => {
  if (status?.streaming || status?.recording) {
    setStreamingMode(true);
  } else {
    setStreamingMode(false);
  }
}, [status?.streaming, status?.recording]);

// 配信中モード時のポーリング間隔変更
useEffect(() => {
  const interval = streamingMode ? 2000 : 1000;
  const stopPolling = startPolling(interval);
  return stopPolling;
}, [streamingMode, startPolling]);
```

### 3.4 通知システム未実装

現在はエラー表示のみで、成功通知やヒントが不足。

**推奨実装:**
- トーストライブラリ（react-hot-toast等）を導入
- 通知レベル（Critical, Warning, Info, Tips）に応じた表示
- 配信中モードでは自動抑制

```tsx
import toast from 'react-hot-toast';

// 成功通知
toast.success('配信を開始しました', {
  duration: 3000,
  position: 'bottom-right',
});

// 警告通知
toast.custom((t) => (
  <div className={`${t.visible ? 'animate-enter' : 'animate-leave'} bg-yellow-50 border border-yellow-300 rounded-md p-4 shadow-lg`}>
    <div className="flex items-start gap-3">
      <span className="text-yellow-600">⚠️</span>
      <div className="flex-1">
        <p className="text-sm font-medium text-yellow-800">
          パソコンに負担がかかっています
        </p>
        <button
          onClick={() => toast.dismiss(t.id)}
          className="mt-2 text-sm text-blue-600 hover:underline"
        >
          対処法を見る
        </button>
      </div>
      <button onClick={() => toast.dismiss(t.id)}>x</button>
    </div>
  </div>
), { duration: Infinity });
```

---

## 4. 改善優先度マトリクス

### Phase 1a（MVP完成前 - 即時対応）

| 優先度 | 改善項目 | 対象ファイル | 工数 |
|-------|---------|-------------|------|
| **Critical** | 誤操作防止（配信停止確認） | ObsStreamControls.tsx | 2h |
| **High** | ARIA属性追加（メトリクス） | MetricsPanel.tsx, DetailedMetricsPanel.tsx | 4h |
| **High** | 初心者向けヒント追加 | ObsConnectionPanel.tsx | 2h |
| **High** | セベリティ色分け実装 | MetricsPanel.tsx | 3h |

### Phase 1b（MVPリリース前）

| 優先度 | 改善項目 | 対象ファイル | 工数 |
|-------|---------|-------------|------|
| **High** | ツールチップで用語説明 | ObsStatusIndicator.tsx | 4h |
| **Medium** | 成功通知トースト実装 | 全コンポーネント | 6h |
| **Medium** | 初心者モードトグル | App.tsx, MetricsPanel.tsx | 5h |
| **Medium** | キーボードショートカット | App.tsx | 4h |

### Phase 2a（機能拡張）

| 優先度 | 改善項目 | 対象ファイル | 工数 |
|-------|---------|-------------|------|
| **Medium** | 配信中モード実装 | App.tsx, 全コンポーネント | 10h |
| **Medium** | グランスビュー追加 | App.tsx（新規コンポーネント） | 6h |
| **Low** | ダークモード実装 | 全コンポーネント | 12h |
| **Low** | トレンドグラフ追加 | MetricsPanel.tsx | 8h |

---

## 5. デザインシステム提案

### 5.1 カラーパレット（WCAG AA準拠）

```css
:root {
  /* Status Colors */
  --color-success: #22c55e;
  --color-warning: #f59e0b;
  --color-error: #ef4444;
  --color-info: #3b82f6;
  --color-neutral: #6b7280;

  /* Semantic Colors */
  --color-streaming: #ef4444;
  --color-recording: #ef4444;
  --color-connected: #22c55e;
  --color-disconnected: #6b7280;

  /* Text Colors (Light Mode) */
  --text-primary: #111827;
  --text-secondary: #6b7280;
  --text-muted: #9ca3af;

  /* Background Colors (Light Mode) */
  --bg-primary: #ffffff;
  --bg-secondary: #f9fafb;
  --bg-tertiary: #f3f4f6;

  /* Border Colors */
  --border-primary: #e5e7eb;
  --border-secondary: #d1d5db;
}

/* Dark Mode */
@media (prefers-color-scheme: dark) {
  :root {
    --text-primary: #f9fafb;
    --text-secondary: #9ca3af;
    --text-muted: #6b7280;
    --bg-primary: #1f2937;
    --bg-secondary: #374151;
    --bg-tertiary: #4b5563;
    --border-primary: #4b5563;
    --border-secondary: #6b7280;
  }
}
```

### 5.2 タイポグラフィスケール

```css
:root {
  /* Font Sizes */
  --text-xs: 0.75rem;   /* 12px */
  --text-sm: 0.875rem;  /* 14px */
  --text-base: 1rem;    /* 16px */
  --text-lg: 1.125rem;  /* 18px */
  --text-xl: 1.25rem;   /* 20px */
  --text-2xl: 1.5rem;   /* 24px */
  --text-3xl: 1.875rem; /* 30px */

  /* Font Weights */
  --font-normal: 400;
  --font-medium: 500;
  --font-semibold: 600;
  --font-bold: 700;

  /* Line Heights */
  --leading-tight: 1.25;
  --leading-normal: 1.5;
  --leading-relaxed: 1.75;
}
```

### 5.3 スペーシングシステム

```css
:root {
  /* Spacing Scale (4px base) */
  --space-1: 0.25rem;  /* 4px */
  --space-2: 0.5rem;   /* 8px */
  --space-3: 0.75rem;  /* 12px */
  --space-4: 1rem;     /* 16px */
  --space-6: 1.5rem;   /* 24px */
  --space-8: 2rem;     /* 32px */
  --space-12: 3rem;    /* 48px */

  /* Border Radius */
  --radius-sm: 0.25rem;  /* 4px */
  --radius-md: 0.375rem; /* 6px */
  --radius-lg: 0.5rem;   /* 8px */
  --radius-full: 9999px;
}
```

---

## 6. ユーザーテストシナリオ案

### 6.1 初心者VTuber「あかり」のシナリオ

1. **初回起動**
   - ツールを起動して5秒以内に何をすべきか理解できるか？
   - OBS接続設定で「localhost」の意味がわかるか？

2. **接続トラブル**
   - OBSが起動していない状態で接続した場合、エラーメッセージから対処法がわかるか？
   - 「ポート番号が違う」場合、どこを確認すればいいか理解できるか？

3. **配信中の監視**
   - CPU使用率90%の警告が表示された場合、何が問題か理解できるか？
   - 「対処法を見る」ボタンから解決策を見つけられるか？

### 6.2 ゲーム配信者「タクヤ」のシナリオ

1. **リアルタイム監視**
   - ゲームプレイ中にフレームドロップが発生した場合、原因を特定できるか？
   - GPU使用率とCPU使用率のどちらがボトルネックか判断できるか？

2. **設定調整**
   - ビットレートを変更した場合、効果がすぐに確認できるか？
   - シーンを切り替えた際、負荷の変化が視覚的にわかるか？

3. **配信停止**
   - 配信中に誤って停止ボタンを押した場合、確認ダイアログで防止できるか？

### 6.3 歌配信者「みさき」のシナリオ

1. **音声優先設定**
   - 音声ビットレートが優先された設定提案があるか？
   - 「高音質」「標準」「低遅延」のプリセットがわかりやすいか？

2. **音ズレ検出**
   - 音ズレが発生した場合、警告が表示されるか？（Phase 2a機能）
   - 解決策が具体的に提示されるか？

---

## 7. 次のステップ

### 7.1 即時対応が必要な項目（Phase 1a）

1. **ObsStreamControls.tsx**: 配信停止時の確認ダイアログ実装
2. **ObsConnectionPanel.tsx**: 初心者向けヒント追加
3. **MetricsPanel.tsx**: セベリティに基づく色分け実装
4. **全コンポーネント**: ARIA属性追加（`role`, `aria-live`, `aria-label`）

### 7.2 中期的な改善項目（Phase 1b）

1. **通知システム**: react-hot-toast導入
2. **ツールチップ**: 専門用語の説明実装
3. **初心者モード**: 用語翻訳トグル実装
4. **キーボード操作**: グローバルショートカット実装

### 7.3 長期的な拡張（Phase 2a以降）

1. **配信中モード**: 自動検出と動作変更
2. **グランスビュー**: 健康状態サマリー画面
3. **ダークモード**: CSS変数ベースの実装
4. **ナビゲーション**: タブUI実装

---

## 8. まとめ

### 8.1 強み

- Tailwind CSSを活用した統一感のあるデザイン
- 適切なローディング状態とエラーハンドリング
- レスポンシブ対応の基盤が整っている
- メトリクス表示が詳細で正確

### 8.2 最も重要な改善点

1. **アクセシビリティ**: WCAG AA準拠に向けたARIA属性追加
2. **初心者UX**: ペルソナ「あかり」のニーズに対応した用語翻訳と誘導
3. **フィードバック**: 成功通知とエラー時の具体的な対処法
4. **誤操作防止**: 配信停止時の確認ダイアログ

### 8.3 提案サマリー

- **即時対応（2週間）**: ARIA属性追加、セベリティ色分け、確認ダイアログ
- **短期（1ヶ月）**: 通知システム、ツールチップ、初心者モード
- **中期（2〜3ヶ月）**: 配信中モード、グランスビュー、ダークモード

このレビューに基づき、ユーザー中心の改善を段階的に実施することで、全ペルソナにとって使いやすいツールになると考えます。

---

**レビュー担当:** UI/UX Designer Agent
**次回レビュー推奨時期:** Phase 1b完了後（通知システム実装後）
