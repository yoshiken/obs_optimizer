import { describe, expect, it } from 'vitest';
import {
  getEncoderCodec,
  getEncoderDisplayLabel,
  getEncoderInfo,
  getEncoderType,
} from './encoderLabels';

describe('encoderLabels', () => {
  describe('getEncoderDisplayLabel', () => {
    it('NVIDIA NVENC (AV1)エンコーダーを正しく変換', () => {
      expect(getEncoderDisplayLabel('jim_av1_nvenc')).toBe('NVIDIA NVENC (AV1)');
      expect(getEncoderDisplayLabel('jim_nvenc')).toBe('NVIDIA NVENC (AV1)');
      expect(getEncoderDisplayLabel('JIM_AV1_NVENC')).toBe('NVIDIA NVENC (AV1)');
    });

    it('NVIDIA NVENC (H.264)エンコーダーを正しく変換', () => {
      expect(getEncoderDisplayLabel('ffmpeg_nvenc')).toBe('NVIDIA NVENC (H.264)');
      expect(getEncoderDisplayLabel('nvenc')).toBe('NVIDIA NVENC (H.264)');
      expect(getEncoderDisplayLabel('nvenc_h264')).toBe('NVIDIA NVENC (H.264)');
    });

    it('NVIDIA NVENC (HEVC)エンコーダーを正しく変換', () => {
      expect(getEncoderDisplayLabel('jim_hevc_nvenc')).toBe('NVIDIA NVENC (HEVC)');
    });

    it('AMD VCEエンコーダーを正しく変換', () => {
      expect(getEncoderDisplayLabel('amd_amf_h264')).toBe('AMD VCE (H.264)');
      expect(getEncoderDisplayLabel('h264_amd')).toBe('AMD VCE (H.264)');
    });

    it('Intel Quick Syncエンコーダーを正しく変換', () => {
      expect(getEncoderDisplayLabel('obs_qsv11')).toBe('Intel Quick Sync Video');
    });

    it('CPUエンコーダーを正しく変換', () => {
      expect(getEncoderDisplayLabel('obs_x264')).toBe('CPU (x264)');
      expect(getEncoderDisplayLabel('x264')).toBe('CPU (x264)');
      expect(getEncoderDisplayLabel('obs_x265')).toBe('CPU (x265)');
      expect(getEncoderDisplayLabel('x265')).toBe('CPU (x265)');
    });

    it('Apple VideoToolboxエンコーダーを正しく変換', () => {
      expect(getEncoderDisplayLabel('videotoolbox_h264')).toBe('Apple VideoToolbox');
    });

    it('未知のエンコーダーはIDをそのまま返す', () => {
      expect(getEncoderDisplayLabel('unknown_encoder')).toBe('unknown_encoder');
    });
  });

  describe('getEncoderType', () => {
    it('GPUエンコーダーを正しく判定', () => {
      expect(getEncoderType('jim_av1_nvenc')).toBe('gpu');
      expect(getEncoderType('ffmpeg_nvenc')).toBe('gpu');
      expect(getEncoderType('amd_amf_h264')).toBe('gpu');
      expect(getEncoderType('obs_qsv11')).toBe('gpu');
      expect(getEncoderType('videotoolbox_h264')).toBe('gpu');
    });

    it('CPUエンコーダーを正しく判定', () => {
      expect(getEncoderType('obs_x264')).toBe('cpu');
      expect(getEncoderType('x264')).toBe('cpu');
      expect(getEncoderType('x265')).toBe('cpu');
    });

    it('未知のエンコーダーを正しく判定', () => {
      expect(getEncoderType('unknown_encoder')).toBe('unknown');
    });
  });

  describe('getEncoderCodec', () => {
    it('AV1コーデックを正しく判定', () => {
      expect(getEncoderCodec('jim_av1_nvenc')).toBe('AV1');
    });

    it('HEVCコーデックを正しく判定', () => {
      expect(getEncoderCodec('jim_hevc_nvenc')).toBe('HEVC');
      expect(getEncoderCodec('x265')).toBe('HEVC');
    });

    it('H.264コーデックを正しく判定', () => {
      expect(getEncoderCodec('ffmpeg_nvenc')).toBe('H.264');
      expect(getEncoderCodec('obs_x264')).toBe('H.264');
      expect(getEncoderCodec('amd_amf_h264')).toBe('H.264');
      expect(getEncoderCodec('obs_qsv11')).toBe('H.264');
    });

    it('未知のコーデックを正しく判定', () => {
      expect(getEncoderCodec('unknown_encoder')).toBe('不明');
    });
  });

  describe('getEncoderInfo', () => {
    it('エンコーダー情報を正しく取得', () => {
      const info = getEncoderInfo('jim_av1_nvenc');
      expect(info.label).toBe('NVIDIA NVENC (AV1)');
      expect(info.type).toBe('gpu');
      expect(info.codec).toBe('AV1');
      expect(info.rawId).toBe('jim_av1_nvenc');
    });

    it('x264エンコーダー情報を正しく取得', () => {
      const info = getEncoderInfo('obs_x264');
      expect(info.label).toBe('CPU (x264)');
      expect(info.type).toBe('cpu');
      expect(info.codec).toBe('H.264');
      expect(info.rawId).toBe('obs_x264');
    });
  });
});
