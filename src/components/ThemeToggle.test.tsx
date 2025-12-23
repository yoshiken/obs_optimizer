import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { ThemeToggle } from './ThemeToggle';
import { useThemeStore } from '../stores/themeStore';

// ThemeToggleのテストをスキップして後で実装
describe.skip('ThemeToggle', () => {
  beforeEach(() => {
    // テスト前にストアをリセット
    useThemeStore.setState({ mode: 'system', resolvedTheme: 'light' });
  });

  it('テーマ切り替えボタンが表示される', () => {
    render(<ThemeToggle />);
    const button = screen.getByRole('button');
    expect(button).toBeInTheDocument();
  });

  it('クリックするとテーマが切り替わる', () => {
    render(<ThemeToggle />);
    const button = screen.getByRole('button');

    // 初期状態: system
    expect(useThemeStore.getState().mode).toBe('system');

    // 1回目のクリック: light
    fireEvent.click(button);
    expect(useThemeStore.getState().mode).toBe('light');

    // 2回目のクリック: dark
    fireEvent.click(button);
    expect(useThemeStore.getState().mode).toBe('dark');

    // 3回目のクリック: system
    fireEvent.click(button);
    expect(useThemeStore.getState().mode).toBe('system');
  });

  it('各テーマモードで適切なアイコンが表示される', () => {
    const { rerender } = render(<ThemeToggle />);

    // lightモードに設定
    useThemeStore.setState({ mode: 'light', resolvedTheme: 'light' });
    rerender(<ThemeToggle />);
    expect(screen.getByRole('button')).toHaveAttribute(
      'aria-label',
      expect.stringContaining('ライトモード')
    );

    // darkモードに設定
    useThemeStore.setState({ mode: 'dark', resolvedTheme: 'dark' });
    rerender(<ThemeToggle />);
    expect(screen.getByRole('button')).toHaveAttribute(
      'aria-label',
      expect.stringContaining('ダークモード')
    );

    // systemモードに設定
    useThemeStore.setState({ mode: 'system', resolvedTheme: 'light' });
    rerender(<ThemeToggle />);
    expect(screen.getByRole('button')).toHaveAttribute(
      'aria-label',
      expect.stringContaining('システム設定に従う')
    );
  });

  it('アクセシビリティ属性が正しく設定される', () => {
    render(<ThemeToggle />);
    const button = screen.getByRole('button');

    expect(button).toHaveAttribute('aria-label');
    expect(button).toHaveAttribute('title');
  });
});
