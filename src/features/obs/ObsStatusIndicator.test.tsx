import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '../../tests/utils/test-utils';
import { ObsStatusIndicator } from './ObsStatusIndicator';
import { useObsStore } from '../../stores/obsStore';
import type { ObsStatus } from '../../types/commands';

describe('ObsStatusIndicator', () => {
  beforeEach(() => {
    // ストアをリセット
    useObsStore.setState({
      connectionState: 'disconnected',
      status: null,
      error: null,
      warning: null,
      loading: false,
      scenes: [],
      lastConnectionParams: null,
    });
  });

  describe('未接続状態', () => {
    it('未接続メッセージを表示する', () => {
      render(<ObsStatusIndicator />);

      expect(screen.getByText('OBSに接続されていません')).toBeInTheDocument();
    });

    it('接続済みインジケーターは表示しない', () => {
      render(<ObsStatusIndicator />);

      expect(screen.queryByText('接続済み')).not.toBeInTheDocument();
    });
  });

  describe('接続状態', () => {
    const mockStatus: ObsStatus = {
      connected: true,
      streaming: false,
      recording: false,
      virtualCamActive: false,
      currentScene: 'テストシーン',
      obsVersion: '30.0.0',
      websocketVersion: '5.0.0',
      streamTimecode: null,
      recordTimecode: null,
      streamBitrate: null,
      recordBitrate: null,
      fps: 60.0,
      renderDroppedFrames: 0,
      outputDroppedFrames: 0,
    };

    it('接続済みメッセージとバージョン情報を表示する', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: mockStatus,
      });

      render(<ObsStatusIndicator />);

      expect(screen.getByText('接続済み')).toBeInTheDocument();
      expect(screen.getByText('OBS 30.0.0 / WS 5.0.0')).toBeInTheDocument();
    });

    it('現在のシーンを表示する', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: mockStatus,
      });

      render(<ObsStatusIndicator />);

      expect(screen.getByText('現在のシーン')).toBeInTheDocument();
      expect(screen.getByText('テストシーン')).toBeInTheDocument();
    });

    it('FPSを表示する', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: mockStatus,
      });

      render(<ObsStatusIndicator />);

      expect(screen.getByText('60.0')).toBeInTheDocument();
      expect(screen.getByText('FPS')).toBeInTheDocument();
    });

    it('ドロップフレームを表示する', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: mockStatus,
      });

      render(<ObsStatusIndicator />);

      expect(screen.getByText('レンダードロップ')).toBeInTheDocument();
      expect(screen.getByText('出力ドロップ')).toBeInTheDocument();
    });

    it('シーンがnullの場合は「不明」と表示する', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: {
          ...mockStatus,
          currentScene: null,
        },
      });

      render(<ObsStatusIndicator />);

      expect(screen.getByText('不明')).toBeInTheDocument();
    });
  });

  describe('配信状態', () => {
    it('配信停止中は適切なメッセージを表示する', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: {
          ...({} as ObsStatus),
          connected: true,
          streaming: false,
          recording: false,
          currentScene: 'テストシーン',
          fps: 60.0,
        },
      });

      render(<ObsStatusIndicator />);

      expect(screen.getByText('配信停止')).toBeInTheDocument();
    });

    it('配信中は詳細情報を表示する', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: {
          ...({} as ObsStatus),
          connected: true,
          streaming: true,
          recording: false,
          currentScene: 'テストシーン',
          streamTimecode: 120000, // 2分
          streamBitrate: 5000000, // 5Mbps
          fps: 60.0,
        },
      });

      render(<ObsStatusIndicator />);

      expect(screen.getByText('配信中')).toBeInTheDocument();
      expect(screen.getByText(/時間:/)).toBeInTheDocument();
      expect(screen.getByText(/ビットレート:/)).toBeInTheDocument();
    });

    it('配信タイムコードを正しくフォーマットする', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: {
          ...({} as ObsStatus),
          connected: true,
          streaming: true,
          streamTimecode: 3661000, // 1時間1分1秒
          streamBitrate: 5000000,
        },
      });

      render(<ObsStatusIndicator />);

      expect(screen.getByText(/01:01:01/)).toBeInTheDocument();
    });

    it('配信ビットレートを正しくフォーマットする', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: {
          ...({} as ObsStatus),
          connected: true,
          streaming: true,
          streamTimecode: 120000,
          streamBitrate: 5500000, // 5.5Mbps
        },
      });

      render(<ObsStatusIndicator />);

      expect(screen.getByText(/5\.50 Mbps/)).toBeInTheDocument();
    });
  });

  describe('録画状態', () => {
    it('録画停止中は適切なメッセージを表示する', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: {
          ...({} as ObsStatus),
          connected: true,
          streaming: false,
          recording: false,
          currentScene: 'テストシーン',
          fps: 60.0,
        },
      });

      render(<ObsStatusIndicator />);

      expect(screen.getByText('録画停止')).toBeInTheDocument();
    });

    it('録画中は詳細情報を表示する', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: {
          ...({} as ObsStatus),
          connected: true,
          streaming: false,
          recording: true,
          currentScene: 'テストシーン',
          recordTimecode: 180000, // 3分
          recordBitrate: 10000000, // 10Mbps
          fps: 60.0,
        },
      });

      render(<ObsStatusIndicator />);

      expect(screen.getByText('録画中')).toBeInTheDocument();
      expect(screen.getAllByText(/時間:/).length).toBeGreaterThan(0);
      expect(screen.getAllByText(/ビットレート:/).length).toBeGreaterThan(0);
    });
  });

  describe('ドロップフレーム警告', () => {
    it('レンダードロップがある場合は黄色で表示する', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: {
          ...({} as ObsStatus),
          connected: true,
          renderDroppedFrames: 10,
          outputDroppedFrames: 0,
          fps: 60.0,
        },
      });

      const { container } = render(<ObsStatusIndicator />);
      const renderedDropText = screen.getByText('10');
      expect(renderedDropText.className).toContain('text-yellow-600');
    });

    it('出力ドロップがある場合は赤色で表示する', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: {
          ...({} as ObsStatus),
          connected: true,
          renderDroppedFrames: 0,
          outputDroppedFrames: 5,
          fps: 60.0,
        },
      });

      const { container } = render(<ObsStatusIndicator />);
      const outputDropText = screen.getByText('5');
      expect(outputDropText.className).toContain('text-red-600');
    });

    it('ドロップフレームがない場合は通常色で表示する', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: {
          ...({} as ObsStatus),
          connected: true,
          renderDroppedFrames: 0,
          outputDroppedFrames: 0,
          fps: 60.0,
        },
      });

      const { container } = render(<ObsStatusIndicator />);
      const renderDropText = screen.getAllByText('0')[0];
      expect(renderDropText.className).toContain('text-gray-800');
    });
  });

  describe('仮想カメラ', () => {
    it('仮想カメラが有効な場合は表示する', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: {
          ...({} as ObsStatus),
          connected: true,
          virtualCamActive: true,
          fps: 60.0,
        },
      });

      render(<ObsStatusIndicator />);

      expect(screen.getByText('仮想カメラ有効')).toBeInTheDocument();
    });

    it('仮想カメラが無効な場合は表示しない', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: {
          ...({} as ObsStatus),
          connected: true,
          virtualCamActive: false,
          fps: 60.0,
        },
      });

      render(<ObsStatusIndicator />);

      expect(screen.queryByText('仮想カメラ有効')).not.toBeInTheDocument();
    });
  });

  describe('アクセシビリティ', () => {
    it('role="status"とaria-live="polite"を持つ', () => {
      useObsStore.setState({
        connectionState: 'connected',
        status: {
          ...({} as ObsStatus),
          connected: true,
          fps: 60.0,
        },
      });

      const { container } = render(<ObsStatusIndicator />);
      const statusElement = container.querySelector('[role="status"]');

      expect(statusElement).toBeInTheDocument();
      expect(statusElement).toHaveAttribute('aria-live', 'polite');
    });
  });
});
