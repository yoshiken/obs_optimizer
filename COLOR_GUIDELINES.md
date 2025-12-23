# カラーガイドライン - OBS配信最適化ツール

## Tailwind CSS テキストカラー使用ルール

### 推奨カラーパレット (白背景 #FFFFFF)

| Tailwind クラス | カラーコード | コントラスト比 | 用途 | 使用可否 |
|----------------|------------|--------------|------|---------|
| `text-gray-900` | #111827 | 16.1:1 | 最も重要な見出し | ✅ 推奨 |
| `text-gray-800` | #1F2937 | 12.6:1 | 見出し、強調テキスト | ✅ 推奨 |
| `text-gray-700` | #374151 | 8.6:1 | 重要なメッセージ、本文 | ✅ 推奨 |
| `text-gray-600` | #4B5563 | 5.9:1 | 補助テキスト、ラベル、ヒント | ✅ 推奨 |
| `text-gray-500` | #6B7280 | 3.2:1 | 大きいグラフィック要素のみ | ⚠️ 限定的 |
| `text-gray-400` | #9CA3AF | 2.6:1 | 使用禁止 | ❌ 不可 |
| `text-gray-300` | #D1D5DB | 1.7:1 | 使用禁止 | ❌ 不可 |

### グレー背景 (#F9FAFB) での使用

| Tailwind クラス | コントラスト比 | 使用可否 |
|----------------|--------------|---------|
| `text-gray-800` | 12.0:1 | ✅ 推奨 |
| `text-gray-700` | 8.2:1 | ✅ 推奨 |
| `text-gray-600` | 5.6:1 | ✅ 推奨 |

## 具体的な使用例

### ✅ 正しい使い方

#### 1. 見出し
```tsx
<h1 className="text-gray-900">メインタイトル</h1>
<h2 className="text-gray-800">セクションタイトル</h2>
<h3 className="text-gray-800">サブセクション</h3>
```

#### 2. 本文テキスト
```tsx
<p className="text-gray-700">
  これは本文テキストです。重要な情報を含みます。
</p>
```

#### 3. 補助テキスト・ラベル
```tsx
<label className="text-gray-700">ホスト</label>
<span className="text-gray-600">補足説明やヒント</span>
<small className="text-gray-600">注意書き</small>
```

#### 4. メタ情報 (小さいテキスト)
```tsx
<time className="text-xs text-gray-600">2025-12-23</time>
<span className="text-xs text-gray-600">
  ({metrics.cpu.coreCount}コア)
</span>
```

#### 5. リンク・ボタン
```tsx
<a href="#" className="text-blue-600 hover:text-blue-700">
  詳細を見る
</a>
<button className="text-gray-700 hover:text-gray-900">
  キャンセル
</button>
```

#### 6. エラー・警告メッセージ
```tsx
<p className="text-red-700">エラー: 接続できませんでした</p>
<p className="text-yellow-700">警告: 設定を確認してください</p>
<p className="text-green-700">成功: 保存しました</p>
```

### ❌ 避けるべき使い方

#### 1. 重要な情報に薄い色を使う
```tsx
{/* ❌ BAD */}
<p className="text-gray-500">重要なお知らせ</p>
<p className="text-gray-400">エラーメッセージ</p>

{/* ✅ GOOD */}
<p className="text-gray-700">重要なお知らせ</p>
<p className="text-red-700">エラーメッセージ</p>
```

#### 2. 小さいテキストに低コントラスト
```tsx
{/* ❌ BAD */}
<small className="text-xs text-gray-500">
  ヒント: この設定は重要です
</small>

{/* ✅ GOOD */}
<small className="text-xs text-gray-600">
  ヒント: この設定は重要です
</small>
```

#### 3. インタラクティブ要素の不明瞭な状態
```tsx
{/* ❌ BAD */}
<button className="text-gray-400">クリック</button>

{/* ✅ GOOD */}
<button className="text-gray-700 hover:text-gray-900">
  クリック
</button>
<button disabled className="text-gray-700 opacity-50 cursor-not-allowed">
  無効化
</button>
```

