import { type ThemeMode, useThemeStore } from '../stores/themeStore';

/**
 * テーマ切り替えボタンコンポーネント
 *
 * 機能:
 * - ライト/ダーク/システムの3つのモードを切り替え
 * - 現在のテーマに応じたアイコン表示
 * - キーボードナビゲーション対応
 *
 * 使用例:
 * ```tsx
 * <ThemeToggle />
 * ```
 */
export function ThemeToggle() {
  const { mode, setTheme } = useThemeStore();

  /**
   * 次のテーマに切り替え
   * light -> dark -> system -> light
   */
  const cycleTheme = () => {
    const nextTheme: Record<ThemeMode, ThemeMode> = {
      light: 'dark',
      dark: 'system',
      system: 'light',
    };
    setTheme(nextTheme[mode]);
  };

  /**
   * テーマに応じたアイコンを取得
   */
  const getThemeIcon = () => {
    switch (mode) {
      case 'light':
        // 太陽アイコン
        return (
          <svg
            className="w-5 h-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            aria-hidden="true"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
            />
          </svg>
        );
      case 'dark':
        // 月アイコン
        return (
          <svg
            className="w-5 h-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            aria-hidden="true"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
            />
          </svg>
        );
      case 'system':
        // デスクトップアイコン
        return (
          <svg
            className="w-5 h-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            aria-hidden="true"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
            />
          </svg>
        );
    }
  };

  /**
   * テーマモードのラベルを取得
   */
  const getThemeLabel = () => {
    switch (mode) {
      case 'light':
        return 'ライトモード';
      case 'dark':
        return 'ダークモード';
      case 'system':
        return 'システム設定に従う';
    }
  };

  return (
    <button
      onClick={cycleTheme}
      className="
        p-2 rounded-md
        text-gray-600 dark:text-gray-300
        hover:bg-gray-100 dark:hover:bg-gray-700
        focus:outline-none focus:ring-2 focus:ring-blue-500
        transition-colors
      "
      aria-label={`テーマ切り替え (現在: ${getThemeLabel()})`}
      title={getThemeLabel()}
    >
      {getThemeIcon()}
    </button>
  );
}
