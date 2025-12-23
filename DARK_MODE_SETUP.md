# ダークモード実装 - セットアップガイド

## クイックスタート

### 1. 依存関係のインストール

Tailwind CSSがまだインストールされていない場合:

```bash
pnpm add -D tailwindcss postcss autoprefixer
```

注: REQ-008として依存関係リクエスト済み

### 2. 開発サーバー起動

```bash
pnpm dev
```

### 3. ダークモードの確認

1. アプリケーションのヘッダー右上にThemeToggleボタンが表示される
2. ボタンをクリックして light → dark → system と切り替え
3. 背景色とテキスト色が変化することを確認

## ファイル構成

```
obs_optimizer/
├── tailwind.config.js          # Tailwind設定（darkMode: 'class'）
├── postcss.config.js            # PostCSS設定
├── src/
│   ├── index.css                # CSS変数とダークモードスタイル
│   ├── main.tsx                 # テーマ初期化
│   ├── App.tsx                  # ThemeToggle配置
│   ├── components/
│   │   ├── ThemeToggle.tsx      # テーマ切り替えボタン
│   │   ├── ThemeToggle.test.tsx # テスト
│   │   └── index.ts             # エクスポート
│   └── stores/
│       ├── themeStore.ts        # テーマ状態管理
│       ├── themeStore.test.ts   # テスト
│       └── index.ts             # エクスポート
```

## 使用方法

### コンポーネントでテーマを使用

```tsx
import { useThemeStore } from './stores/themeStore';

function MyComponent() {
  const { mode, resolvedTheme } = useThemeStore();
  
  return (
    <div className="bg-white dark:bg-gray-800">
      <p className="text-gray-900 dark:text-gray-100">
        現在のテーマ: {mode} (実際の表示: {resolvedTheme})
      </p>
    </div>
  );
}
```

### プログラムでテーマを変更

```tsx
import { useThemeStore } from './stores/themeStore';

function SettingsPanel() {
  const { setTheme } = useThemeStore();
  
  return (
    <div>
      <button onClick={() => setTheme('light')}>ライト</button>
      <button onClick={() => setTheme('dark')}>ダーク</button>
      <button onClick={() => setTheme('system')}>システム</button>
    </div>
  );
}
```

## Tailwind dark: バリアント

### 基本パターン

```tsx
// 背景色
<div className="bg-white dark:bg-gray-800">

// テキスト色
<p className="text-gray-900 dark:text-gray-100">

// ボーダー
<div className="border-gray-300 dark:border-gray-600">

// ボタン
<button className="bg-blue-500 dark:bg-blue-600 hover:bg-blue-600 dark:hover:bg-blue-700">
```

### エラー表示

```tsx
<div className="bg-red-100 dark:bg-red-900/30 border-red-300 dark:border-red-700">
  <span className="text-red-700 dark:text-red-400">エラーメッセージ</span>
</div>
```

### 入力フィールド

```tsx
<input 
  className="
    bg-white dark:bg-gray-700 
    text-gray-900 dark:text-gray-100
    border-gray-300 dark:border-gray-600
    focus:ring-blue-500 dark:focus:ring-blue-400
    placeholder:text-gray-400 dark:placeholder:text-gray-500
  "
/>
```

## CSS変数を使用した高度なテーマ

CSS変数で独自のテーマカラーを定義:

```css
/* index.css */
:root {
  --my-primary: #3b82f6;
  --my-secondary: #6b7280;
}

.dark {
  --my-primary: #60a5fa;
  --my-secondary: #9ca3af;
}
```

Tailwindで使用:

```tsx
<div style={{ backgroundColor: 'var(--my-primary)' }}>
```

## テスト

### 単体テスト実行

```bash
pnpm test
```

### カバレッジ取得

```bash
pnpm test:coverage
```

### 手動テスト項目

- [ ] ThemeToggleボタンでテーマ切り替え
- [ ] localStorageに設定が保存される
- [ ] ページリロード後もテーマが維持される
- [ ] システム設定変更時に自動切り替わる（systemモード時）
- [ ] 全コンポーネントで背景色・テキスト色が適切に変化

## トラブルシューティング

### Q: スタイルが適用されない

A: Tailwind CSSがインストールされているか確認:

```bash
pnpm list tailwindcss
```

インストールされていない場合:

```bash
pnpm add -D tailwindcss postcss autoprefixer
```

### Q: dark:クラスが効かない

A: `tailwind.config.js` で `darkMode: 'class'` が設定されているか確認

### Q: テーマが永続化されない

A: localStorage API が利用可能か確認（Tauri環境では通常利用可能）

### Q: システムテーマが反映されない

A: `initializeTheme()` が `main.tsx` で実行されているか確認

## 既存コンポーネントへの適用手順

1. コンポーネントファイルを開く
2. `bg-`, `text-`, `border-` などのクラスを見つける
3. 各クラスに対応する `dark:` バリアントを追加
4. ブラウザで両方のテーマを確認

例:

```diff
- <div className="bg-white text-gray-900">
+ <div className="bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100">
```

## パフォーマンス最適化

- CSS変数を使用してテーマ切り替えを高速化
- トランジション時間を200msに設定
- `prefers-reduced-motion`でアニメーション無効化対応
- Zustand persistで最小限の永続化

## アクセシビリティ

- ARIAラベルでスクリーンリーダー対応
- キーボードナビゲーション対応（Tab + Enter）
- focus-visibleで適切なフォーカス表示
- prefers-color-schemeでシステム設定に追従

## 次のステップ

1. [ ] 残りのコンポーネントに dark: バリアント追加
2. [ ] カスタムカラーテーマの実装
3. [ ] ハイコントラストモードの追加
4. [ ] キーボードショートカット（Ctrl+Shift+D）

## 関連ドキュメント

- [DARK_MODE_IMPLEMENTATION.md](./DARK_MODE_IMPLEMENTATION.md) - 詳細な実装ドキュメント
- [Tailwind CSS Dark Mode](https://tailwindcss.com/docs/dark-mode)
- [Zustand Persist](https://docs.pmnd.rs/zustand/integrations/persisting-store-data)

---

Updated: 2025-12-23
