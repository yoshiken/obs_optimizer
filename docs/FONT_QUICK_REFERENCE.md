# フォント設定 - クイックリファレンス

## 基本的な使い方

### Tailwind CSS クラス

```tsx
// デフォルトフォント（自動適用、指定不要）
<h1 className="text-2xl font-bold">見出し</h1>

// モノスペースフォント（数値表示）
<span className="font-mono">85.3%</span>

// フォントウェイト
<p className="font-normal">通常 (400)</p>
<p className="font-medium">中太 (500)</p>
<p className="font-bold">太字 (700)</p>
```

### カスタムCSSクラス

```tsx
// メトリクス値（推奨）
<div className="metric-value">85.3%</div>

// モノスペース数値
<span className="font-mono-metric">1920x1080</span>

// 日本語ウェイト最適化
<h2 className="font-ja-bold">セクション</h2>
```

## フォントスタック

### サンセリフ（デフォルト）
```
Inter → Noto Sans JP → Hiragino Sans → Yu Gothic UI → Meiryo
```

### モノスペース（数値用）
```
JetBrains Mono → Source Code Pro → Noto Sans Mono CJK JP → Consolas
```

## 推奨ウェイト

| ウェイト | 値  | 用途 |
|---------|-----|------|
| Normal  | 400 | 本文 |
| Medium  | 500 | 小見出し |
| Bold    | 700 | 大見出し |

## 適用例

### メトリクスカード
```tsx
<div className="bg-blue-50 p-4 rounded-lg">
  <p className="text-sm text-blue-600 font-medium">CPU使用率</p>
  <p className="metric-value text-3xl font-bold text-blue-700">45.8%</p>
</div>
```

### 混在テキスト
```tsx
<p className="text-base">
  解像度: <span className="font-mono">1920x1080</span> @
  <span className="font-mono">60fps</span>
</p>
```

## ファイル

- 設定: `/home/yskn/git/obs_optimizer/tailwind.config.js`
- CSS: `/home/yskn/git/obs_optimizer/src/index.css`
- 詳細: `/home/yskn/git/obs_optimizer/docs/FONT_CONFIGURATION.md`
