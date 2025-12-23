import { useAlertStore } from '../stores/alertStore';
import { Toast } from './Toast';

/**
 * トースト通知コンテナ
 *
 * 画面右上に固定表示し、複数の通知を管理
 *
 * @example
 * // App.tsxに配置
 * <ToastContainer />
 */
export function ToastContainer() {
  const { getActiveAlerts, dismissAlert, streamingMode } = useAlertStore();

  const activeAlerts = getActiveAlerts();

  // ストリーミングモード時はCriticalとWarningのみ表示
  const visibleAlerts = streamingMode
    ? activeAlerts.filter(
        (alert) => alert.severity === 'critical' || alert.severity === 'warning'
      )
    : activeAlerts;

  if (visibleAlerts.length === 0) {
    return null;
  }

  return (
    <div
      className="fixed top-4 right-4 z-50 space-y-3 w-96 max-w-full"
      aria-live="polite"
      aria-label="通知一覧"
    >
      {visibleAlerts.map((alert) => (
        <Toast key={alert.id} alert={alert} onDismiss={dismissAlert} />
      ))}
    </div>
  );
}
