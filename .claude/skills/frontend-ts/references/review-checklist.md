# フロントエンド セルフレビューチェックリスト

## 1. 型安全性【最優先】

### 必須確認項目

- [ ] **`any` 型を使用していない**
  ```typescript
  // NG
  const handleData = (data: any) => { ... }

  // OK
  interface DataShape { value: string; count: number }
  const handleData = (data: DataShape) => { ... }
  ```

- [ ] **型アサーション（`as unknown as T`）を使用していない**
  ```typescript
  // NG
  const result = response as unknown as ResponseType;

  // OK: 型ガードを使用
  function isResponseType(obj: unknown): obj is ResponseType {
    return typeof obj === 'object' && obj !== null && 'field' in obj;
  }
  ```

- [ ] **`@ts-ignore` / `@ts-expect-error` を使用していない**

- [ ] **Tauri `invoke` でジェネリック型を指定**
  ```typescript
  // NG
  const status = await invoke('get_obs_status');

  // OK
  const status = await invoke<ObsStatus>('get_obs_status');
  ```

- [ ] **`src/types/commands.ts` の型定義と整合性が取れている**

## 2. Reactコンポーネント

- [ ] **コンポーネント名が PascalCase**

- [ ] **カスタムフックが `use` プレフィックス + camelCase**

- [ ] **propsに明示的な型定義**
  ```typescript
  interface DashboardProps {
    status: ObsStatus;
    onRefresh: () => void;
  }
  ```

- [ ] **useEffectの依存配列が正確**
  ```typescript
  useEffect(() => {
    fetchData(userId);
  }, [userId]); // 依存配列に漏れがないか
  ```

- [ ] **useEffectのクリーンアップ実装**
  ```typescript
  useEffect(() => {
    const interval = setInterval(() => execute(), 1000);
    return () => clearInterval(interval); // クリーンアップ必須
  }, []);
  ```

## 3. Zustand状態管理

- [ ] **ストアの型定義が明示的**

- [ ] **非同期アクションでエラーハンドリング実装**
  ```typescript
  loadData: async () => {
    set({ loading: true, error: null });
    try {
      const result = await invoke<DataType>('command');
      set({ data: result, loading: false });
    } catch (e) {
      set({ error: e instanceof Error ? e.message : String(e), loading: false });
    }
  }
  ```

- [ ] **セレクタで必要な状態のみ取得**
  ```typescript
  // NG: 全状態を取得（不要な再レンダリング）
  const state = useObsStore();

  // OK: 必要な状態のみ
  const connected = useObsStore(state => state.connected);
  ```

## 4. エラーハンドリング

- [ ] **すべてのTauri `invoke` 呼び出しに try-catch**

- [ ] **ユーザーに分かりやすいエラーメッセージ表示**

- [ ] **エラー状態の適切なUI表示**

## 5. スタイリング

- [ ] **ダークモード対応（`dark:` プレフィックス）**
  ```tsx
  // NG
  <div className="bg-white text-gray-900">

  // OK
  <div className="bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100">
  ```

- [ ] **レスポンシブデザイン（モバイルファースト）**
  ```tsx
  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
  ```

- [ ] **Tailwind CSSを使用（インラインスタイルは動的値のみ）**

## 6. パフォーマンス

- [ ] **不要な再レンダリング防止（`useMemo`, `useCallback`）**
  ```typescript
  const expensiveValue = useMemo(() =>
    computeExpensiveValue(data),
    [data]
  );
  ```

- [ ] **大きなリストで仮想化を検討**

- [ ] **ポーリング間隔は1秒以上**

- [ ] **並列可能な処理は `Promise.all` を使用**

## 7. アクセシビリティ

- [ ] **セマンティックHTML使用**
  ```tsx
  // NG
  <div onClick={handleClick}>クリック</div>

  // OK
  <button onClick={handleClick}>クリック</button>
  ```

- [ ] **適切なARIA属性**
  ```tsx
  <button
    onClick={toggle}
    aria-expanded={isOpen}
    aria-controls="content-id"
  >
  ```

- [ ] **キーボード操作可能**

## 8. セキュリティ

- [ ] **`dangerouslySetInnerHTML` を使用していない**

- [ ] **機密情報をコンソールログに出力していない**

- [ ] **ユーザー入力のバリデーション**

## 9. 静的解析の実行

```bash
# 型チェック
pnpm tsc --noEmit

# Lint
pnpm lint

# テスト
pnpm test:run
```

## 優先度別チェックリスト

### Critical（必須修正）

1. `any` / `as unknown as` / `@ts-ignore` の使用
2. エラーハンドリングの欠如
3. `commands.ts` との型不整合
4. ダークモード非対応
5. `dangerouslySetInnerHTML` の使用

### High（修正推奨）

1. useEffectのクリーンアップ漏れ
2. 不要な再レンダリング
3. アクセシビリティ問題
4. テストカバレッジ不足
5. Lint警告の放置

### Medium（改善検討）

1. コードの重複
2. 長すぎるコンポーネント
3. マジックナンバー
4. ドキュメント不足
5. 命名の不適切さ
