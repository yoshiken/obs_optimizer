---
name: frontend-ts
description: |
  OBS Optimizer プロジェクトのフロントエンド開発用スキル。
  このスキルは、TypeScript/React/Zustand ファイル(src/)の編集時にTDD駆動でベストプラクティスに従った実装とセルフコードレビューを自動的に行う。
  フロントエンドの機能実装・改善、新規コンポーネント追加時に使用する。
---

# Frontend TypeScript Development Skill

OBS Optimizer (React 18 + TypeScript 5.x + Zustand) のフロントエンド開発を支援するスキル。

## 開発方針: TDD駆動開発

このスキルは**テスト駆動開発（TDD）**を採用する。すべての機能実装は以下のサイクルで行う:

```
Red → Green → Refactor
```

1. **Red（失敗するテストを書く）**: 実装前に期待する動作をテストとして記述
2. **Green（テストを通す）**: テストをパスする最小限のコードを実装
3. **Refactor（リファクタリング）**: テストが通る状態を維持しながらコードを改善

## 適用条件

以下の条件でこのスキルを使用する:

- `src/` 配下のTypeScript/TSXファイルを編集する場合
- 新しいReactコンポーネントを追加する場合
- Zustandストアを追加・変更する場合
- フロントエンドの機能実装・改善を行う場合

## 開発ワークフロー

### Phase 1: 事前確認

1. **ロック確認**: `.claude-locks/` で担当領域がロックされていないか確認
2. **型定義確認**: `src/types/commands.ts` で必要な型を確認（Tauriコマンド連携時）
3. **依存関係**: 新規パッケージ必要時は `.claude/dependency-requests.md` に記載

### Phase 2: 実装（TDDサイクル）

#### Step 1: テストを先に書く（Red）

```typescript
import { render, screen, fireEvent } from '@testing-library/react';
import { Component } from './Component';

describe('Component', () => {
  const mockStatus = { connected: true, streaming: false };
  const mockOnRefresh = vi.fn();

  it('displays status correctly', () => {
    render(<Component status={mockStatus} onRefresh={mockOnRefresh} />);
    expect(screen.getByText(/connected/i)).toBeInTheDocument();
  });

  it('shows loading state', () => {
    render(<Component status={mockStatus} loading={true} onRefresh={mockOnRefresh} />);
    expect(screen.getByText('読み込み中...')).toBeInTheDocument();
  });

  it('calls onRefresh when button clicked', async () => {
    render(<Component status={mockStatus} onRefresh={mockOnRefresh} />);
    fireEvent.click(screen.getByRole('button', { name: /refresh/i }));
    expect(mockOnRefresh).toHaveBeenCalled();
  });
});
```

#### Step 2: テストを通す実装（Green）

#### コンポーネントの標準パターン

```tsx
import { useState, useCallback } from 'react';
import type { ObsStatus } from '@/types/commands';

interface ComponentProps {
  /** JSDocで必ずプロパティの役割を説明 */
  status: ObsStatus;
  /** オプショナルプロパティにはデフォルト値を設定 */
  loading?: boolean;
  /** コールバックは適切な型定義 */
  onRefresh: () => Promise<void>;
}

export function Component({ status, loading = false, onRefresh }: ComponentProps) {
  const [error, setError] = useState<string | null>(null);

  const handleRefresh = useCallback(async () => {
    try {
      await onRefresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, [onRefresh]);

  if (loading) return <div>読み込み中...</div>;
  if (error) return <div className="text-red-500">{error}</div>;

  return (
    <div className="bg-white dark:bg-gray-800 p-4 rounded-lg">
      {/* コンテンツ */}
    </div>
  );
}
```

#### Step 3: リファクタリング（Refactor）

テストが通ったら、以下を確認しながらコードを改善:
- 重複コードの抽出
- 命名の改善
- useMemo/useCallbackによる最適化
- コンポーネント分割

**リファクタリング中は常にテストを実行して動作を保証する。**

```bash
pnpm test:run
```

#### 絶対禁止事項

- `any` 型の使用
- `as unknown as T` の使用
- `@ts-ignore` / `@ts-expect-error` の使用
- `package.json` / `tauri.conf.json` の直接編集
- `dangerouslySetInnerHTML` の使用

#### 必須事項

- すべての関数に明示的な戻り値の型を定義
- `src/types/commands.ts` から型をインポート
- ダークモード対応（`dark:` プレフィックス）
- エラーハンドリング（try-catch）必須
- 日本語でコメントを記述

### Phase 3: セルフレビュー

実装完了後、以下のチェックを実行:

#### 3.1 静的解析（必須）

```bash
pnpm tsc --noEmit
pnpm lint
pnpm test:run
```

#### 3.2 コードレビューチェックリスト

詳細は `references/review-checklist.md` を参照。主要項目:

**Critical（必須）**:
- `any` / `as unknown as` / `@ts-ignore` を使用していない
- すべてのTauri `invoke` 呼び出しに try-catch がある
- `commands.ts` の型定義と整合性がある
- ダークモード対応している

**High（推奨）**:
- `useMemo` / `useCallback` で最適化されている
- useEffectのクリーンアップが実装されている
- セマンティックHTMLを使用している
- テストが追加されている

#### 3.3 アーキテクチャ確認

詳細は `references/architecture.md` を参照。主要ポイント:

- `features/` 配下に機能単位で配置
- `components/` は共有UIコンポーネントのみ
- ストアは `stores/` に配置し、ドメインごとに分離
- 上位層は下位層をimportできるが、逆は不可

### Phase 4: 完了処理

1. **型定義更新**: 新規Tauriコマンド連携時は `src/types/commands.ts` を確認
2. **ビルド確認**: `pnpm tauri dev` で動作確認

## クイックリファレンス

### Tauri連携

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { SystemMetrics } from '@/types/commands';

// 型付きinvoke
const metrics = await invoke<SystemMetrics>('get_system_metrics');
```

### Zustandストア

```typescript
import { create } from 'zustand';

interface StoreState {
  data: DataType | null;
  loading: boolean;
  error: string | null;
  loadData: () => Promise<void>;
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
    } catch (e) {
      set({ error: e instanceof Error ? e.message : String(e), loading: false });
    }
  },
}));
```

### イベントリスナー

```typescript
import { listen } from '@tauri-apps/api/event';

useEffect(() => {
  const unlisten = listen<PayloadType>('event-name', (event) => {
    // 処理
  });

  return () => {
    void unlisten.then((fn) => fn()); // クリーンアップ必須
  };
}, []);
```

### ダークモード対応

```tsx
// 必ず両モード対応
<div className="bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100">
```

## 参考資料

- `references/review-checklist.md`: 詳細なレビューチェックリスト
- `references/architecture.md`: アーキテクチャガイドライン
- `references/best-practices.md`: React/TypeScriptベストプラクティス集
- `/home/yskn/git/obs_optimizer/CLAUDE.md`: プロジェクト全体のルール
