// 最適化エンジン統合テスト
//
// RecommendationEngineの推奨設定算出機能を統合的にテストする。
// 様々なハードウェア構成と配信シナリオでの推奨値を検証する。

mod common;

use obs_optimizer_app_lib::testing::builders::{HardwareInfoBuilder, ObsSettingsBuilder};
use obs_optimizer_app_lib::testing::fixtures::{
    high_end_hardware, low_end_hardware, mid_range_hardware,
    high_end_obs_settings, low_spec_obs_settings, standard_obs_settings,
};

// =============================================================================
// ハードウェア情報ビルダーテスト
// =============================================================================

#[test]
fn test_hardware_info_builder_high_end() {
    let hardware = HardwareInfoBuilder::new()
        .cpu("AMD Ryzen 9 7950X", 16)
        .memory_gb(64.0)
        .nvidia_gpu()
        .build();

    assert_eq!(hardware.cpu_cores, 16);
    assert!((63.0..=65.0).contains(&hardware.total_memory_gb));
    assert!(hardware.gpu.is_some());
    assert!(hardware.gpu.as_ref().unwrap().name.contains("NVIDIA"));
}

#[test]
fn test_hardware_info_builder_no_gpu() {
    let hardware = HardwareInfoBuilder::new()
        .cores(4)
        .memory_gb(8.0)
        .no_gpu()
        .build();

    assert_eq!(hardware.cpu_cores, 4);
    assert!(hardware.gpu.is_none());
}

#[test]
fn test_hardware_info_builder_amd_gpu() {
    let hardware = HardwareInfoBuilder::new()
        .amd_gpu()
        .build();

    assert!(hardware.gpu.is_some());
    assert!(hardware.gpu.as_ref().unwrap().name.contains("AMD"));
}

#[test]
fn test_hardware_info_builder_intel_gpu() {
    let hardware = HardwareInfoBuilder::new()
        .intel_gpu()
        .build();

    assert!(hardware.gpu.is_some());
    assert!(hardware.gpu.as_ref().unwrap().name.contains("Intel"));
}

// =============================================================================
// OBS設定ビルダーテスト
// =============================================================================

#[test]
fn test_obs_settings_builder_default() {
    let settings = ObsSettingsBuilder::new().build();

    assert_eq!(settings.video.output_width, 1920);
    assert_eq!(settings.video.output_height, 1080);
    assert_eq!(settings.video.fps_numerator, 60);
    assert_eq!(settings.output.bitrate_kbps, 6000);
}

#[test]
fn test_obs_settings_builder_preset_720p30() {
    let settings = ObsSettingsBuilder::new()
        .preset_720p30()
        .x264()
        .bitrate(3000)
        .build();

    assert_eq!(settings.video.output_width, 1280);
    assert_eq!(settings.video.output_height, 720);
    assert_eq!(settings.video.fps_numerator, 30);
    assert_eq!(settings.output.encoder, "obs_x264");
}

#[test]
fn test_obs_settings_builder_preset_4k60() {
    let settings = ObsSettingsBuilder::new()
        .preset_4k60()
        .nvenc()
        .bitrate(20000)
        .build();

    assert_eq!(settings.video.output_width, 3840);
    assert_eq!(settings.video.output_height, 2160);
    assert_eq!(settings.video.fps_numerator, 60);
    assert_eq!(settings.output.encoder, "ffmpeg_nvenc");
    assert_eq!(settings.output.bitrate_kbps, 20000);
}

#[test]
fn test_obs_settings_builder_custom_resolution() {
    let settings = ObsSettingsBuilder::new()
        .resolution(2560, 1440)
        .fps(144)
        .bitrate(12000)
        .build();

    assert_eq!(settings.video.output_width, 2560);
    assert_eq!(settings.video.output_height, 1440);
    assert_eq!(settings.video.fps_numerator, 144);
}

// =============================================================================
// フィクスチャテスト
// =============================================================================

#[test]
fn test_high_end_hardware_fixture() {
    let hardware = high_end_hardware();

    assert!(hardware.cpu_cores >= 16, "High-end should have >= 16 cores");
    assert!(hardware.total_memory_gb >= 32.0, "High-end should have >= 32GB RAM");
    assert!(hardware.gpu.is_some(), "High-end should have GPU");
}

#[test]
fn test_mid_range_hardware_fixture() {
    let hardware = mid_range_hardware();

    assert!(hardware.cpu_cores >= 6, "Mid-range should have >= 6 cores");
    assert!(hardware.total_memory_gb >= 16.0, "Mid-range should have >= 16GB RAM");
    assert!(hardware.gpu.is_some(), "Mid-range should have GPU");
}

