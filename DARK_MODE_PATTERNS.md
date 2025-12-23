# ダークモード実装パターン集

このドキュメントでは、既存コンポーネントにダークモードを追加する際の具体的なパターンを示します。

## 基本原則

1. **常にペアで追加**: ライトモードとダークモードのクラスを必ずペアで記述
2. **コントラスト確保**: 可読性を保つため、十分なコントラスト比を確保
3. **一貫性**: 同じ用途の要素には同じカラーパターンを使用

## カラーマッピング表

### 背景色

| 用途 | ライトモード | ダークモード |
|------|-------------|-------------|
| メイン背景 | `bg-gray-100` | `bg-gray-900` |
| カード背景 | `bg-white` | `bg-gray-800` |
| セカンダリ背景 | `bg-gray-50` | `bg-gray-700` |
| ホバー背景 | `hover:bg-gray-100` | `hover:bg-gray-700` |

### テキスト色

| 用途 | ライトモード | ダークモード |
|------|-------------|-------------|
| プライマリテキスト | `text-gray-900` | `text-gray-100` |
| セカンダリテキスト | `text-gray-600` | `text-gray-300` |
| ヘルプテキスト | `text-gray-500` | `text-gray-400` |

### ボーダー

| 用途 | ライトモード | ダークモード |
|------|-------------|-------------|
| 通常ボーダー | `border-gray-300` | `border-gray-600` |
| ホバーボーダー | `border-gray-400` | `border-gray-500` |
| フォーカスリング | `ring-blue-500` | `ring-blue-400` |

### アクセントカラー

| 用途 | ライトモード | ダークモード |
|------|-------------|-------------|
| プライマリボタン | `bg-blue-500` | `bg-blue-600` |
| プライマリホバー | `hover:bg-blue-600` | `hover:bg-blue-700` |
| エラー背景 | `bg-red-100` | `bg-red-900/30` |
| エラーテキスト | `text-red-700` | `text-red-400` |
| エラーボーダー | `border-red-300` | `border-red-700` |
| 警告背景 | `bg-yellow-100` | `bg-yellow-900/30` |
| 警告テキスト | `text-yellow-700` | `text-yellow-400` |
| 成功背景 | `bg-green-100` | `bg-green-900/30` |
| 成功テキスト | `text-green-700` | `text-green-400` |

## パターン別実装例

### パターン1: カードコンポーネント

```tsx
// Before
<div className="bg-white rounded-lg shadow-md p-6">
  <h3 className="text-lg font-semibold text-gray-800 mb-4">
    タイトル
  </h3>
  <p className="text-gray-600">
    説明文
  </p>
</div>

// After
<div className="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
  <h3 className="text-lg font-semibold text-gray-800 dark:text-gray-100 mb-4">
    タイトル
  </h3>
  <p className="text-gray-600 dark:text-gray-300">
    説明文
  </p>
</div>
```

### パターン2: 入力フォーム

```tsx
// Before
<div>
  <label className="block text-sm font-medium text-gray-700 mb-1">
    ラベル
  </label>
  <input
    type="text"
    className="w-full px-3 py-2 border border-gray-300 rounded-md
               focus:outline-none focus:ring-2 focus:ring-blue-500"
    placeholder="入力してください"
  />
  <p className="mt-1 text-xs text-gray-500">
    ヘルプテキスト
  </p>
</div>

// After
<div>
  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
    ラベル
  </label>
  <input
    type="text"
    className="w-full px-3 py-2
               border border-gray-300 dark:border-gray-600
               bg-white dark:bg-gray-700
               text-gray-900 dark:text-gray-100
               rounded-md
               focus:outline-none focus:ring-2
               focus:ring-blue-500 dark:focus:ring-blue-400
               placeholder:text-gray-400 dark:placeholder:text-gray-500"
    placeholder="入力してください"
  />
  <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
    ヘルプテキスト
  </p>
</div>
```

