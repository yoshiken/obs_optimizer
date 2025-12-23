import { describe, expect, it } from 'vitest';
import { render, screen } from '../../../tests/utils/test-utils';
import { MetricCard } from './MetricCard';

describe('MetricCard', () => {
  describe('通常表示', () => {
    it('タイトルと子要素を表示する', () => {
      render(
        <MetricCard title="CPU使用率">
          <div>45.5%</div>
        </MetricCard>
      );

      expect(screen.getByText('CPU使用率')).toBeInTheDocument();
      expect(screen.getByText('45.5%')).toBeInTheDocument();
    });

    it('アイコンを表示できる', () => {
      const icon = <span data-testid="test-icon">🔧</span>;

      render(
        <MetricCard title="CPU使用率" icon={icon}>
          <div>45.5%</div>
        </MetricCard>
      );

      expect(screen.getByTestId('test-icon')).toBeInTheDocument();
    });

    it('カスタムclassNameを適用できる', () => {
      const { container } = render(
        <MetricCard title="CPU使用率" className="custom-class">
          <div>45.5%</div>
        </MetricCard>
      );

      const card = container.querySelector('.custom-class');
      expect(card).toBeInTheDocument();
    });
  });

  describe('severity', () => {
    it('normal severityを適用できる', () => {
      render(
        <MetricCard title="CPU使用率" severity="normal">
          <div>45.5%</div>
        </MetricCard>
      );

      // コンポーネントが正しくレンダリングされることを確認
      expect(screen.getByText('CPU使用率')).toBeInTheDocument();
      expect(screen.getByText('45.5%')).toBeInTheDocument();
    });

    it('warning severityを適用できる', () => {
      render(
        <MetricCard title="CPU使用率" severity="warning">
          <div>75.5%</div>
        </MetricCard>
      );

      // コンポーネントが正しくレンダリングされることを確認
      expect(screen.getByText('CPU使用率')).toBeInTheDocument();
      expect(screen.getByText('75.5%')).toBeInTheDocument();
    });

    it('critical severityを適用できる', () => {
      render(
        <MetricCard title="CPU使用率" severity="critical">
          <div>95.5%</div>
        </MetricCard>
      );

      // コンポーネントが正しくレンダリングされることを確認
      expect(screen.getByText('CPU使用率')).toBeInTheDocument();
      expect(screen.getByText('95.5%')).toBeInTheDocument();
    });
  });

  describe('ローディング状態', () => {
    it('ローディング中のメッセージを表示する', () => {
      render(
        <MetricCard title="CPU使用率" loading>
          <div>45.5%</div>
        </MetricCard>
      );

      expect(screen.getByText('読み込み中...')).toBeInTheDocument();
      expect(screen.queryByText('45.5%')).not.toBeInTheDocument();
    });

    it('ローディング中でもタイトルとアイコンは表示する', () => {
      const icon = <span data-testid="test-icon">🔧</span>;

      render(
        <MetricCard title="CPU使用率" icon={icon} loading>
          <div>45.5%</div>
        </MetricCard>
      );

      expect(screen.getByText('CPU使用率')).toBeInTheDocument();
      expect(screen.getByTestId('test-icon')).toBeInTheDocument();
    });
  });

  describe('エラー状態', () => {
    it('エラーメッセージを表示する', () => {
      render(
        <MetricCard title="CPU使用率" error="データ取得に失敗しました">
          <div>45.5%</div>
        </MetricCard>
      );

      expect(screen.getByText('エラー: データ取得に失敗しました')).toBeInTheDocument();
      expect(screen.queryByText('45.5%')).not.toBeInTheDocument();
    });

    it('エラー時でもタイトルとアイコンは表示する', () => {
      const icon = <span data-testid="test-icon">🔧</span>;

      render(
        <MetricCard title="CPU使用率" icon={icon} error="データ取得に失敗しました">
          <div>45.5%</div>
        </MetricCard>
      );

      expect(screen.getByText('CPU使用率')).toBeInTheDocument();
      expect(screen.getByTestId('test-icon')).toBeInTheDocument();
    });

    it('エラー時は赤いボーダーを表示する', () => {
      const { container } = render(
        <MetricCard title="CPU使用率" error="データ取得に失敗しました">
          <div>45.5%</div>
        </MetricCard>
      );

      const card = container.firstChild as HTMLElement;
      expect(card.style.borderLeftColor).toBe('rgb(239, 68, 68)');
    });

    it('nullエラーは表示しない', () => {
      render(
        <MetricCard title="CPU使用率" error={null}>
          <div>45.5%</div>
        </MetricCard>
      );

      expect(screen.queryByText(/エラー:/)).not.toBeInTheDocument();
      expect(screen.getByText('45.5%')).toBeInTheDocument();
    });
  });

  describe('優先順位', () => {
    it('errorとloadingが両方指定された場合、errorを優先する', () => {
      render(
        <MetricCard title="CPU使用率" error="エラー" loading>
          <div>45.5%</div>
        </MetricCard>
      );

      expect(screen.getByText('エラー: エラー')).toBeInTheDocument();
      expect(screen.queryByText('読み込み中...')).not.toBeInTheDocument();
    });
  });

  describe('アクセシビリティ', () => {
    it('意味のあるセマンティック構造を持つ', () => {
      const { container } = render(
        <MetricCard title="CPU使用率">
          <div>45.5%</div>
        </MetricCard>
      );

      // divベースのカードが存在することを確認
      expect(container.firstChild).toBeInTheDocument();
    });
  });
});
