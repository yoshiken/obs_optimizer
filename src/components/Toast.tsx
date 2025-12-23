import { useEffect, useState } from 'react';
import type { AlertSeverity, UIAlert } from '../stores/alertStore';

// ========================================
// 型定義
// ========================================

interface ToastProps {
  alert: UIAlert;
  onDismiss: (id: string) => void;
}

// ========================================
// 定数
// ========================================

/** 自動消去タイマー（ミリ秒） */
const AUTO_DISMISS_TIMEOUT: Record<AlertSeverity, number | null> = {
  critical: null, // 手動dismissのみ
  warning: null, // 手動dismissのみ
  info: 5000, // 5秒
  tips: 5000, // 5秒
};

// ========================================
// スタイリング設定
// ========================================

/** レベル別の背景・テキスト色 */
function getSeverityStyles(severity: AlertSeverity): {
  bgColor: string;
  borderColor: string;
  iconColor: string;
  textColor: string;
} {
  switch (severity) {
    case 'critical':
      return {
        bgColor: 'bg-red-50',
        borderColor: 'border-red-300',
        iconColor: 'text-red-600',
        textColor: 'text-red-800',
      };
    case 'warning':
      return {
        bgColor: 'bg-yellow-50',
        borderColor: 'border-yellow-300',
        iconColor: 'text-yellow-600',
        textColor: 'text-yellow-800',
      };
    case 'info':
      return {
        bgColor: 'bg-blue-50',
        borderColor: 'border-blue-300',
        iconColor: 'text-blue-600',
        textColor: 'text-blue-800',
      };
    case 'tips':
      return {
        bgColor: 'bg-gray-50',
        borderColor: 'border-gray-300',
        iconColor: 'text-gray-600',
        textColor: 'text-gray-800',
      };
  }
}

/** レベル別のアイコン */
function SeverityIcon({ severity }: { severity: AlertSeverity }) {
  const { iconColor } = getSeverityStyles(severity);

  switch (severity) {
    case 'critical':
      return (
        <svg
          className={`w-5 h-5 ${iconColor}`}
          fill="currentColor"
          viewBox="0 0 20 20"
          aria-hidden="true"
        >
          <path
            fillRule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
            clipRule="evenodd"
          />
        </svg>
      );
    case 'warning':
      return (
        <svg
          className={`w-5 h-5 ${iconColor}`}
          fill="currentColor"
          viewBox="0 0 20 20"
          aria-hidden="true"
        >
          <path
            fillRule="evenodd"
            d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
            clipRule="evenodd"
          />
        </svg>
      );
    case 'info':
      return (
        <svg
          className={`w-5 h-5 ${iconColor}`}
          fill="currentColor"
          viewBox="0 0 20 20"
          aria-hidden="true"
        >
          <path
            fillRule="evenodd"
            d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
            clipRule="evenodd"
          />
        </svg>
      );
    case 'tips':
      return (
        <svg
          className={`w-5 h-5 ${iconColor}`}
          fill="currentColor"
          viewBox="0 0 20 20"
          aria-hidden="true"
        >
          <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
          <path
            fillRule="evenodd"
            d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z"
            clipRule="evenodd"
          />
        </svg>
      );
  }
}

// ========================================
// コンポーネント
// ========================================

/**
 * トースト通知コンポーネント
 *
 * レベル別スタイリング、自動消去タイマー、アニメーション、アクセシビリティ対応
 *
 * @example
 * <Toast alert={alert} onDismiss={handleDismiss} />
 */
export function Toast({ alert, onDismiss }: ToastProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [isExiting, setIsExiting] = useState(false);

  const { bgColor, borderColor, textColor } = getSeverityStyles(alert.severity);
  const timeout = AUTO_DISMISS_TIMEOUT[alert.severity];

  // マウント時のスライドインアニメーション
  useEffect(() => {
    const timer = setTimeout(() => setIsVisible(true), 10);
    return () => clearTimeout(timer);
  }, []);

  // 自動消去タイマー
  useEffect(() => {
    if (timeout === null) {
      return;
    }

    const timer = setTimeout(() => {
      handleDismiss();
    }, timeout);

    return () => clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [timeout]);

  // Dismiss処理（スライドアウトアニメーション付き）
  const handleDismiss = () => {
    setIsExiting(true);
    setTimeout(() => {
      onDismiss(alert.id);
    }, 300); // アニメーション時間
  };

  return (
    <div
      className={`
        ${bgColor} ${borderColor} ${textColor}
        border-l-4 p-4 rounded-md shadow-lg
        transform transition-all duration-300 ease-in-out
        ${isVisible && !isExiting ? 'translate-x-0 opacity-100' : 'translate-x-full opacity-0'}
      `}
      role="alert"
      aria-live={alert.severity === 'critical' || alert.severity === 'warning' ? 'assertive' : 'polite'}
      aria-atomic="true"
    >
      <div className="flex items-start">
        {/* アイコン */}
        <div className="flex-shrink-0">
          <SeverityIcon severity={alert.severity} />
        </div>

        {/* コンテンツ */}
        <div className="ml-3 flex-1">
          <h3 className="text-sm font-medium">{alert.title}</h3>
          <div className="mt-1 text-sm">{alert.message}</div>
        </div>

        {/* 閉じるボタン */}
        <button
          onClick={handleDismiss}
          className={`ml-3 flex-shrink-0 inline-flex ${textColor} hover:opacity-75 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-${alert.severity === 'critical' ? 'red' : alert.severity === 'warning' ? 'yellow' : 'blue'}-500`}
          aria-label="通知を閉じる"
        >
          <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
            <path
              fillRule="evenodd"
              d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
              clipRule="evenodd"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}
