/**
 * 日本語フォント設定のデモコンポーネント
 *
 * このコンポーネントは、フォント設定の使用例を示します。
 * 本番環境では使用せず、開発時の参考として利用してください。
 */

export function FontExample() {
  return (
    <div className="max-w-4xl mx-auto p-8 space-y-8">
      {/* サンセリフフォントの例 */}
      <section className="bg-white rounded-lg shadow-md p-6 space-y-4">
        <h2 className="text-2xl font-bold text-gray-900">
          サンセリフフォント（デフォルト）
        </h2>

        <div className="space-y-2">
          <p className="text-sm text-gray-600">
            フォントウェイト: Normal (400)
          </p>
          <p className="font-normal text-gray-900">
            日本語テキストの通常表示です。OBS配信最適化ツールは、配信品質を向上させます。
          </p>
        </div>

        <div className="space-y-2">
          <p className="text-sm text-gray-600">
            フォントウェイト: Medium (500)
          </p>
          <p className="font-medium text-gray-900">
            日本語テキストの中太表示です。OBS配信最適化ツールは、配信品質を向上させます。
          </p>
        </div>

        <div className="space-y-2">
          <p className="text-sm text-gray-600">
            フォントウェイト: Bold (700)
          </p>
          <p className="font-bold text-gray-900">
            日本語テキストの太字表示です。OBS配信最適化ツールは、配信品質を向上させます。
          </p>
        </div>

        <div className="mt-4 p-4 bg-gray-50 rounded">
          <p className="text-xs text-gray-500 font-mono">
            font-family: 'Inter', 'Noto Sans JP', 'Hiragino Sans',
            'Hiragino Kaku Gothic ProN', 'Yu Gothic UI', 'Meiryo', sans-serif
          </p>
        </div>
      </section>

      {/* モノスペースフォントの例 */}
      <section className="bg-white rounded-lg shadow-md p-6 space-y-4">
        <h2 className="text-2xl font-bold text-gray-900">
          モノスペースフォント（数値表示用）
        </h2>

        <div className="space-y-4">
          <div className="space-y-2">
            <p className="text-sm text-gray-600">
              Tailwind: font-mono
            </p>
            <div className="flex items-baseline gap-4">
              <span className="font-mono text-2xl text-gray-900">85.3%</span>
              <span className="font-mono text-2xl text-gray-900">60 FPS</span>
              <span className="font-mono text-2xl text-gray-900">6000 kbps</span>
            </div>
          </div>

          <div className="space-y-2">
            <p className="text-sm text-gray-600">
              カスタムクラス: metric-value
            </p>
            <div className="flex items-baseline gap-4">
              <span className="metric-value text-2xl text-gray-900">85.3%</span>
              <span className="metric-value text-2xl text-gray-900">60 FPS</span>
              <span className="metric-value text-2xl text-gray-900">6000 kbps</span>
            </div>
          </div>

          <div className="space-y-2">
            <p className="text-sm text-gray-600">
              カスタムクラス: font-mono-metric
            </p>
            <div className="flex items-baseline gap-4">
              <span className="font-mono-metric text-2xl text-gray-900">85.3%</span>
              <span className="font-mono-metric text-2xl text-gray-900">60 FPS</span>
              <span className="font-mono-metric text-2xl text-gray-900">6000 kbps</span>
            </div>
          </div>
        </div>

        <div className="mt-4 p-4 bg-gray-50 rounded">
          <p className="text-xs text-gray-500 font-mono">
            font-family: 'JetBrains Mono', 'Source Code Pro',
            'Noto Sans Mono CJK JP', 'Consolas', 'Monaco', 'Courier New', monospace
          </p>
          <p className="text-xs text-gray-500 font-mono mt-2">
            font-variant-numeric: tabular-nums (数字を等幅で表示)
          </p>
        </div>
      </section>

      {/* メトリクス表示の実例 */}
      <section className="bg-white rounded-lg shadow-md p-6 space-y-4">
        <h2 className="text-2xl font-bold text-gray-900">
          メトリクス表示の実例
        </h2>

        <div className="grid grid-cols-3 gap-4">
          <div className="bg-blue-50 p-4 rounded-lg">
            <p className="text-sm text-blue-600 font-medium mb-1">CPU使用率</p>
            <p className="metric-value text-3xl font-bold text-blue-700">45.8%</p>
          </div>

          <div className="bg-green-50 p-4 rounded-lg">
            <p className="text-sm text-green-600 font-medium mb-1">GPU使用率</p>
            <p className="metric-value text-3xl font-bold text-green-700">72.3%</p>
          </div>

          <div className="bg-purple-50 p-4 rounded-lg">
            <p className="text-sm text-purple-600 font-medium mb-1">FPS</p>
            <p className="metric-value text-3xl font-bold text-purple-700">60</p>
          </div>
        </div>

        <div className="mt-4 p-4 bg-gray-50 rounded">
          <p className="text-xs text-gray-500">
            数値がすべて等幅で表示され、桁揃えが改善されていることを確認できます。
          </p>
        </div>
      </section>

      {/* 日本語とアルファベットの混在 */}
      <section className="bg-white rounded-lg shadow-md p-6 space-y-4">
        <h2 className="text-2xl font-bold text-gray-900">
          日本語と英数字の混在表示
        </h2>

        <div className="space-y-3">
          <p className="text-base text-gray-900">
            OBS配信最適化ツールは、Open Broadcaster Softwareの設定を最適化します。
          </p>
          <p className="text-base text-gray-900">
            CPUとGPUの使用率を監視し、リアルタイムで配信品質を改善します。
          </p>
          <p className="text-base text-gray-900">
            解像度: <span className="font-mono">1920x1080</span> @ <span className="font-mono">60fps</span>
          </p>
          <p className="text-base text-gray-900">
            ビットレート: <span className="font-mono">6000 kbps</span> (推奨: <span className="font-mono">4500-6500 kbps</span>)
          </p>
        </div>

        <div className="mt-4 p-4 bg-gray-50 rounded">
          <p className="text-xs text-gray-500">
            日本語と英数字が混在する場合でも、読みやすく表示されます。
            数値部分には font-mono を適用することで、可読性が向上します。
          </p>
        </div>
      </section>
    </div>
  );
}