## 特殊なケース

### 1. 無効化された要素
```tsx
{/* 背景を暗くする + opacity で視覚的に無効を示す */}
<button
  disabled
  className="text-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
>
  無効なボタン
</button>
```

### 2. プレースホルダー
```tsx
{/* プレースホルダーはWCAG対象外だが、できるだけ読みやすく */}
<input
  type="text"
  placeholder="入力してください"
  className="placeholder:text-gray-500"
/>
```

### 3. 装飾的なアイコン
```tsx
{/* aria-hidden="true" で支援技術から隠す */}
<svg
  className="text-gray-500"
  aria-hidden="true"
>
  {/* アイコンの内容 */}
</svg>
```

### 4. ツールチップ
```tsx
{/* ツールチップは背景が暗いので明るい文字を使う */}
<div className="bg-gray-900 text-gray-100 px-3 py-2 rounded">
  これはツールチップです
</div>
```

## コンポーネント別推奨カラー

### OBS機能コンポーネント

| 要素 | 推奨クラス | コントラスト比 |
|------|-----------|--------------|
| パネルタイトル | `text-gray-800` | 12.6:1 |
| ステータスラベル | `text-gray-700` | 8.6:1 |
| メトリクス値 | `text-gray-800` | 12.6:1 |
| 補助情報 | `text-gray-600` | 5.9:1 |
| ヒント | `text-gray-600` | 5.9:1 |
| エラーメッセージ | `text-red-700` | 8.2:1 |

### モニタリングコンポーネント

| 要素 | 推奨クラス | コントラスト比 |
|------|-----------|--------------|
| カードタイトル | `text-gray-700` | 8.6:1 |
| メトリクス値 (大) | `text-gray-800` | 12.6:1 |
| 単位表記 | `text-gray-600` | 5.9:1 |
| ラベル | `text-gray-600` | 5.9:1 |

### ダイアログ・モーダル

| 要素 | 推奨クラス | コントラスト比 |
|------|-----------|--------------|
| タイトル | `text-gray-800` | 12.6:1 |
| メッセージ | `text-gray-700` | 8.6:1 |
| ボタンテキスト (白背景ボタン) | `text-gray-700` | 8.6:1 |

## チェックリスト

開発時に以下を確認してください:

- [ ] テキストカラーは `text-gray-600` 以上を使用しているか
- [ ] 小さいテキスト (14px未満) で `text-gray-500` を使っていないか
- [ ] 重要な情報に適切な濃さの色を使用しているか
- [ ] エラー・警告には専用の色 (red, yellow) を使用しているか
- [ ] 装飾的な要素には `aria-hidden="true"` を付けているか
- [ ] 無効化状態は `opacity` または `disabled` スタイルを使用しているか

## ツール

### コントラスト比チェッカー
1. **WebAIM Contrast Checker**
   - https://webaim.org/resources/contrastchecker/

2. **Chrome DevTools**
   - Elements タブで色をホバー → コントラスト比が表示される

3. **Tailwind CSS公式カラーパレット**
   - https://tailwindcss.com/docs/customizing-colors

### コントラスト比計算式
```
コントラスト比 = (L1 + 0.05) / (L2 + 0.05)
L = 明度 (0-1の範囲)
```

## 参考資料

- [WCAG 2.1 Understanding SC 1.4.3: Contrast (Minimum)](https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html)
- [WCAG 2.1 Understanding SC 1.4.6: Contrast (Enhanced)](https://www.w3.org/WAI/WCAG21/Understanding/contrast-enhanced.html)
- [WebAIM: Contrast and Color Accessibility](https://webaim.org/articles/contrast/)

---

**最終更新**: 2025-12-23
**バージョン**: 1.0.0
**維持管理者**: SESSION_UI
