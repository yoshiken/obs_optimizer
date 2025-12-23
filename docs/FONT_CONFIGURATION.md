# 日本語フォント設定ガイド

## 概要

OBS配信最適化ツールは、Windows/Mac両対応の最適化された日本語フォントスタックを使用しています。

## フォントスタック構成

### サンセリフ（デフォルト）

```css
font-family: 'Inter', 'Noto Sans JP', 'Hiragino Sans', 'Hiragino Kaku Gothic ProN',
             'Yu Gothic UI', 'Meiryo', sans-serif;
```

#### フォント優先順位

1. **Inter** - 英数字用の現代的なサンセリフフォント
2. **Noto Sans JP** - Googleの日本語フォント（クロスプラットフォーム対応）
3. **Hiragino Sans** - macOS Catalina以降の標準日本語フォント
4. **Hiragino Kaku Gothic ProN** - macOS Mojave以前の標準日本語フォント
5. **Yu Gothic UI** - Windows 10/11の高品質UI向け日本語フォント
6. **Meiryo** - Windows 7/8のフォールバック
7. **sans-serif** - システムデフォルトフォント

### モノスペース（数値表示用）

```css
font-family: 'JetBrains Mono', 'Source Code Pro', 'Noto Sans Mono CJK JP',
             'Consolas', 'Monaco', 'Courier New', 'monospace';
```

#### 使用箇所

- CPU/GPU使用率
- フレームレート（FPS）
- ビットレート
- その他の数値メトリクス

#### 特徴

- `font-variant-numeric: tabular-nums` で数字を等幅表示
- 数値の桁揃えを改善し、可読性を向上

## 設定ファイル

### 1. Tailwind設定 (`/home/yskn/git/obs_optimizer/tailwind.config.js`)

```javascript
theme: {
  extend: {
    fontFamily: {
      sans: ['Inter', 'Noto Sans JP', ...],
      mono: ['JetBrains Mono', 'Source Code Pro', ...],
    },
  },
}
```

### 2. グローバルCSS (`/home/yskn/git/obs_optimizer/src/index.css`)

```css
:root {
  font-family: 'Inter', 'Noto Sans JP', 'Hiragino Sans', ...;
}
```

### 3. レガシーCSS (`/home/yskn/git/obs_optimizer/src/App.css`)

互換性のため、App.cssにも同様の設定を保持しています。

## 使用方法

### Tailwindクラス使用時

```tsx
// デフォルトのサンセリフフォント（指定不要）
<h1 className="text-2xl font-bold">OBS配信最適化ツール</h1>

// モノスペースフォント（数値表示）
<span className="font-mono text-lg">85.3%</span>

// フォントウェイト
<p className="font-normal">通常</p>
<p className="font-medium">中太</p>
<p className="font-bold">太字</p>
```

### カスタムCSSクラス使用時

```tsx
// メトリクス値の表示
<div className="metric-value">85.3%</div>

// 数値専用のモノスペースフォント
<span className="font-mono-metric">1920x1080</span>

// 日本語フォントウェイト最適化
<h2 className="font-ja-bold">セクション見出し</h2>
```

### data属性使用時

```tsx
// data-numeric属性で自動的にモノスペースフォントを適用
<span data-numeric="true">60</span>
```

## フォントウェイトの推奨値

日本語フォントは通常、以下のウェイトが推奨されます：

| ウェイト | 値  | 用途 |
|---------|-----|------|
| Normal  | 400 | 本文、通常テキスト |
| Medium  | 500 | 小見出し、強調テキスト |
| Bold    | 700 | 大見出し、重要な情報 |

**注意**: 多くの日本語フォントでは300（Light）や600（Semibold）が不明瞭になる場合があります。

## プラットフォーム別の表示フォント

### Windows 10/11

- 英数字: Inter
- 日本語: Yu Gothic UI

### Windows 7/8

- 英数字: Inter
- 日本語: Meiryo

### macOS Catalina以降

- 英数字: Inter
- 日本語: Hiragino Sans

### macOS Mojave以前

- 英数字: Inter
- 日本語: Hiragino Kaku Gothic ProN

## レンダリング最適化

以下のCSS設定により、日本語フォントの可読性を向上させています：

```css
-webkit-font-smoothing: antialiased;
-moz-osx-font-smoothing: grayscale;
text-rendering: optimizeLegibility;
```

## トラブルシューティング

### フォントが反映されない場合

1. Tailwind CSSがインストールされているか確認
   ```bash
   pnpm list tailwindcss
   ```

2. ビルドを再実行
   ```bash
   pnpm tauri build
   ```

3. ブラウザキャッシュをクリア（開発時）
   ```
   Ctrl + Shift + R (Windows/Linux)
   Cmd + Shift + R (Mac)
   ```

### 数値の桁揃えがずれる場合

1. `font-mono`または`metric-value`クラスが適用されているか確認
2. `font-variant-numeric: tabular-nums`が適用されているか確認

## 依存関係

必要なパッケージ（REQ-008で申請中）：

- `tailwindcss@^3.4.17`
- `postcss@^8.4.49`
- `autoprefixer@^10.4.20`

## 関連ファイル

- `/home/yskn/git/obs_optimizer/tailwind.config.js` - Tailwind設定
- `/home/yskn/git/obs_optimizer/postcss.config.js` - PostCSS設定
- `/home/yskn/git/obs_optimizer/src/index.css` - グローバルCSS
- `/home/yskn/git/obs_optimizer/src/App.css` - レガシーCSS（互換性用）
