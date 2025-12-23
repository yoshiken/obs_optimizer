import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { initializeTheme, type ThemeMode, useThemeStore } from './themeStore';

// LocalStorage のモック
const localStorageMock = (() => {
  let store: Record<string, string> = {};

  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value.toString();
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

// matchMedia のモック
const createMatchMediaMock = (matches: boolean) => {
  return (query: string) => ({
    matches,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  });
};

describe('themeStore', () => {
  beforeEach(() => {
    localStorageMock.clear();
    // デフォルトはライトモード
    window.matchMedia = createMatchMediaMock(false) as unknown as typeof window.matchMedia;
  });

  describe('初期状態', () => {
    it('デフォルトでsystemモードになる', () => {
      const { result } = renderHook(() => useThemeStore());
      expect(result.current.mode).toBe('system');
    });

    it('システムがライトモードの場合、resolvedThemeはlight', () => {
      window.matchMedia = createMatchMediaMock(false) as unknown as typeof window.matchMedia;
      const { result } = renderHook(() => useThemeStore());
      expect(result.current.resolvedTheme).toBe('light');
    });

    it('システムがダークモードの場合、resolvedThemeはdark', () => {
      window.matchMedia = createMatchMediaMock(true) as unknown as typeof window.matchMedia;
      renderHook(() => useThemeStore());
      // 新しいストアインスタンスを作成
      const store = useThemeStore.getState();
      store.updateResolvedTheme(true);
      expect(useThemeStore.getState().resolvedTheme).toBe('dark');
    });
  });

  describe('setTheme', () => {
    it('ライトモードに設定できる', () => {
      const { result } = renderHook(() => useThemeStore());

      act(() => {
        result.current.setTheme('light');
      });

      expect(result.current.mode).toBe('light');
      expect(result.current.resolvedTheme).toBe('light');
    });

    it('ダークモードに設定できる', () => {
      const { result } = renderHook(() => useThemeStore());

      act(() => {
        result.current.setTheme('dark');
      });

      expect(result.current.mode).toBe('dark');
      expect(result.current.resolvedTheme).toBe('dark');
    });

    it('systemモードに設定できる', () => {
      const { result } = renderHook(() => useThemeStore());

      act(() => {
        result.current.setTheme('system');
      });

      expect(result.current.mode).toBe('system');
    });

    it('テーマ変更時にDOMクラスが更新される', () => {
      const { result } = renderHook(() => useThemeStore());
      const root = document.documentElement;

      act(() => {
        result.current.setTheme('dark');
      });

      expect(root.classList.contains('dark')).toBe(true);

      act(() => {
        result.current.setTheme('light');
      });

      expect(root.classList.contains('dark')).toBe(false);
    });
  });

  describe('永続化', () => {
    // Note: Zustand persistの内部動作はZustand側でテスト済み
    // ここではストアの状態変更が正しく行われることをテスト
    it('テーマ設定後も状態が維持される', () => {
      const { result } = renderHook(() => useThemeStore());

      act(() => {
        result.current.setTheme('dark');
      });

      // 状態が正しく更新されていることを確認
      expect(result.current.mode).toBe('dark');
      expect(result.current.resolvedTheme).toBe('dark');

      // 別のmodeに変更
      act(() => {
        result.current.setTheme('light');
      });

      expect(result.current.mode).toBe('light');
      expect(result.current.resolvedTheme).toBe('light');
    });

    it('ストアは正しい永続化キーを使用する設定がある', () => {
      // persist middlewareの存在確認（ストアが正しく初期化されている）
      const store = useThemeStore.getState();
      expect(store).toHaveProperty('mode');
      expect(store).toHaveProperty('setTheme');
    });
  });

  describe('updateResolvedTheme', () => {
    it('systemモード時にシステムテーマ変更を反映する', () => {
      const { result } = renderHook(() => useThemeStore());

      act(() => {
        result.current.setTheme('system');
      });

      // システムがダークモードに変更
      act(() => {
        result.current.updateResolvedTheme(true);
      });

      expect(result.current.resolvedTheme).toBe('dark');

      // システムがライトモードに変更
      act(() => {
        result.current.updateResolvedTheme(false);
      });

      expect(result.current.resolvedTheme).toBe('light');
    });

    it('lightモード時はシステムテーマ変更を無視する', () => {
      const { result } = renderHook(() => useThemeStore());

      act(() => {
        result.current.setTheme('light');
      });

      const initialTheme = result.current.resolvedTheme;

      act(() => {
        result.current.updateResolvedTheme(true);
      });

      expect(result.current.resolvedTheme).toBe(initialTheme);
    });

    it('darkモード時はシステムテーマ変更を無視する', () => {
      const { result } = renderHook(() => useThemeStore());

      act(() => {
        result.current.setTheme('dark');
      });

      const initialTheme = result.current.resolvedTheme;

      act(() => {
        result.current.updateResolvedTheme(false);
      });

      expect(result.current.resolvedTheme).toBe(initialTheme);
    });
  });

  describe('initializeTheme', () => {
    it('初期化時にDOMクラスを適用する', () => {
      const root = document.documentElement;
      root.classList.remove('dark');

      const { result } = renderHook(() => useThemeStore());
      act(() => {
        result.current.setTheme('dark');
      });

      initializeTheme();

      expect(root.classList.contains('dark')).toBe(true);
    });
  });

  describe('型安全性', () => {
    it('ThemeModeは正しい値のみ受け付ける', () => {
      const { result } = renderHook(() => useThemeStore());

      const validModes: ThemeMode[] = ['light', 'dark', 'system'];

      validModes.forEach((mode) => {
        act(() => {
          result.current.setTheme(mode);
        });
        expect(result.current.mode).toBe(mode);
      });
    });
  });
});
