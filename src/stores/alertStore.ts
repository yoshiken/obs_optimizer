import { create } from 'zustand';
import type { AlertSeverity } from '../types/commands';

// ========================================
// アラートの型定義（フロントエンド用拡張）
// ========================================

export interface UIAlert {
  id: string;
  severity: AlertSeverity;
  title: string;
  message: string;
  timestamp: number;
  dismissed: boolean;
}

// 再エクスポート
export type { AlertSeverity };

// ========================================
// ストア状態の型定義
// ========================================

interface AlertState {
  /** アラート一覧（新しい順） */
  alerts: UIAlert[];
  /** ストリーミングモード（配信中は通知を抑制） */
  streamingMode: boolean;

  // アクション
  addAlert: (alert: Omit<UIAlert, 'id' | 'timestamp' | 'dismissed'>) => void;
  dismissAlert: (id: string) => void;
  clearAll: () => void;
  setStreamingMode: (enabled: boolean) => void;
  getActiveAlerts: () => UIAlert[];
  getAlertsByPriority: () => UIAlert[];
}

// ========================================
// ヘルパー関数
// ========================================

/** ユニークなIDを生成 */
function generateId(): string {
  return `alert-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
}

/** 重要度の優先順位を数値化 */
function getSeverityPriority(severity: AlertSeverity): number {
  const priorities: Record<AlertSeverity, number> = {
    critical: 0,
    warning: 1,
    info: 2,
    tips: 3,
  };
  return priorities[severity];
}

// ========================================
// ストア実装
// ========================================

export const useAlertStore = create<AlertState>((set, get) => ({
  alerts: [],
  streamingMode: false,

  addAlert: (alert) => {
    const newAlert: UIAlert = {
      ...alert,
      id: generateId(),
      timestamp: Date.now(),
      dismissed: false,
    };

    set((state) => ({
      alerts: [newAlert, ...state.alerts],
    }));
  },

  dismissAlert: (id) => {
    set((state) => ({
      alerts: state.alerts.map((alert) =>
        alert.id === id ? { ...alert, dismissed: true } : alert
      ),
    }));
  },

  clearAll: () => {
    set((state) => ({
      alerts: state.alerts.map((alert) => ({ ...alert, dismissed: true })),
    }));
  },

  setStreamingMode: (enabled) => {
    set({ streamingMode: enabled });
  },

  /** 未dismissのアラートのみ取得 */
  getActiveAlerts: () => {
    return get().alerts.filter((alert) => !alert.dismissed);
  },

  /** 重要度順（critical > warning > info > tips）でソート */
  getAlertsByPriority: () => {
    return [...get().alerts].sort(
      (a, b) => getSeverityPriority(a.severity) - getSeverityPriority(b.severity)
    );
  },
}));
