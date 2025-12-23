/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  // クラスベースのダークモード（htmlタグにdarkクラスを追加）
  darkMode: 'class',
  theme: {
    extend: {
      fontFamily: {
        // サンセリフ: 日本語対応のデフォルトフォント
        // Windows/Mac両対応で最適な日本語表示を実現
        sans: [
          'Inter',
          'Noto Sans JP',
          'Hiragino Sans',
          'Hiragino Kaku Gothic ProN',
          'Yu Gothic UI',
          'Meiryo',
          'sans-serif',
        ],
        // モノスペース: 数値表示用（CPU使用率、FPS、ビットレートなど）
        // 数字の桁揃えが重要なメトリクス表示に使用
        mono: [
          'JetBrains Mono',
          'Source Code Pro',
          'Noto Sans Mono CJK JP',
          'Consolas',
          'Monaco',
          'Courier New',
          'monospace',
        ],
      },
      // フォントウェイトの最適化
      // 日本語フォントで利用可能なウェイトを定義
      fontWeight: {
        light: '300',
        normal: '400',
        medium: '500',
        semibold: '600',
        bold: '700',
      },
      colors: {
        // カスタムカラー（必要に応じて拡張）
        primary: {
          light: '#3b82f6', // blue-500
          dark: '#60a5fa',  // blue-400
        },
      },
    },
  },
  plugins: [],
}
