/**
 * オンボーディング Step 7: 完了画面
 */
export function Complete() {
  return (
    <div className="text-center py-8">
      <div className="mb-6">
        <div className="w-20 h-20 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
          <svg
            className="w-10 h-10 text-green-600"
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
        <h2 className="text-2xl font-bold text-gray-900 mb-2">
          セットアップ完了！
        </h2>
        <p className="text-gray-600">
          OBS配信最適化ツールの準備が整いました
        </p>
      </div>

      <div className="max-w-2xl mx-auto space-y-6">
        {/* 次のステップ */}
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-6 text-left">
          <h3 className="font-semibold text-blue-900 mb-3">次にできること</h3>
          <ul className="space-y-2 text-sm text-blue-800">
            <li className="flex items-start gap-2">
              <span className="flex-shrink-0 mt-0.5">✓</span>
              <span>リアルタイムでCPU・メモリ使用率をモニタリング</span>
            </li>
            <li className="flex items-start gap-2">
              <span className="flex-shrink-0 mt-0.5">✓</span>
              <span>配信・録画の開始/停止をワンクリック操作</span>
            </li>
            <li className="flex items-start gap-2">
              <span className="flex-shrink-0 mt-0.5">✓</span>
              <span>定期的に設定を再診断して最適化</span>
            </li>
            <li className="flex items-start gap-2">
              <span className="flex-shrink-0 mt-0.5">✓</span>
              <span>通知履歴で問題を早期発見</span>
            </li>
          </ul>
        </div>

        {/* ヒント */}
        <div className="bg-gray-50 border border-gray-200 rounded-lg p-6 text-left">
          <h3 className="font-semibold text-gray-900 mb-3">ヒント</h3>
          <ul className="space-y-2 text-sm text-gray-700">
            <li className="flex items-start gap-2">
              <span className="flex-shrink-0 mt-0.5">💡</span>
              <span>
                配信スタイルやPCスペックが変わった場合は、再度診断を実行すると最適な設定が得られます
              </span>
            </li>
            <li className="flex items-start gap-2">
              <span className="flex-shrink-0 mt-0.5">💡</span>
              <span>
                配信中はストリーミングモードがONになり、重要な通知のみ表示されます
              </span>
            </li>
            <li className="flex items-start gap-2">
              <span className="flex-shrink-0 mt-0.5">💡</span>
              <span>
                設定は後からいつでも変更できます
              </span>
            </li>
          </ul>
        </div>
      </div>
    </div>
  );
}
