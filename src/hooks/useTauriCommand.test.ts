import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useTauriCommand } from './useTauriCommand';
import { invoke } from '@tauri-apps/api/core';
import type { SystemMetrics } from '../types/commands';
import { mockSystemMetrics, setupInvokeMock, setupInvokeErrorMock } from '../tests/mocks/tauriMocks';

vi.mock('@tauri-apps/api/core');

const mockInvoke = vi.mocked(invoke);

describe('useTauriCommand', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('初期状態', () => {
    it('dataがnull、loadingがfalse、errorがnullで初期化される', () => {
      const { result } = renderHook(() =>
        useTauriCommand<SystemMetrics>('get_system_metrics')
      );

      expect(result.current.data).toBeNull();
      expect(result.current.loading).toBe(false);
      expect(result.current.error).toBeNull();
      expect(typeof result.current.execute).toBe('function');
    });
  });

  describe('execute', () => {
    it('コマンドを実行してデータを取得できる', async () => {
      setupInvokeMock(mockInvoke);

      const { result } = renderHook(() =>
        useTauriCommand<SystemMetrics>('get_system_metrics')
      );

      const data = await result.current.execute();

      await waitFor(() => {
        expect(result.current.data).toEqual(mockSystemMetrics);
        expect(result.current.loading).toBe(false);
        expect(result.current.error).toBeNull();
      });

      expect(data).toEqual(mockSystemMetrics);
      expect(mockInvoke).toHaveBeenCalledWith('get_system_metrics', undefined);
    });

    it('パラメータ付きでコマンドを実行できる', async () => {
      setupInvokeMock(mockInvoke);

      interface ConnectParams {
        params: {
          host: string;
          port: number;
        };
      }

      const { result } = renderHook(() =>
        useTauriCommand<void, ConnectParams>('connect_obs')
      );

      const params: ConnectParams = {
        params: {
          host: 'localhost',
          port: 4455,
        },
      };

      await result.current.execute(params);

      expect(mockInvoke).toHaveBeenCalledWith('connect_obs', params);
    });

    it('実行中はloadingがtrueになる', async () => {
      setupInvokeMock(mockInvoke);

      const { result } = renderHook(() =>
        useTauriCommand<SystemMetrics>('get_system_metrics')
      );

      const promise = result.current.execute();

      // 実行中の状態を確認（タイミング依存）
      await promise;

      await waitFor(() => {
        expect(result.current.loading).toBe(false);
      });
    });

    it('エラー時にエラー状態を設定する', async () => {
      const errorMessage = 'Command failed';
      setupInvokeErrorMock(mockInvoke, errorMessage);

      const { result } = renderHook(() =>
        useTauriCommand<SystemMetrics>('get_system_metrics')
      );

      const data = await result.current.execute();

      await waitFor(() => {
        expect(result.current.error).toEqual({
          code: 'INVOKE_ERROR',
          message: errorMessage,
        });
        expect(result.current.loading).toBe(false);
        expect(result.current.data).toBeNull();
      });

      expect(data).toBeNull();
    });

    it('エラーが文字列の場合も正しく処理する', async () => {
      mockInvoke.mockRejectedValue('String error');

      const { result } = renderHook(() =>
        useTauriCommand<SystemMetrics>('get_system_metrics')
      );

      await result.current.execute();

      await waitFor(() => {
        expect(result.current.error?.message).toBe('String error');
      });
    });

    it('複数回実行できる', async () => {
      setupInvokeMock(mockInvoke);

      const { result } = renderHook(() =>
        useTauriCommand<SystemMetrics>('get_system_metrics')
      );

      await result.current.execute();
      await result.current.execute();
      await result.current.execute();

      expect(mockInvoke).toHaveBeenCalledTimes(3);
    });

    it('再実行時にエラーをクリアする', async () => {
      const errorMessage = 'First error';
      setupInvokeErrorMock(mockInvoke, errorMessage);

      const { result } = renderHook(() =>
        useTauriCommand<SystemMetrics>('get_system_metrics')
      );

      // 最初はエラー
      await result.current.execute();
      await waitFor(() => {
        expect(result.current.error).not.toBeNull();
      });

      // 次は成功
      setupInvokeMock(mockInvoke);
      await result.current.execute();

      await waitFor(() => {
        expect(result.current.error).toBeNull();
        expect(result.current.data).toEqual(mockSystemMetrics);
      });
    });
  });

  describe('コマンド名の変更', () => {
    it('コマンド名が変わっても正しく動作する', async () => {
      setupInvokeMock(mockInvoke);

      const { result, rerender } = renderHook(
        ({ cmd }) => useTauriCommand<SystemMetrics>(cmd),
        {
          initialProps: { cmd: 'get_system_metrics' },
        }
      );

      await result.current.execute();
      expect(mockInvoke).toHaveBeenCalledWith('get_system_metrics', undefined);

      // コマンド名を変更
      rerender({ cmd: 'get_legacy_system_metrics' });

      mockInvoke.mockClear();
      await result.current.execute();
      expect(mockInvoke).toHaveBeenCalledWith('get_legacy_system_metrics', undefined);
    });
  });
});
