import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, waitFor } from '../tests/utils/test-utils';
import { Toast } from './Toast';
import type { UIAlert } from '../stores/alertStore';
import userEvent from '@testing-library/user-event';

describe('Toast', () => {
  let mockOnDismiss: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.useFakeTimers();
    mockOnDismiss = vi.fn();
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  const mockAlert: UIAlert = {
    id: 'test-alert-1',
    severity: 'info',
    title: 'テストタイトル',
    message: 'テストメッセージ',
    timestamp: Date.now(),
    active: true,
  };

  describe('レンダリング', () => {
    it('タイトルとメッセージを表示する', () => {
      render(<Toast alert={mockAlert} onDismiss={mockOnDismiss} />);

      expect(screen.getByText('テストタイトル')).toBeInTheDocument();
      expect(screen.getByText('テストメッセージ')).toBeInTheDocument();
    });

    it('role="alert"属性を持つ', () => {
      render(<Toast alert={mockAlert} onDismiss={mockOnDismiss} />);

      const alert = screen.getByRole('alert');
      expect(alert).toBeInTheDocument();
    });

    it('閉じるボタンを表示する', () => {
      render(<Toast alert={mockAlert} onDismiss={mockOnDismiss} />);

      const closeButton = screen.getByLabelText('通知を閉じる');
      expect(closeButton).toBeInTheDocument();
    });
  });

  describe('severity別スタイリング', () => {
    it('critical severityを表示できる', () => {
      const criticalAlert: UIAlert = {
        ...mockAlert,
        severity: 'critical',
      };

      render(<Toast alert={criticalAlert} onDismiss={mockOnDismiss} />);

      expect(screen.getByText('テストタイトル')).toBeInTheDocument();
    });

    it('warning severityを表示できる', () => {
      const warningAlert: UIAlert = {
        ...mockAlert,
        severity: 'warning',
      };

      render(<Toast alert={warningAlert} onDismiss={mockOnDismiss} />);

      expect(screen.getByText('テストタイトル')).toBeInTheDocument();
    });

    it('info severityを表示できる', () => {
      const infoAlert: UIAlert = {
        ...mockAlert,
        severity: 'info',
      };

      render(<Toast alert={infoAlert} onDismiss={mockOnDismiss} />);

      expect(screen.getByText('テストタイトル')).toBeInTheDocument();
    });

    it('tips severityを表示できる', () => {
      const tipsAlert: UIAlert = {
        ...mockAlert,
        severity: 'tips',
      };

      render(<Toast alert={tipsAlert} onDismiss={mockOnDismiss} />);

      expect(screen.getByText('テストタイトル')).toBeInTheDocument();
    });
  });

  describe('aria-live属性', () => {
    it('criticalとwarningはaria-live="assertive"を持つ', () => {
      const criticalAlert: UIAlert = {
        ...mockAlert,
        severity: 'critical',
      };

      render(<Toast alert={criticalAlert} onDismiss={mockOnDismiss} />);

      const alert = screen.getByRole('alert');
      expect(alert).toHaveAttribute('aria-live', 'assertive');
    });

    it('infoとtipsはaria-live="polite"を持つ', () => {
      const infoAlert: UIAlert = {
        ...mockAlert,
        severity: 'info',
      };

      render(<Toast alert={infoAlert} onDismiss={mockOnDismiss} />);

      const alert = screen.getByRole('alert');
      expect(alert).toHaveAttribute('aria-live', 'polite');
    });
  });

  describe('手動Dismiss', () => {
    it('閉じるボタンをクリックするとonDismissが呼ばれる', async () => {
      render(<Toast alert={mockAlert} onDismiss={mockOnDismiss} />);

      const closeButton = screen.getByLabelText('通知を閉じる');

      // クリックイベントを発火
      closeButton.click();

      // 全てのタイマーを実行
      await vi.runAllTimersAsync();

      expect(mockOnDismiss).toHaveBeenCalledWith(mockAlert.id);
    });

    it('アニメーション完了後にonDismissが呼ばれる', async () => {
      render(<Toast alert={mockAlert} onDismiss={mockOnDismiss} />);

      const closeButton = screen.getByLabelText('通知を閉じる');
      closeButton.click();

      // アニメーション時間未満
      await vi.advanceTimersByTimeAsync(100);
      expect(mockOnDismiss).not.toHaveBeenCalled();

      // アニメーション時間完了
      await vi.advanceTimersByTimeAsync(200);
      expect(mockOnDismiss).toHaveBeenCalled();
    });
  });

  describe('自動Dismiss', () => {
    it('info severityは5秒後に自動的にdismissされる', async () => {
      const infoAlert: UIAlert = {
        ...mockAlert,
        severity: 'info',
      };

      render(<Toast alert={infoAlert} onDismiss={mockOnDismiss} />);

      // 4秒後 - まだdismissされていない
      await vi.advanceTimersByTimeAsync(4000);
      expect(mockOnDismiss).not.toHaveBeenCalled();

      // 5秒後 + アニメーション時間
      await vi.advanceTimersByTimeAsync(1000 + 300);
      expect(mockOnDismiss).toHaveBeenCalledWith(infoAlert.id);
    });

    it('tips severityは5秒後に自動的にdismissされる', async () => {
      const tipsAlert: UIAlert = {
        ...mockAlert,
        severity: 'tips',
      };

      render(<Toast alert={tipsAlert} onDismiss={mockOnDismiss} />);

      await vi.advanceTimersByTimeAsync(5000 + 300);
      expect(mockOnDismiss).toHaveBeenCalledWith(tipsAlert.id);
    });

    it('critical severityは自動的にdismissされない', async () => {
      const criticalAlert: UIAlert = {
        ...mockAlert,
        severity: 'critical',
      };

      render(<Toast alert={criticalAlert} onDismiss={mockOnDismiss} />);

      // 10秒経過してもdismissされない
      await vi.advanceTimersByTimeAsync(10000);
      expect(mockOnDismiss).not.toHaveBeenCalled();
    });

    it('warning severityは自動的にdismissされない', async () => {
      const warningAlert: UIAlert = {
        ...mockAlert,
        severity: 'warning',
      };

      render(<Toast alert={warningAlert} onDismiss={mockOnDismiss} />);

      // 10秒経過してもdismissされない
      await vi.advanceTimersByTimeAsync(10000);
      expect(mockOnDismiss).not.toHaveBeenCalled();
    });
  });

  describe('アニメーション', () => {
    it('マウント時にスライドインアニメーションを開始する', () => {
      render(<Toast alert={mockAlert} onDismiss={mockOnDismiss} />);

      // アニメーション開始前の確認は難しいため、レンダリングされることを確認
      expect(screen.getByText('テストタイトル')).toBeInTheDocument();
    });
  });

  describe('アクセシビリティ', () => {
    it('aria-atomic="true"を持つ', () => {
      render(<Toast alert={mockAlert} onDismiss={mockOnDismiss} />);

      const alert = screen.getByRole('alert');
      expect(alert).toHaveAttribute('aria-atomic', 'true');
    });

    it('閉じるボタンにaria-labelがある', () => {
      render(<Toast alert={mockAlert} onDismiss={mockOnDismiss} />);

      const closeButton = screen.getByLabelText('通知を閉じる');
      expect(closeButton).toBeInTheDocument();
    });

    it('アイコンがaria-hidden="true"を持つ', () => {
      const { container } = render(<Toast alert={mockAlert} onDismiss={mockOnDismiss} />);

      const icons = container.querySelectorAll('[aria-hidden="true"]');
      expect(icons.length).toBeGreaterThan(0);
    });
  });

  describe('クリーンアップ', () => {
    it('アンマウント時にタイマーをクリアする', async () => {
      const { unmount } = render(<Toast alert={mockAlert} onDismiss={mockOnDismiss} />);

      unmount();

      // タイマーが適切にクリアされていれば、アンマウント後に進めても呼ばれない
      await vi.advanceTimersByTimeAsync(10000);
      expect(mockOnDismiss).not.toHaveBeenCalled();
    });
  });
});