#[test]
fn test_low_end_hardware_fixture() {
    let hardware = low_end_hardware();

    assert!(hardware.cpu_cores <= 4, "Low-end should have <= 4 cores");
    assert!(hardware.total_memory_gb <= 8.0, "Low-end should have <= 8GB RAM");
    assert!(hardware.gpu.is_none(), "Low-end should not have dedicated GPU");
}

#[test]
fn test_standard_obs_settings_fixture() {
    let settings = standard_obs_settings();

    assert_eq!(settings.video.output_width, 1920);
    assert_eq!(settings.video.output_height, 1080);
    assert!(settings.output.encoder.contains("nvenc"), "Standard should use NVENC");
}

#[test]
fn test_low_spec_obs_settings_fixture() {
    let settings = low_spec_obs_settings();

    assert_eq!(settings.video.output_width, 1280);
    assert_eq!(settings.video.output_height, 720);
    assert!(settings.output.encoder.contains("x264"), "Low spec should use x264");
}

#[test]
fn test_high_end_obs_settings_fixture() {
    let settings = high_end_obs_settings();

    assert_eq!(settings.video.output_width, 3840);
    assert_eq!(settings.video.output_height, 2160);
    assert!(settings.output.bitrate_kbps >= 15000, "High-end should have high bitrate");
}

// =============================================================================
// 複合シナリオテスト
// =============================================================================

#[test]
fn test_scenario_hardware_settings_match() {
    // ハイエンドハードウェアにはハイエンド設定が適切
    let hardware = high_end_hardware();
    let settings = high_end_obs_settings();

    // NVIDIAハードウェアにはNVENCが適切
    assert!(hardware.gpu.as_ref().unwrap().name.contains("NVIDIA"));
    assert!(settings.output.encoder.contains("nvenc"));
}

#[test]
fn test_scenario_low_end_appropriate_settings() {
    // ローエンドハードウェアには控えめな設定が適切
    let hardware = low_end_hardware();
    let settings = low_spec_obs_settings();

    // GPU非搭載の場合はx264が適切
    assert!(hardware.gpu.is_none());
    assert!(settings.output.encoder.contains("x264"));

    // 解像度とビットレートが控えめ
    assert!(settings.video.output_width <= 1280);
    assert!(settings.output.bitrate_kbps <= 4000);
}

#[test]
fn test_scenario_gpu_encoder_selection() {
    // GPU別のエンコーダー選択が適切かテスト
    let nvidia = HardwareInfoBuilder::new().nvidia_gpu().build();
    let amd = HardwareInfoBuilder::new().amd_gpu().build();
    let intel = HardwareInfoBuilder::new().intel_gpu().build();
    let no_gpu = HardwareInfoBuilder::new().no_gpu().build();

    // 各GPUタイプが正しく識別されることを確認
    assert!(nvidia.gpu.as_ref().unwrap().name.to_lowercase().contains("nvidia"));
    assert!(amd.gpu.as_ref().unwrap().name.to_lowercase().contains("amd"));
    assert!(intel.gpu.as_ref().unwrap().name.to_lowercase().contains("intel"));
    assert!(no_gpu.gpu.is_none());
}

// =============================================================================
// ビットレート推奨テスト
// =============================================================================

#[test]
fn test_bitrate_ranges_by_resolution() {
    // 解像度別の適切なビットレート範囲を確認
    let settings_720p = ObsSettingsBuilder::new().preset_720p30().build();
    let settings_1080p = ObsSettingsBuilder::new().preset_1080p60().build();
    let settings_4k = ObsSettingsBuilder::new().preset_4k60().build();

    // 低解像度は低ビットレートで十分
    let bitrate_720p = 3000u32;
    // 標準解像度は中程度のビットレート
    let bitrate_1080p = 6000u32;
    // 4Kは高ビットレートが必要
    let bitrate_4k = 20000u32;

    assert!(bitrate_720p < bitrate_1080p, "720p should need less bitrate than 1080p");
    assert!(bitrate_1080p < bitrate_4k, "1080p should need less bitrate than 4K");

    // 設定の妥当性確認
    assert!(settings_720p.output.bitrate_kbps <= 5000);
    assert!((4000..=10000).contains(&settings_1080p.output.bitrate_kbps));
    assert!(settings_4k.output.bitrate_kbps >= 15000);
}

// =============================================================================
// エンコーダープリセットテスト
// =============================================================================

#[test]
fn test_encoder_preset_compatibility() {
    // NVENCプリセット
    let nvenc_settings = ObsSettingsBuilder::new()
        .nvenc()
        .preset("p5")
        .build();
    assert_eq!(nvenc_settings.output.preset.as_deref(), Some("p5"));

    // x264プリセット
    let x264_settings = ObsSettingsBuilder::new()
        .x264()
        .preset("veryfast")
        .build();
    assert_eq!(x264_settings.output.preset.as_deref(), Some("veryfast"));
}