### パターン3: ボタン

```tsx
// Primary Button
<button className="
  px-4 py-2
  bg-blue-500 dark:bg-blue-600
  text-white
  rounded-md
  hover:bg-blue-600 dark:hover:bg-blue-700
  disabled:opacity-50
  transition-colors
">
  保存
</button>

// Secondary Button
<button className="
  px-4 py-2
  bg-gray-200 dark:bg-gray-700
  text-gray-800 dark:text-gray-200
  rounded-md
  hover:bg-gray-300 dark:hover:bg-gray-600
  transition-colors
">
  キャンセル
</button>

// Danger Button
<button className="
  px-4 py-2
  bg-red-500 dark:bg-red-600
  text-white
  rounded-md
  hover:bg-red-600 dark:hover:bg-red-700
  transition-colors
">
  削除
</button>
```

### パターン4: エラー表示

```tsx
// Before
<div className="p-3 bg-red-100 border border-red-300 rounded-md">
  <span className="text-sm font-medium text-red-700">
    エラーが発生しました
  </span>
  <p className="text-xs text-red-600">
    詳細: ファイルが見つかりません
  </p>
</div>

// After
<div className="
  p-3
  bg-red-100 dark:bg-red-900/30
  border border-red-300 dark:border-red-700
  rounded-md
">
  <span className="text-sm font-medium text-red-700 dark:text-red-400">
    エラーが発生しました
  </span>
  <p className="text-xs text-red-600 dark:text-red-400">
    詳細: ファイルが見つかりません
  </p>
</div>
```

### パターン5: ステータスバッジ

```tsx
// Success Badge
<span className="
  inline-flex items-center
  px-2.5 py-0.5
  rounded-full
  text-xs font-medium
  bg-green-100 dark:bg-green-900/30
  text-green-800 dark:text-green-400
">
  成功
</span>

// Warning Badge
<span className="
  inline-flex items-center
  px-2.5 py-0.5
  rounded-full
  text-xs font-medium
  bg-yellow-100 dark:bg-yellow-900/30
  text-yellow-800 dark:text-yellow-400
">
  警告
</span>

// Error Badge
<span className="
  inline-flex items-center
  px-2.5 py-0.5
  rounded-full
  text-xs font-medium
  bg-red-100 dark:bg-red-900/30
  text-red-800 dark:text-red-400
">
  エラー
</span>
```

### パターン6: テーブル

```tsx
<table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
  <thead className="bg-gray-50 dark:bg-gray-800">
    <tr>
      <th className="
        px-6 py-3
        text-left text-xs font-medium
        text-gray-500 dark:text-gray-400
        uppercase tracking-wider
      ">
        項目名
      </th>
    </tr>
  </thead>
  <tbody className="
    bg-white dark:bg-gray-900
    divide-y divide-gray-200 dark:divide-gray-800
  ">
    <tr className="hover:bg-gray-50 dark:hover:bg-gray-800">
      <td className="px-6 py-4 text-sm text-gray-900 dark:text-gray-100">
        データ
      </td>
    </tr>
  </tbody>
</table>
```

### パターン7: モーダル/ダイアログ

```tsx
<div className="fixed inset-0 bg-gray-500 dark:bg-gray-900 bg-opacity-75 dark:bg-opacity-75">
  <div className="
    bg-white dark:bg-gray-800
    rounded-lg
    p-6
    max-w-md mx-auto mt-20
    shadow-xl
  ">
    <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
      確認
    </h2>
    <p className="text-gray-600 dark:text-gray-300 mb-6">
      本当に削除しますか？
    </p>
    <div className="flex gap-3 justify-end">
      <button className="
        px-4 py-2
        bg-gray-200 dark:bg-gray-700
        text-gray-800 dark:text-gray-200
        rounded-md
      ">
        キャンセル
      </button>
      <button className="
        px-4 py-2
        bg-red-500 dark:bg-red-600
        text-white
        rounded-md
      ">
        削除
      </button>
    </div>
  </div>
</div>
```

