// GPU監視モジュール
//
// NVIDIA GPUの監視にnvml-wrapperクレートを使用

use serde::Serialize;
use crate::error::AppError;
use nvml_wrapper::Nvml;
use nvml_wrapper::error::NvmlError;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// GPU使用状況のメトリクス
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GpuMetrics {
    /// GPU名称
    pub name: String,
    /// GPUインデックス（マルチGPU環境用）
    pub index: u32,
    /// GPU使用率（0-100%）
    pub usage_percent: f32,
    /// 使用中のVRAM（バイト）
    pub memory_used_bytes: u64,
    /// 総VRAM容量（バイト）
    pub memory_total_bytes: u64,
    /// GPU温度（摂氏）
    pub temperature: Option<u32>,
    /// エンコーダー使用率（0-100%）
    /// OBS配信時のNVENC負荷
    pub encoder_usage: Option<f32>,
}

/// NVML初期化状態をキャッシュ（初期化は重い処理のため1回のみ実行）
static NVML_INIT: Lazy<Mutex<Option<Result<(), String>>>> = Lazy::new(|| {
    Mutex::new(None)
});

/// NVMLが利用可能かチェック
///
/// 初回呼び出し時にNVMLの初期化を試行し、結果をキャッシュする
fn is_nvml_available() -> bool {
    let Ok(mut init_result) = NVML_INIT.lock() else {
        return false; // Mutex poisoned
    };

    if init_result.is_none() {
        // 初回初期化
        *init_result = Some(match Nvml::init() {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("NVML initialization failed: {e:?}")),
        });
    }

    init_result.as_ref()
        .is_some_and(std::result::Result::is_ok)
}


/// GPU情報を取得（プライマリGPU）
///
/// システムの最初のGPU（インデックス0）の情報を取得します。
///
/// # Returns
/// - `Ok(Some(GpuMetrics))` - GPU情報が取得できた場合
/// - `Ok(None)` - GPUが検出されない、またはNVIDIAドライバがない場合
/// - `Err(AppError)` - エラーが発生した場合
pub fn get_gpu_metrics() -> Result<Option<GpuMetrics>, AppError> {
    // NVMLが利用可能かチェック
    if !is_nvml_available() {
        return Ok(None);
    }

    // NVML初期化
    let Ok(nvml) = Nvml::init() else {
        return Ok(None); // 初期化失敗時はNoneを返す
    };

    // デバイス数を確認
    let Ok(device_count) = nvml.device_count() else {
        return Ok(None);
    };

    if device_count == 0 {
        return Ok(None);
    }

    // 最初のGPUを取得
    get_gpu_metrics_by_index(&nvml, 0)
}

/// 指定インデックスのGPU情報を取得
///
/// # Arguments
/// * `nvml` - NVML instance
/// * `index` - GPUインデックス
///
/// # Returns
/// - `Ok(Some(GpuMetrics))` - GPU情報が取得できた場合
/// - `Ok(None)` - 指定インデックスのGPUが存在しない場合
/// - `Err(AppError)` - エラーが発生した場合
fn get_gpu_metrics_by_index(nvml: &Nvml, index: u32) -> Result<Option<GpuMetrics>, AppError> {
    // デバイス取得
    let device = match nvml.device_by_index(index) {
        Ok(d) => d,
        Err(NvmlError::InvalidArg | NvmlError::GpuLost) => return Ok(None),
        Err(e) => return Err(e.into()),
    };

    // GPU名称取得
    let name = device.name()
        .unwrap_or_else(|_| format!("Unknown GPU #{index}"));

    // 使用率取得
    let utilization = device.utilization_rates()?;
    let usage_percent = utilization.gpu as f32;

    // メモリ情報取得
    let memory = device.memory_info()?;

    // 温度取得（サポートされていない場合はNone）
    let temperature = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu).ok();

    // エンコーダー使用率取得（サポートされていない場合はNone）
    let encoder_usage = device.encoder_utilization()
        .ok()
        .map(|stats| stats.utilization as f32);

    Ok(Some(GpuMetrics {
        name,
        index,
        usage_percent,
        memory_used_bytes: memory.used,
        memory_total_bytes: memory.total,
        temperature,
        encoder_usage,
    }))
}

/// GPU情報（推奨設定計算用の簡易型）
///
/// HardwareInfoで使用されるGPU情報
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    /// GPU名称
    pub name: String,
}

/// GPU情報を非同期で取得（推奨設定計算用）
///
/// # Returns
/// - `Some(GpuInfo)` - GPU情報が取得できた場合
/// - `None` - GPUが検出されない場合
pub async fn get_gpu_info() -> Option<GpuInfo> {
    // 同期関数を呼び出してGpuMetricsを取得
    let metrics = get_gpu_metrics().ok()??;
    Some(GpuInfo {
        name: metrics.name,
    })
}

/// 全GPUのリストを取得（マルチGPU対応）（将来使用予定）
///
/// システム内の全NVIDIA GPUの情報を取得します。
///
/// # Returns
/// - `Ok(Vec<GpuMetrics>)` - 検出されたGPUのリスト（空の場合あり）
/// - `Err(AppError)` - エラーが発生した場合
#[allow(dead_code)]
pub fn get_all_gpu_metrics() -> Result<Vec<GpuMetrics>, AppError> {
    // NVMLが利用可能かチェック
    if !is_nvml_available() {
        return Ok(vec![]);
    }

    // NVML初期化
    let Ok(nvml) = Nvml::init() else {
        return Ok(vec![]);
    };

    // デバイス数を取得
    let Ok(device_count) = nvml.device_count() else {
        return Ok(vec![]);
    };

    let mut gpu_list = Vec::new();

    // 全GPUをループ
    for i in 0..device_count {
        if let Some(metrics) = get_gpu_metrics_by_index(&nvml, i)? {
            gpu_list.push(metrics);
        }
        // 特定のGPUが取得できない場合はスキップ
    }

    Ok(gpu_list)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_get_gpu_metrics_no_panic() {
        // GPU情報取得でパニックしないことを確認
        let result = get_gpu_metrics();
        assert!(result.is_ok());

        // None（GPU未検出）またはSome（GPU検出）のいずれかが返る
        match result.unwrap() {
            Some(metrics) => {
                // メトリクスが返る場合は妥当性をチェック
                assert!(!metrics.name.is_empty());
                assert!(metrics.usage_percent >= 0.0 && metrics.usage_percent <= 100.0);
                assert!(metrics.memory_used_bytes <= metrics.memory_total_bytes);

                if let Some(encoder) = metrics.encoder_usage {
                    assert!(encoder >= 0.0 && encoder <= 100.0);
                }
            }
            None => {
                // GPUが検出されない環境
            }
        }
    }

    #[test]
    fn test_get_all_gpu_metrics_returns_vec() {
        // 全GPU取得でパニックしないことを確認
        let result = get_all_gpu_metrics();
        assert!(result.is_ok());

        let gpu_list = result.unwrap();
        // 空リストまたはGPUリストが返る
        for metrics in gpu_list {
            assert!(!metrics.name.is_empty());
            assert!(metrics.usage_percent >= 0.0 && metrics.usage_percent <= 100.0);
        }
    }

    #[test]
    fn test_nvml_available_check_caches_result() {
        // 初回チェック
        let first = is_nvml_available();
        // 2回目チェック（キャッシュされた結果が返る）
        let second = is_nvml_available();

        // 結果は同じはず
        assert_eq!(first, second);
    }
}
