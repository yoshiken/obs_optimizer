import { create } from 'zustand';
import { persist } from 'zustand/middleware';

// ========================================
// テーマ関連の型定義
// ========================================

/** テーマモード */
export type ThemeMode = 'light' | 'dark' | 'system';

/** テーマストアの状態 */
interface ThemeState {
  /** 現在のテーマモード設定 */
  mode: ThemeMode;
  /** 実際に適用されているテーマ（systemの場合はOSの設定を反映） */
  resolvedTheme: 'light' | 'dark';

  // アクション
  /** テーマモードを設定 */
  setTheme: (mode: ThemeMode) => void;
  /** OSのテーマ設定に基づいて解決済みテーマを更新 */
  updateResolvedTheme: (isDark: boolean) => void;
}

// ========================================
// ユーティリティ関数
// ========================================

/** システムのダークモード設定を取得 */
function getSystemTheme(): 'light' | 'dark' {
  if (typeof window === 'undefined') {return 'light';}
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

/** テーマモードに基づいて解決済みテーマを計算 */
function resolveTheme(mode: ThemeMode): 'light' | 'dark' {
  if (mode === 'system') {
    return getSystemTheme();
  }
  return mode;
}

/** DOMにテーマクラスを適用 */
function applyThemeToDOM(theme: 'light' | 'dark') {
  const root = document.documentElement;
  if (theme === 'dark') {
    root.classList.add('dark');
  } else {
    root.classList.remove('dark');
  }
}

// ========================================
// ストア実装
// ========================================

export const useThemeStore = create<ThemeState>()(
  persist(
    (set, get) => ({
      mode: 'system',
      resolvedTheme: getSystemTheme(),

      setTheme: (mode: ThemeMode) => {
        const resolvedTheme = resolveTheme(mode);
        set({ mode, resolvedTheme });
        applyThemeToDOM(resolvedTheme);
      },

      updateResolvedTheme: (isDark: boolean) => {
        const { mode } = get();
        if (mode === 'system') {
          const resolvedTheme = isDark ? 'dark' : 'light';
          set({ resolvedTheme });
          applyThemeToDOM(resolvedTheme);
        }
      },
    }),
    {
      name: 'theme-storage', // localStorageのキー
      partialize: (state) => ({ mode: state.mode }), // modeのみ永続化
    }
  )
);

// ========================================
// 初期化処理
// ========================================

/**
 * テーマストアを初期化
 * - localStorageから設定を読み込み
 * - システムテーマの変更を監視
 */
export function initializeTheme() {
  const store = useThemeStore.getState();

  // 初期テーマを適用
  applyThemeToDOM(store.resolvedTheme);

  // システムテーマの変更を監視
  if (typeof window !== 'undefined') {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

    const handleChange = (e: MediaQueryListEvent) => {
      store.updateResolvedTheme(e.matches);
    };

    // Modern API
    if (mediaQuery.addEventListener) {
      mediaQuery.addEventListener('change', handleChange);
    } else {
      // Legacy API fallback
      mediaQuery.addListener(handleChange);
    }

    // クリーンアップ関数を返す
    return () => {
      if (mediaQuery.removeEventListener) {
        mediaQuery.removeEventListener('change', handleChange);
      } else {
        mediaQuery.removeListener(handleChange);
      }
    };
  }
}
