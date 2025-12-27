# フロントエンド アーキテクチャガイドライン

## 1. ディレクトリ構造パターン

Feature-Sliced Design に近い構造を採用。

```
src/
├── components/          # 共有UIコンポーネント（再利用可能）
│   ├── common/          # 汎用コンポーネント（ConfirmDialog等）
│   ├── Toast.tsx        # 通知コンポーネント
│   └── ThemeToggle.tsx  # テーマ切り替え
├── features/            # 機能単位のモジュール
│   ├── obs/             # OBS接続・制御
│   │   ├── index.ts     # パブリックAPI（export集約）
│   │   ├── ObsConnectionPanel.tsx
│   │   └── ObsStatusIndicator.tsx
│   ├── monitor/         # システム監視
│   │   ├── index.ts
│   │   ├── utils/       # 機能固有のユーティリティ
│   │   └── components/  # 機能内部の小コンポーネント
│   ├── optimization/    # 設定最適化
│   └── onboarding/      # 初回セットアップウィザード
├── hooks/               # 共有カスタムフック
├── stores/              # Zustandストア（グローバル状態）
├── types/               # 型定義
│   └── commands.ts      # Tauri連携の型定義（信頼の源泉）
└── utils/               # 共有ユーティリティ
```

## 2. コンポーネント階層パターン

3層構造を採用:

```
[App.tsx] ─────────────────────────────────────────────────
    │
    │  ページレベル（タブコンテンツ）
    ├─→ [DashboardTab, AnalysisTab, OptimizationTab...]
    │
    │  機能コンポーネント（features/以下）
    ├─→ [ObsConnectionPanel, MetricsPanel, ProblemDashboard...]
    │
    │  共有UIコンポーネント（components/以下）
    └─→ [MetricCard, ConfirmDialog, Toast...]
```

### 階層間の依存ルール

- 上位層は下位層をimportできる
- 下位層は上位層に依存しない
- 同一層間のimportは `index.ts` 経由のパブリックAPIのみ

## 3. 状態管理パターン（Store per Domain）

| ストア | 責務 |
|--------|------|
| `useObsStore` | OBS接続状態、配信/録画状態 |
| `useMetricsStore` | システムメトリクス |
| `useConfigStore` | アプリケーション設定 |
| `useAlertStore` | アラート管理 |
| `useProfileStore` | 設定プロファイル |

### ストア設計ルール

```typescript
interface StoreState {
  // 状態
  data: DataType | null;
  loading: boolean;
  error: string | null;

  // アクション（状態を変更するメソッド）
  loadData: () => Promise<void>;
  clearError: () => void;
}
```

## 4. SOLID原則の適用

### 単一責任の原則 (SRP)

- 1つのコンポーネントが複数の無関係な責務を持たない
- 1つのストアが複数のドメインを管理しない

```
良い例 - 責務が分離されている:
features/obs/
├── ObsStatusIndicator.tsx  # 状態表示のみ
├── ObsStreamControls.tsx   # 配信制御のみ
├── ObsSceneSelector.tsx    # シーン選択のみ
└── ObsConnectionPanel.tsx  # 接続管理のみ
```

### 開放閉鎖の原則 (OCP)

Propsによる拡張性:

```typescript
interface MetricCardProps {
  title: string;
  icon?: ReactNode;           // 拡張ポイント
  severity?: Severity;        // 拡張ポイント
  children: ReactNode;        // 拡張ポイント
  className?: string;         // 拡張ポイント
}
```

### インターフェース分離の原則 (ISP)

型定義の分割:

```typescript
// 巨大なインターフェースではなく、用途別に分割
interface VideoSettings { ... }
interface AudioSettings { ... }
interface ObsSettings {
  video: VideoSettings;
  audio: AudioSettings;
}
```

### 依存性逆転の原則 (DIP)

Tauri APIの抽象化:

```typescript
// コンポーネントはTauri APIを直接呼び出さない（ストアまたはフック経由）
const { execute } = useTauriCommand<SystemMetrics>('get_system_metrics');
```

## 5. ファイル配置ルール

| 新規追加内容 | 配置先 |
|--------------|--------|
| 新機能コンポーネント | `src/features/<機能名>/` |
| 共有UIコンポーネント | `src/components/` |
| カスタムフック | `src/hooks/` |
| ストア | `src/stores/` |
| 型定義 | `src/types/` |
| ユーティリティ | `src/utils/` |

## 6. 命名規約

| 対象 | 規約 | 例 |
|------|------|-----|
| コンポーネント | `PascalCase.tsx` | `ObsConnectionPanel.tsx` |
| カスタムフック | `useCamelCase.ts` | `useTauriCommand.ts` |
| ストア | `camelCaseStore.ts` | `obsStore.ts` |
| ユーティリティ | `camelCase.ts` | `formatters.ts` |
| 定数 | `UPPER_SNAKE_CASE` | `MAX_RETRY_COUNT` |

## 7. アーキテクチャ影響度評価

### High Impact（要注意）

- `commands.ts` の型変更（全セッション影響）
- ストアの構造変更
- 共通コンポーネントの変更

### Medium Impact

- 機能モジュール全体のリファクタリング
- 新機能の追加

### Low Impact

- 単一コンポーネントの修正
- テストの追加

## 8. レビュー時のチェックリスト

- [ ] 新ファイルは適切な場所に配置されているか
- [ ] `index.ts` でexportが追加されているか
- [ ] Propsインターフェースが明示的に定義されているか
- [ ] ダークモード対応しているか
- [ ] 循環依存が発生していないか
- [ ] 上位層への依存がないか
