import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { EncoderBadge } from './EncoderBadge';

describe('EncoderBadge', () => {
  describe('基本表示', () => {
    it('GPUエンコーダー(AV1)を正しく表示', () => {
      render(<EncoderBadge encoderId="jim_av1_nvenc" />);
      expect(screen.getByText('NVIDIA NVENC (AV1)')).toBeInTheDocument();
    });

    it('GPUエンコーダー(H.264)を正しく表示', () => {
      render(<EncoderBadge encoderId="ffmpeg_nvenc" />);
      expect(screen.getByText('NVIDIA NVENC (H.264)')).toBeInTheDocument();
    });

    it('CPUエンコーダーを正しく表示', () => {
      render(<EncoderBadge encoderId="obs_x264" />);
      expect(screen.getByText('CPU (x264)')).toBeInTheDocument();
    });
  });

  describe('詳細表示', () => {
    it('showDetails=trueでコーデック情報を表示', () => {
      render(<EncoderBadge encoderId="jim_av1_nvenc" showDetails />);
      expect(screen.getByText('NVIDIA NVENC (AV1)')).toBeInTheDocument();
      expect(screen.getByText('AV1')).toBeInTheDocument();
      expect(screen.getByText('⚡')).toBeInTheDocument(); // GPUアイコン
    });

    it('showDetails=trueでCPUエンコーダーのアイコンを表示', () => {
      render(<EncoderBadge encoderId="obs_x264" showDetails />);
      expect(screen.getByText('CPU (x264)')).toBeInTheDocument();
      expect(screen.getByText('H.264')).toBeInTheDocument();
      expect(screen.getByText('💻')).toBeInTheDocument(); // CPUアイコン
    });

    it('showDetails=falseで詳細を非表示', () => {
      render(<EncoderBadge encoderId="jim_av1_nvenc" showDetails={false} />);
      expect(screen.getByText('NVIDIA NVENC (AV1)')).toBeInTheDocument();
      expect(screen.queryByText('AV1')).not.toBeInTheDocument();
      expect(screen.queryByText('⚡')).not.toBeInTheDocument();
    });
  });

  describe('スタイリング', () => {
    it('カスタムクラス名を適用', () => {
      render(<EncoderBadge encoderId="jim_av1_nvenc" className="custom-class" />);
      // カスタムクラスが適用されていることを確認
      const badge = screen.getByText('NVIDIA NVENC (AV1)').closest('.custom-class');
      expect(badge).toBeInTheDocument();
    });
  });

  describe('ツールチップ', () => {
    it('メインバッジにtitle属性を設定', () => {
      render(<EncoderBadge encoderId="jim_av1_nvenc" />);
      const badge = screen.getByText('NVIDIA NVENC (AV1)');
      expect(badge).toHaveAttribute('title');
      expect(badge.getAttribute('title')).toContain('GPU');
      expect(badge.getAttribute('title')).toContain('AV1');
    });
  });
});
