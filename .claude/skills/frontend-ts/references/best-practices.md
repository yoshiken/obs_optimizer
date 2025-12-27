# フロントエンド ベストプラクティス集

## 1. コンポーネント設計

### Propsの型定義

```typescript
interface ComponentProps {
  /** JSDocで必ずプロパティの役割を説明 */
  title: string;
  /** オプショナルプロパティにはデフォルト値を設定 */
  loading?: boolean;
  /** コールバックは適切な型定義 */
  onSubmit: (data: FormData) => Promise<void>;
}

export function Component({ title, loading = false, onSubmit }: ComponentProps) {
  // 実装
}
```

### 条件付きレンダリング

```tsx
// 良い: 早期リターンで不要なレンダリングを防ぐ
if (loading) return <LoadingSpinner />;
if (error) return <ErrorMessage error={error} />;
return <Content data={data} />;

// 避ける: 三項演算子のネスト
return loading ? <Spinner /> : error ? <Error /> : <Content />;
```

## 2. Tauri連携

### コマンド呼び出しの標準パターン

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { SystemMetrics } from '@/types/commands';

async function fetchMetrics(): Promise<void> {
  try {
    const metrics = await invoke<SystemMetrics>('get_system_metrics');
    setMetrics(metrics);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    setError(message);
    console.error('メトリクス取得エラー:', error);
  }
}
```

### イベントリスナー

```typescript
import { listen } from '@tauri-apps/api/event';
import { OBS_EVENTS, type ConnectionChangedPayload } from '@/types/commands';

useEffect(() => {
  const unlisten = listen<ConnectionChangedPayload>(
    OBS_EVENTS.CONNECTION_CHANGED,
    (event) => {
      console.log('接続状態変更:', event.payload);
    }
  );

  return () => {
    void unlisten.then((fn) => fn()); // クリーンアップ必須
  };
}, []);
```

## 3. Zustandストア

### 基本パターン

```typescript
import { create } from 'zustand';

interface StoreState {
  data: DataType | null;
  loading: boolean;
  error: string | null;
  loadData: () => Promise<void>;
  reset: () => void;
}

export const useStore = create<StoreState>((set) => ({
  data: null,
  loading: false,
  error: null,

  loadData: async () => {
    set({ loading: true, error: null });
    try {
      const result = await invoke<DataType>('command');
      set({ data: result, loading: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : String(error),
        loading: false
      });
    }
  },

  reset: () => set({ data: null, loading: false, error: null }),
}));
```

### セレクタで最適化

```typescript
// 良い: 必要な値だけを取得
const { data, loading } = useStore();

// さらに良い: セレクタで最適化
const dataOnly = useStore((state) => state.data);
```

### 永続化

```typescript
import { persist } from 'zustand/middleware';

export const useThemeStore = create<ThemeState>()(
  persist(
    (set) => ({
      // ストア定義
    }),
    {
      name: 'theme-storage',
      partialize: (state) => ({ mode: state.mode }),
    }
  )
);
```

## 4. パフォーマンス最適化

### useMemo / useCallback

```typescript
// 重い計算はメモ化
const processedData = useMemo(() => {
  return heavyComputation(rawData);
}, [rawData]);

// コールバックの再生成を防ぐ
const handleSubmit = useCallback(async (data: FormData) => {
  await invoke('submit', { data });
}, []);
```

### ポーリング

```typescript
const POLLING_INTERVAL = 1000; // 1秒以上推奨

useEffect(() => {
  const intervalId = setInterval(() => {
    void execute();
  }, POLLING_INTERVAL);

  return () => clearInterval(intervalId);
}, [execute]);
```

### デバウンス

```typescript
import { useDebouncedCallback } from 'use-debounce';

const debouncedSave = useDebouncedCallback(
  (value: string) => {
    invoke('save_settings', { value });
  },
  500
);
```

## 5. スタイリング

### ダークモード対応

```tsx
// 必ず両モード対応
<div className="bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100">
  {/* コンテンツ */}
</div>
```

### レスポンシブデザイン

```tsx
// モバイルファースト
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
  {/* sm: 640px, md: 768px, lg: 1024px, xl: 1280px */}
</div>
```

### 動的な値

```tsx
// 動的な値: インラインスタイル
<div style={{ borderLeftColor: getSeverityColor(severity) }}>

// 静的なスタイル: Tailwind
<div className="rounded-lg shadow-md p-4">
```

## 6. 型ガード

```typescript
function isError(value: unknown): value is Error {
  return value instanceof Error;
}

try {
  // ...
} catch (error) {
  if (isError(error)) {
    setError(error.message);
  } else {
    setError(String(error));
  }
}
```

## 7. よくある間違いと回避方法

### イベントリスナーのクリーンアップ忘れ

```typescript
// NG
useEffect(() => {
  listen('event', handler); // メモリリーク
}, []);

// OK
useEffect(() => {
  const unlisten = listen('event', handler);
  return () => {
    void unlisten.then((fn) => fn());
  };
}, []);
```

### ストア全体の取得

```typescript
// NG: 不要な再レンダリング
const store = useObsStore();
return <div>{store.status.connected}</div>;

// OK
const connected = useObsStore((state) => state.status.connected);
```

### ダークモード対応忘れ

```tsx
// NG
<div className="bg-white text-gray-900">

// OK
<div className="bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100">
```

### divでボタン実装

```tsx
// NG
<div onClick={handleClick}>クリック</div>

// OK
<button onClick={handleClick}>クリック</button>
```

## 8. テストパターン

```typescript
import { render, screen } from '@testing-library/react';
import { DashboardPanel } from './DashboardPanel';

test('displays connection status', () => {
  render(<DashboardPanel connected={true} />);
  expect(screen.getByText('接続済み')).toBeInTheDocument();
});
```

### Tauriモック

```typescript
jest.mock('@tauri-apps/api/core', () => ({
  invoke: jest.fn(),
}));
```

## 9. コメント（日本語）

```typescript
// 複雑なロジックには日本語コメント
// GPU世代×グレードによる統合ティアを判定
const tier = calculateGpuTier(gpuInfo);

// TODOコメントに担当セッション名を記載
// TODO(SESSION_OBS): WebSocket再接続ロジックの実装
```

## 10. ビルドコマンド

```bash
# 開発サーバー起動
pnpm tauri dev

# ビルド
pnpm tauri build

# 型チェック
pnpm tsc --noEmit

# Lint
pnpm lint
pnpm lint --fix

# テスト
pnpm test           # Watch モード
pnpm test:run       # 一度だけ実行
pnpm test:coverage  # カバレッジ付き
```