### パターン8: ナビゲーション

```tsx
<nav className="bg-white dark:bg-gray-800 shadow">
  <div className="max-w-7xl mx-auto px-4">
    <div className="flex justify-between h-16">
      <div className="flex space-x-8">
        <a
          href="#"
          className="
            inline-flex items-center
            px-1 pt-1
            border-b-2 border-blue-500
            text-sm font-medium
            text-gray-900 dark:text-gray-100
          "
        >
          ホーム
        </a>
        <a
          href="#"
          className="
            inline-flex items-center
            px-1 pt-1
            border-b-2 border-transparent
            text-sm font-medium
            text-gray-500 dark:text-gray-400
            hover:text-gray-700 dark:hover:text-gray-300
            hover:border-gray-300 dark:hover:border-gray-600
          "
        >
          設定
        </a>
      </div>
    </div>
  </div>
</nav>
```

### パターン9: ツールチップ

```tsx
<div className="relative group">
  <button className="text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200">
    ?
  </button>
  <div className="
    absolute bottom-full left-1/2 -translate-x-1/2 mb-2
    px-3 py-2
    bg-gray-900 dark:bg-gray-700
    text-white dark:text-gray-100
    text-sm rounded-md
    opacity-0 group-hover:opacity-100
    transition-opacity
    pointer-events-none
  ">
    ヘルプテキスト
  </div>
</div>
```

### パターン10: ローディングスピナー

```tsx
<div className="flex items-center justify-center">
  <div className="
    animate-spin
    rounded-full
    h-8 w-8
    border-b-2
    border-blue-600 dark:border-blue-400
  " />
</div>
```

## チェックリスト

コンポーネントにダークモードを追加する際のチェックリスト:

- [ ] 背景色に `dark:bg-*` を追加
- [ ] テキスト色に `dark:text-*` を追加
- [ ] ボーダー色に `dark:border-*` を追加
- [ ] ホバー状態に `dark:hover:*` を追加
- [ ] フォーカス状態に `dark:focus:*` を追加
- [ ] プレースホルダーに `dark:placeholder:*` を追加
- [ ] disabled状態に `dark:disabled:*` を追加（必要に応じて）
- [ ] アイコンやSVGの色を調整
- [ ] 両方のテーマで視覚的に確認

## ツール

### VS Code拡張機能

- **Tailwind CSS IntelliSense**: クラス名の自動補完
- **Error Lens**: インラインエラー表示

### ブラウザ拡張機能

- **React Developer Tools**: コンポーネントの状態確認
- **Tailwind CSS Viewer**: Tailwindクラスのプレビュー

## よくある間違い

### 間違い1: dark:のみ追加してライトモードを削除

```tsx
// ❌ 間違い
<div className="dark:bg-gray-800">

// ✅ 正しい
<div className="bg-white dark:bg-gray-800">
```

### 間違い2: コントラスト不足

```tsx
// ❌ 間違い（ダークモードでコントラストが低い）
<div className="bg-gray-900 dark:bg-gray-800">
  <p className="text-gray-800 dark:text-gray-700">テキスト</p>
</div>

// ✅ 正しい
<div className="bg-white dark:bg-gray-800">
  <p className="text-gray-900 dark:text-gray-100">テキスト</p>
</div>
```

### 間違い3: 透明度の不適切な使用

```tsx
// ❌ 間違い（ライトモードで見えづらい）
<div className="bg-red-900/30 dark:bg-red-900/30">

// ✅ 正しい
<div className="bg-red-100 dark:bg-red-900/30">
```

## リファレンス

- [Tailwind CSS Colors](https://tailwindcss.com/docs/customizing-colors)
- [Tailwind CSS Dark Mode](https://tailwindcss.com/docs/dark-mode)
- [WCAG Contrast Guidelines](https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html)

---

Updated: 2025-12-23
