/**
 * エンコーダーIDをユーザー向けラベルに変換するユーティリティ
 * バックエンドの get_encoder_display_label() と同期
 */

/**
 * エンコーダーIDをユーザーフレンドリーなラベルに変換
 *
 * @param encoderId - OBSエンコーダーID (例: "jim_av1_nvenc", "ffmpeg_nvenc")
 * @returns ユーザー向けラベル (例: "NVIDIA NVENC (AV1)", "NVIDIA NVENC (H.264)")
 */
export function getEncoderDisplayLabel(encoderId: string): string {
  // 小文字に正規化
  const normalized = encoderId.toLowerCase().trim();

  // NVIDIA NVENC (AV1)
  if (normalized === 'jim_av1_nvenc' || normalized === 'jim_nvenc' || normalized.includes('av1_nvenc')) {
    return 'NVIDIA NVENC (AV1)';
  }

  // NVIDIA NVENC (HEVC/H.265)
  if (normalized === 'jim_hevc_nvenc' || normalized.includes('hevc_nvenc')) {
    return 'NVIDIA NVENC (HEVC)';
  }

  // NVIDIA NVENC (H.264)
  if (
    normalized === 'ffmpeg_nvenc' ||
    normalized === 'nvenc' ||
    normalized === 'nvenc_h264' ||
    normalized.includes('nvenc')
  ) {
    return 'NVIDIA NVENC (H.264)';
  }

  // AMD VCE/AMF (H.264)
  if (
    normalized === 'amd_amf_h264' ||
    normalized === 'h264_amd' ||
    normalized.includes('amf') ||
    normalized.includes('amd')
  ) {
    return 'AMD VCE (H.264)';
  }

  // Intel Quick Sync Video
  if (normalized === 'obs_qsv11' || normalized.includes('qsv')) {
    return 'Intel Quick Sync Video';
  }

  // CPU x264 (H.264)
  if (normalized === 'obs_x264' || normalized === 'x264') {
    return 'CPU (x264)';
  }

  // CPU x265 (HEVC)
  if (normalized === 'obs_x265' || normalized === 'x265') {
    return 'CPU (x265)';
  }

  // Apple Video Toolbox
  if (normalized.includes('videotoolbox') || normalized.includes('vt_')) {
    return 'Apple VideoToolbox';
  }

  // AV1ソフトウェアエンコーダー
  if (normalized.includes('av1') && !normalized.includes('nvenc')) {
    return 'CPU (AV1)';
  }

  // 未知のエンコーダーはIDをそのまま返す
  return encoderId;
}

/**
 * エンコーダーの種類を判定
 *
 * @param encoderId - OBSエンコーダーID
 * @returns エンコーダーの種類 ("gpu" | "cpu" | "unknown")
 */
export function getEncoderType(encoderId: string): 'gpu' | 'cpu' | 'unknown' {
  const normalized = encoderId.toLowerCase().trim();

  if (
    normalized.includes('nvenc') ||
    normalized.includes('amf') ||
    normalized.includes('amd') ||
    normalized.includes('qsv') ||
    normalized.includes('videotoolbox')
  ) {
    return 'gpu';
  }

  if (normalized.includes('x264') || normalized.includes('x265')) {
    return 'cpu';
  }

  // AV1は文脈依存
  if (normalized.includes('av1')) {
    return normalized.includes('nvenc') ? 'gpu' : 'cpu';
  }

  return 'unknown';
}

/**
 * エンコーダーが対応するコーデックを判定
 *
 * @param encoderId - OBSエンコーダーID
 * @returns コーデック名 ("H.264" | "HEVC" | "AV1" | "不明")
 */
export function getEncoderCodec(encoderId: string): 'H.264' | 'HEVC' | 'AV1' | '不明' {
  const normalized = encoderId.toLowerCase().trim();

  if (normalized.includes('av1')) {
    return 'AV1';
  }

  if (normalized.includes('hevc') || normalized.includes('h265') || normalized.includes('x265')) {
    return 'HEVC';
  }

  if (
    normalized.includes('h264') ||
    normalized.includes('x264') ||
    normalized.includes('nvenc') ||
    normalized.includes('amf') ||
    normalized.includes('qsv')
  ) {
    return 'H.264';
  }

  return '不明';
}

/**
 * エンコーダー情報のサマリーを取得
 *
 * @param encoderId - OBSエンコーダーID
 * @returns エンコーダー情報
 */
export interface EncoderInfo {
  /** ユーザー向けラベル */
  label: string;
  /** エンコーダーの種類 */
  type: 'gpu' | 'cpu' | 'unknown';
  /** コーデック */
  codec: 'H.264' | 'HEVC' | 'AV1' | '不明';
  /** 元のID */
  rawId: string;
}

export function getEncoderInfo(encoderId: string): EncoderInfo {
  return {
    label: getEncoderDisplayLabel(encoderId),
    type: getEncoderType(encoderId),
    codec: getEncoderCodec(encoderId),
    rawId: encoderId,
  };
}
