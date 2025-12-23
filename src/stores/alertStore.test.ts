import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useAlertStore } from './alertStore';
import type { AlertSeverity } from './alertStore';

describe('alertStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // ストアをリセット
    useAlertStore.setState({
      alerts: [],
      streamingMode: false,
    });
  });

  describe('初期状態', () => {
    it('アラートが空配列で初期化される', () => {
      const state = useAlertStore.getState();
      expect(state.alerts).toEqual([]);
      expect(state.streamingMode).toBe(false);
    });
  });

  describe('addAlert', () => {
    it('新しいアラートを追加できる', () => {
      const { addAlert } = useAlertStore.getState();

      addAlert({
        severity: 'warning',
        title: 'テスト警告',
        message: 'これはテストメッセージです',
      });

      const state = useAlertStore.getState();
      expect(state.alerts).toHaveLength(1);
      expect(state.alerts[0].title).toBe('テスト警告');
      expect(state.alerts[0].message).toBe('これはテストメッセージです');
      expect(state.alerts[0].severity).toBe('warning');
      expect(state.alerts[0].dismissed).toBe(false);
    });

    it('アラートにユニークなIDを付与する', () => {
      const { addAlert } = useAlertStore.getState();

      addAlert({
        severity: 'info',
        title: 'アラート1',
        message: 'メッセージ1',
      });

      addAlert({
        severity: 'info',
        title: 'アラート2',
        message: 'メッセージ2',
      });

      const state = useAlertStore.getState();
      expect(state.alerts[0].id).toBeDefined();
      expect(state.alerts[1].id).toBeDefined();
      expect(state.alerts[0].id).not.toBe(state.alerts[1].id);
    });

    it('タイムスタンプを自動的に設定する', () => {
      const { addAlert } = useAlertStore.getState();
      const beforeTimestamp = Date.now();

      addAlert({
        severity: 'info',
        title: 'タイムスタンプテスト',
        message: 'テスト',
      });

      const state = useAlertStore.getState();
      const afterTimestamp = Date.now();

      expect(state.alerts[0].timestamp).toBeGreaterThanOrEqual(beforeTimestamp);
      expect(state.alerts[0].timestamp).toBeLessThanOrEqual(afterTimestamp);
    });

    it('複数のアラートを新しい順に並べる', () => {
      const { addAlert } = useAlertStore.getState();

      addAlert({
        severity: 'info',
        title: '最初のアラート',
        message: 'メッセージ1',
      });

      addAlert({
        severity: 'warning',
        title: '2番目のアラート',
        message: 'メッセージ2',
      });

      const state = useAlertStore.getState();
      expect(state.alerts[0].title).toBe('2番目のアラート');
      expect(state.alerts[1].title).toBe('最初のアラート');
    });

    it('全てのseverityレベルのアラートを追加できる', () => {
      const { addAlert } = useAlertStore.getState();
      const severities: AlertSeverity[] = ['critical', 'warning', 'info', 'tips'];

      severities.forEach((severity) => {
        addAlert({
          severity,
          title: `${severity}アラート`,
          message: `${severity}メッセージ`,
        });
      });

      const state = useAlertStore.getState();
      expect(state.alerts).toHaveLength(4);
      severities.forEach((severity, index) => {
        expect(state.alerts[3 - index].severity).toBe(severity);
      });
    });
  });

  describe('dismissAlert', () => {
    it('指定したIDのアラートをdismissできる', () => {
      const { addAlert, dismissAlert } = useAlertStore.getState();

      addAlert({
        severity: 'info',
        title: 'テストアラート',
        message: 'テスト',
      });

      const alertId = useAlertStore.getState().alerts[0].id;
      dismissAlert(alertId);

      const state = useAlertStore.getState();
      expect(state.alerts[0].dismissed).toBe(true);
    });

    it('複数のアラートから特定のアラートのみdismissできる', () => {
      const { addAlert, dismissAlert } = useAlertStore.getState();

      addAlert({
        severity: 'info',
        title: 'アラート1',
        message: 'メッセージ1',
      });

      addAlert({
        severity: 'warning',
        title: 'アラート2',
        message: 'メッセージ2',
      });

      addAlert({
        severity: 'critical',
        title: 'アラート3',
        message: 'メッセージ3',
      });

      const alerts = useAlertStore.getState().alerts;
      const targetId = alerts[1].id; // 2番目のアラートをdismiss

      dismissAlert(targetId);

      const state = useAlertStore.getState();
      expect(state.alerts[0].dismissed).toBe(false);
      expect(state.alerts[1].dismissed).toBe(true);
      expect(state.alerts[2].dismissed).toBe(false);
    });

    it('存在しないIDでdismissしても他のアラートに影響しない', () => {
      const { addAlert, dismissAlert } = useAlertStore.getState();

      addAlert({
        severity: 'info',
        title: 'テストアラート',
        message: 'テスト',
      });

      dismissAlert('non-existent-id');

      const state = useAlertStore.getState();
      expect(state.alerts[0].dismissed).toBe(false);
    });
  });

  describe('clearAll', () => {
    it('全てのアラートをdismissできる', () => {
      const { addAlert, clearAll } = useAlertStore.getState();

      addAlert({
        severity: 'info',
        title: 'アラート1',
        message: 'メッセージ1',
      });

      addAlert({
        severity: 'warning',
        title: 'アラート2',
        message: 'メッセージ2',
      });

      addAlert({
        severity: 'critical',
        title: 'アラート3',
        message: 'メッセージ3',
      });

      clearAll();

      const state = useAlertStore.getState();
      expect(state.alerts).toHaveLength(3);
      state.alerts.forEach((alert) => {
        expect(alert.dismissed).toBe(true);
      });
    });

    it('アラートがない状態でclearAllを呼んでもエラーにならない', () => {
      const { clearAll } = useAlertStore.getState();

      expect(() => clearAll()).not.toThrow();

      const state = useAlertStore.getState();
      expect(state.alerts).toEqual([]);
    });
  });

  describe('setStreamingMode', () => {
    it('ストリーミングモードを有効化できる', () => {
      const { setStreamingMode } = useAlertStore.getState();

      setStreamingMode(true);

      const state = useAlertStore.getState();
      expect(state.streamingMode).toBe(true);
    });

    it('ストリーミングモードを無効化できる', () => {
      useAlertStore.setState({ streamingMode: true });

      const { setStreamingMode } = useAlertStore.getState();
      setStreamingMode(false);

      const state = useAlertStore.getState();
      expect(state.streamingMode).toBe(false);
    });

    it('ストリーミングモードのトグルが正しく動作する', () => {
      const { setStreamingMode } = useAlertStore.getState();

      setStreamingMode(true);
      expect(useAlertStore.getState().streamingMode).toBe(true);

      setStreamingMode(false);
      expect(useAlertStore.getState().streamingMode).toBe(false);

      setStreamingMode(true);
      expect(useAlertStore.getState().streamingMode).toBe(true);
    });
  });

  describe('getActiveAlerts', () => {
    it('未dismissのアラートのみ取得できる', () => {
      const { addAlert, dismissAlert, getActiveAlerts } = useAlertStore.getState();

      addAlert({
        severity: 'info',
        title: 'アラート1',
        message: 'メッセージ1',
      });

      addAlert({
        severity: 'warning',
        title: 'アラート2',
        message: 'メッセージ2',
      });

      addAlert({
        severity: 'critical',
        title: 'アラート3',
        message: 'メッセージ3',
      });

      const alerts = useAlertStore.getState().alerts;
      dismissAlert(alerts[1].id); // 2番目をdismiss

      const activeAlerts = getActiveAlerts();
      expect(activeAlerts).toHaveLength(2);
      expect(activeAlerts[0].title).toBe('アラート3');
      expect(activeAlerts[1].title).toBe('アラート1');
    });

    it('全てdismissされている場合は空配列を返す', () => {
      const { addAlert, clearAll, getActiveAlerts } = useAlertStore.getState();

      addAlert({
        severity: 'info',
        title: 'アラート1',
        message: 'メッセージ1',
      });

      clearAll();

      const activeAlerts = getActiveAlerts();
      expect(activeAlerts).toEqual([]);
    });

    it('アラートがない場合は空配列を返す', () => {
      const { getActiveAlerts } = useAlertStore.getState();

      const activeAlerts = getActiveAlerts();
      expect(activeAlerts).toEqual([]);
    });
  });

  describe('getAlertsByPriority', () => {
    it('重要度順（critical > warning > info > tips）でソートされる', () => {
      const { addAlert, getAlertsByPriority } = useAlertStore.getState();

      // わざと逆順に追加
      addAlert({
        severity: 'tips',
        title: 'Tipsアラート',
        message: 'Tips',
      });

      addAlert({
        severity: 'info',
        title: 'Infoアラート',
        message: 'Info',
      });

      addAlert({
        severity: 'warning',
        title: 'Warningアラート',
        message: 'Warning',
      });

      addAlert({
        severity: 'critical',
        title: 'Criticalアラート',
        message: 'Critical',
      });

      const sortedAlerts = getAlertsByPriority();
      expect(sortedAlerts).toHaveLength(4);
      expect(sortedAlerts[0].severity).toBe('critical');
      expect(sortedAlerts[1].severity).toBe('warning');
      expect(sortedAlerts[2].severity).toBe('info');
      expect(sortedAlerts[3].severity).toBe('tips');
    });

    it('同じseverityのアラートは元の順序を保持する', () => {
      const { addAlert, getAlertsByPriority } = useAlertStore.getState();

      addAlert({
        severity: 'info',
        title: 'Info1',
        message: 'メッセージ1',
      });

      addAlert({
        severity: 'info',
        title: 'Info2',
        message: 'メッセージ2',
      });

      addAlert({
        severity: 'info',
        title: 'Info3',
        message: 'メッセージ3',
      });

      const sortedAlerts = getAlertsByPriority();
      expect(sortedAlerts[0].title).toBe('Info3');
      expect(sortedAlerts[1].title).toBe('Info2');
      expect(sortedAlerts[2].title).toBe('Info1');
    });

    it('dismissedのアラートも含めてソートされる', () => {
      const { addAlert, dismissAlert, getAlertsByPriority } = useAlertStore.getState();

      addAlert({
        severity: 'warning',
        title: 'Warning',
        message: 'Warning',
      });

      addAlert({
        severity: 'critical',
        title: 'Critical',
        message: 'Critical',
      });

      const alerts = useAlertStore.getState().alerts;
      dismissAlert(alerts[0].id); // Criticalをdismiss

      const sortedAlerts = getAlertsByPriority();
      expect(sortedAlerts).toHaveLength(2);
      expect(sortedAlerts[0].severity).toBe('critical');
      expect(sortedAlerts[0].dismissed).toBe(true);
    });

    it('元の配列を変更しない（イミュータブル）', () => {
      const { addAlert, getAlertsByPriority } = useAlertStore.getState();

      addAlert({
        severity: 'info',
        title: 'Info',
        message: 'Info',
      });

      addAlert({
        severity: 'critical',
        title: 'Critical',
        message: 'Critical',
      });

      const originalAlerts = useAlertStore.getState().alerts;
      const sortedAlerts = getAlertsByPriority();

      // ソートされた配列は元の配列と異なる順序（criticalは最初、infoは2番目→ソート後は逆）
      expect(sortedAlerts[0].severity).toBe('critical');
      expect(sortedAlerts[1].severity).toBe('info');
      // 元の配列は新しい順（critical, info）
      expect(originalAlerts[0].severity).toBe('critical');
      expect(originalAlerts[1].severity).toBe('info');

      // sortedAlertsは新しい配列である
      expect(sortedAlerts).not.toBe(originalAlerts);
    });
  });

  describe('複合操作', () => {
    it('アラート追加、dismiss、アクティブ取得の一連の流れが正しく動作する', () => {
      const { addAlert, dismissAlert, getActiveAlerts } = useAlertStore.getState();

      // 5つのアラートを追加
      addAlert({ severity: 'critical', title: 'C1', message: 'Critical 1' });
      addAlert({ severity: 'warning', title: 'W1', message: 'Warning 1' });
      addAlert({ severity: 'info', title: 'I1', message: 'Info 1' });
      addAlert({ severity: 'tips', title: 'T1', message: 'Tips 1' });
      addAlert({ severity: 'warning', title: 'W2', message: 'Warning 2' });

      expect(useAlertStore.getState().alerts).toHaveLength(5);

      // 2つをdismiss
      const alerts = useAlertStore.getState().alerts;
      dismissAlert(alerts[1].id); // W1
      dismissAlert(alerts[3].id); // T1

      // アクティブなアラートは3つ
      const activeAlerts = getActiveAlerts();
      expect(activeAlerts).toHaveLength(3);

      // アクティブなアラートにdismissされたものは含まれない
      const activeTitles = activeAlerts.map((a) => a.title);
      expect(activeTitles).not.toContain('W1');
      expect(activeTitles).not.toContain('T1');
    });

    it('ストリーミングモードの変更がアラートに影響しない', () => {
      const { addAlert, setStreamingMode, getActiveAlerts } = useAlertStore.getState();

      addAlert({
        severity: 'info',
        title: 'テスト',
        message: 'メッセージ',
      });

      setStreamingMode(true);

      const activeAlerts = getActiveAlerts();
      expect(activeAlerts).toHaveLength(1);
      expect(useAlertStore.getState().streamingMode).toBe(true);
    });
  });
});
