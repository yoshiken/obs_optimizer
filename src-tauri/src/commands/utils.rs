// コマンド共通ユーティリティ
//
// 複数のコマンドで共有する関数を提供

use crate::monitor::{get_cpu_core_count, get_memory_info};
use crate::monitor::gpu::get_gpu_info;
use crate::services::optimizer::HardwareInfo;

/// ハードウェア情報を取得（共通関数）
///
/// CPU、メモリ、GPU情報を収集し、HardwareInfo構造体を返す
///
/// # Returns
/// ハードウェア情報
pub async fn get_hardware_info() -> HardwareInfo {
    let cpu_cores = get_cpu_core_count().unwrap_or(4);
    let (_, total_memory) = get_memory_info().unwrap_or((0, 8_000_000_000)); // デフォルト8GB
    let total_memory_gb = total_memory as f64 / 1_000_000_000.0;
    let gpu_info = get_gpu_info().await;

    HardwareInfo {
        cpu_name: "CPU".to_string(), // TODO: 実際のCPU名を取得
        cpu_cores,
        total_memory_gb,
        gpu: gpu_info,
    }
}
