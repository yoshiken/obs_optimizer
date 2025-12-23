import { create } from 'zustand';
import type { StreamPlatform, StreamStyle } from '../types/commands';

// ========================================
// ユーザー設定の型定義
// ========================================

export interface UserPreferences {
  streamStyle: StreamStyle | null;
  platform: StreamPlatform | null;
}

// ========================================
// ストア状態の型定義
// ========================================

interface OnboardingState {
  /** 現在のステップ（1-7） */
  currentStep: number;
  /** オンボーディング完了フラグ */
  completed: boolean;
  /** ユーザーの配信設定 */
  userPreferences: UserPreferences;

  // アクション
  setStep: (step: number) => void;
  nextStep: () => void;
  prevStep: () => void;
  setUserPreferences: (prefs: Partial<UserPreferences>) => void;
  completeOnboarding: () => void;
  resetOnboarding: () => void;
}

// ========================================
// 定数
// ========================================

/** 総ステップ数 */
const TOTAL_STEPS = 7;

/** 必須ステップ（スキップ不可） */
const REQUIRED_STEPS = [2, 5]; // Step 2: OBS接続、Step 5: 環境分析

// ========================================
// ストア実装
// ========================================

export const useOnboardingStore = create<OnboardingState>((set, get) => ({
  currentStep: 1,
  completed: false,
  userPreferences: {
    streamStyle: null,
    platform: null,
  },

  setStep: (step) => {
    if (step >= 1 && step <= TOTAL_STEPS) {
      set({ currentStep: step });
    }
  },

  nextStep: () => {
    const { currentStep } = get();
    if (currentStep < TOTAL_STEPS) {
      set({ currentStep: currentStep + 1 });
    }
  },

  prevStep: () => {
    const { currentStep } = get();
    if (currentStep > 1) {
      set({ currentStep: currentStep - 1 });
    }
  },

  setUserPreferences: (prefs) => {
    set((state) => ({
      userPreferences: {
        ...state.userPreferences,
        ...prefs,
      },
    }));
  },

  completeOnboarding: () => {
    set({ completed: true, currentStep: TOTAL_STEPS });
  },

  resetOnboarding: () => {
    set({
      currentStep: 1,
      completed: false,
      userPreferences: {
        streamStyle: null,
        platform: null,
      },
    });
  },
}));

// ========================================
// エクスポート
// ========================================

export { TOTAL_STEPS, REQUIRED_STEPS };
