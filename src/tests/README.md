# フロントエンドテストガイド

このディレクトリには、OBS配信最適化ツールのフロントエンドテストが含まれています。

## テスト構成

### テストランナー
- **Vitest** - 高速なViteネイティブのテストランナー
- **React Testing Library** - React コンポーネントのテスト
- **@testing-library/jest-dom** - DOM検証用のカスタムマッチャー
- **jsdom** - ブラウザ環境のシミュレーション

### ディレクトリ構造

```
src/tests/
├── README.md              # このファイル
├── setup.ts               # グローバルテストセットアップ
├── mocks/
│   └── tauriMocks.ts     # Tauri APIのモック
└── utils/
    └── test-utils.tsx    # カスタムレンダー関数

src/
├── stores/
│   ├── obsStore.test.ts          # OBSストアのテスト
│   └── metricsStore.test.ts      # メトリクスストアのテスト
├── hooks/
│   └── useTauriCommand.test.ts   # カスタムフックのテスト
└── features/
    ├── obs/
    │   └── ObsStatusIndicator.test.tsx  # OBSステータスコンポーネントのテスト
    └── monitor/
        ├── components/
        │   └── MetricCard.test.tsx      # メトリックカードのテスト
        └── utils/
            └── formatters.test.ts       # フォーマッター関数のテスト
```

## テストの実行

### 基本的なコマンド

```bash
# ウォッチモードでテストを実行（開発中）
pnpm test

# すべてのテストを1回実行
pnpm test:run

# UIモードでテストを実行（ブラウザで結果を確認）
pnpm test:ui

# カバレッジレポートを生成
pnpm test:coverage
```

### 特定のテストファイルを実行

```bash
# ファイル名でフィルタリング
pnpm test obsStore

# パターンマッチング
pnpm test "stores/*.test.ts"
```

### テストのデバッグ

```bash
# --inspect-brkフラグを使用してデバッガーを起動
node --inspect-brk ./node_modules/vitest/vitest.mjs run
```

## テストの書き方

### 1. Zustandストアのテスト

Zustandストアは、モックなしで直接テストできます。

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { useObsStore } from './obsStore';

describe('obsStore', () => {
  beforeEach(() => {
    // 各テスト前にストアをリセット
    useObsStore.setState({
      connectionState: 'disconnected',
      status: null,
      // ...
    });
  });

  it('初期状態をテストする', () => {
    const state = useObsStore.getState();
    expect(state.connectionState).toBe('disconnected');
  });

  it('アクションをテストする', async () => {
    const { connect } = useObsStore.getState();
    await connect({ host: 'localhost', port: 4455 });

    const state = useObsStore.getState();
    expect(state.connectionState).toBe('connected');
  });
});
```

### 2. Reactコンポーネントのテスト

React Testing Libraryを使用してコンポーネントをテストします。

```typescript
import { describe, it, expect } from 'vitest';
import { render, screen } from '../../tests/utils/test-utils';
import { MyComponent } from './MyComponent';

describe('MyComponent', () => {
  it('テキストを表示する', () => {
    render(<MyComponent text="Hello" />);
    expect(screen.getByText('Hello')).toBeInTheDocument();
  });

  it('ボタンクリックを処理する', async () => {
    const { user } = render(<MyComponent />);
    const button = screen.getByRole('button');

    await user.click(button);

    expect(screen.getByText('Clicked')).toBeInTheDocument();
  });
});
```

### 3. カスタムフックのテスト

renderHookを使用してフックをテストします。

```typescript
import { renderHook, waitFor } from '@testing-library/react';
import { useMyHook } from './useMyHook';

it('フックの動作をテストする', async () => {
  const { result } = renderHook(() => useMyHook());

  expect(result.current.loading).toBe(false);

  await result.current.fetchData();

  await waitFor(() => {
    expect(result.current.data).not.toBeNull();
  });
});
```

### 4. Tauri APIのモック

Tauri APIはモック化されています。`src/tests/mocks/tauriMocks.ts`を参照してください。

```typescript
import { vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { setupInvokeMock } from '../tests/mocks/tauriMocks';

vi.mock('@tauri-apps/api/core');
const mockInvoke = vi.mocked(invoke);

it('Tauriコマンドを呼び出す', async () => {
  setupInvokeMock(mockInvoke);

  const result = await invoke('get_obs_status');

  expect(result).toEqual(mockObsStatus);
});
```

## ベストプラクティス

### DO ✓

- **ユーザーの視点からテストする** - `getByRole`, `getByLabelText`などを使用
- **非同期処理には`waitFor`を使用** - 状態更新を待つ
- **各テスト前にストアをリセット** - テスト間の独立性を保つ
- **意味のあるテスト名** - 日本語で明確に記述
- **AAA パターン** - Arrange, Act, Assert

### DON'T ✗

- **実装の詳細をテストしない** - 内部の状態やプライベートメソッドは避ける
- **スナップショットテストを過度に使用しない** - 特定の値を検証する
- **テスト間で状態を共有しない** - 各テストは独立して実行可能にする
- **console.log でデバッグしない** - デバッガーを使用する

## カバレッジ目標

- **ストア**: 90%以上
- **コンポーネント**: 80%以上
- **ユーティリティ関数**: 95%以上
- **カスタムフック**: 85%以上

現在のカバレッジを確認:

```bash
pnpm test:coverage
```

## トラブルシューティング

### act() 警告が出る

React 19では`act()`の扱いが変わりました。`waitFor`を使用して非同期更新を待ちます。

```typescript
await waitFor(() => {
  expect(result.current.loading).toBe(false);
});
```

### Tauri APIのモックが動作しない

`vi.mock()`がファイルの先頭にあることを確認してください。

```typescript
import { vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core'); // ← インポートの直後

const mockInvoke = vi.mocked(invoke);
```

### タイマーのテスト

`vi.useFakeTimers()`を使用してタイマーを制御します。

```typescript
import { vi, beforeEach, afterEach } from 'vitest';

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

it('タイマーをテストする', () => {
  const callback = vi.fn();
  setTimeout(callback, 1000);

  vi.advanceTimersByTime(1000);

  expect(callback).toHaveBeenCalled();
});
```

## 参考リンク

- [Vitest Documentation](https://vitest.dev/)
- [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/)
- [Testing Library Queries](https://testing-library.com/docs/queries/about)
- [jest-dom Matchers](https://github.com/testing-library/jest-dom)
