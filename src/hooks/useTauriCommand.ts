import { invoke, type InvokeArgs } from '@tauri-apps/api/core';
import { useState, useCallback } from 'react';
import type { AppError } from '../types';

/** Tauriコマンドのパラメータ型制約 */
type TauriParams = InvokeArgs | undefined;

interface UseTauriCommandResult<T, P extends TauriParams = undefined> {
  data: T | null;
  error: AppError | null;
  loading: boolean;
  execute: P extends undefined ? () => Promise<T | null> : (params: P) => Promise<T | null>;
}

/**
 * Tauriコマンドを呼び出すためのフック
 *
 * @typeParam T - コマンドの戻り値の型
 * @typeParam P - コマンドのパラメータ型（InvokeArgs を拡張する必要がある）
 * @param command コマンド名
 * @returns data, error, loading, execute
 *
 * @example
 * ```tsx
 * // パラメータなしの場合
 * const { data, execute } = useTauriCommand<SystemMetrics>('get_system_metrics');
 *
 * // パラメータありの場合
 * const { execute } = useTauriCommand<void, { params: ConnectionParams }>('connect_obs');
 * await execute({ params: { host: 'localhost', port: 4455 } });
 * ```
 */
export function useTauriCommand<T, P extends TauriParams = undefined>(
  command: string
): UseTauriCommandResult<T, P> {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<AppError | null>(null);
  const [loading, setLoading] = useState(false);

  const execute = useCallback(async (params?: P): Promise<T | null> => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<T>(command, params);
      setData(result);
      return result;
    } catch (e) {
      const appError: AppError = {
        code: 'INVOKE_ERROR',
        message: e instanceof Error ? e.message : String(e),
      };
      setError(appError);
      return null;
    } finally {
      setLoading(false);
    }
  }, [command]);

  return { data, error, loading, execute } as UseTauriCommandResult<T, P>;
}
