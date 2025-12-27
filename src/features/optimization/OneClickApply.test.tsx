import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { OneClickApply } from './OneClickApply';

// Tauriのinvokeをモック
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const { invoke } = await import('@tauri-apps/api/core');

describe('OneClickApply', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('onAppliedコールバックが最適化成功時に呼ばれる', async () => {
    const mockResult = {
      appliedCount: 5,
      failedCount: 0,
      errors: [],
    };

    // モックの設定
    vi.mocked(invoke).mockResolvedValue(mockResult);

    const onApplied = vi.fn();

    render(<OneClickApply onApplied={onApplied} />);

    // 適用ボタンをクリック
    const applyButton = screen.getByText('最適化を適用');
    fireEvent.click(applyButton);

    // 確認ダイアログで適用を確定
    const confirmButton = await screen.findByText('適用');
    fireEvent.click(confirmButton);

    // onAppliedコールバックが呼ばれることを確認
    await waitFor(() => {
      expect(onApplied).toHaveBeenCalledWith(mockResult);
    });
  });

  it('適用失敗時にonErrorコールバックが呼ばれる', async () => {
    const mockResult = {
      appliedCount: 3,
      failedCount: 2,
      errors: ['エラー1', 'エラー2'],
    };

    vi.mocked(invoke).mockResolvedValue(mockResult);

    const onError = vi.fn();

    render(<OneClickApply onError={onError} />);

    const applyButton = screen.getByText('最適化を適用');
    fireEvent.click(applyButton);

    const confirmButton = await screen.findByText('適用');
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(onError).toHaveBeenCalledWith('2件の設定適用に失敗しました');
    });
  });

  it('確認ダイアログでキャンセルすると適用されない', async () => {
    const onApplied = vi.fn();

    render(<OneClickApply onApplied={onApplied} />);

    const applyButton = screen.getByText('最適化を適用');
    fireEvent.click(applyButton);

    // 確認ダイアログが表示される
    expect(screen.getByText('最適化を適用しますか?')).toBeInTheDocument();

    // キャンセルボタンをクリック
    const cancelButton = screen.getByText('キャンセル');
    fireEvent.click(cancelButton);

    // invokeが呼ばれないことを確認
    expect(invoke).not.toHaveBeenCalled();
    expect(onApplied).not.toHaveBeenCalled();
  });
});
