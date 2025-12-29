// システム監視モジュール
//
// CPU、メモリ、GPU、ネットワーク、プロセスの監視機能を提供

pub mod gpu;
pub mod network;
pub mod process;

#[cfg(test)]
mod tests;

use sysinfo::System;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::error::AppError;

// 公開エクスポート
pub use gpu::GpuMetrics;
pub use network::NetworkMetrics;
pub use process::ObsProcessMetrics;

// グローバルなSystem インスタンス（スレッドセーフ）
// Mutex::lock() はpoisoned状態（パニック発生時）でもmap_errで適切にエラー変換される
// 競合時はロック取得まで待機し、デッドロックは発生しない設計
static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| {
    Mutex::new(System::new_all())
});

/// CPU使用率を取得（0-100%）
pub fn get_cpu_usage() -> Result<f32, AppError> {
    let mut sys = SYSTEM.lock()
        .map_err(|e| AppError::system_monitor(&format!("Failed to lock system mutex: {e}")))?;
    sys.refresh_cpu_usage();

    // 全CPUの平均使用率を計算
    let cpus = sys.cpus();
    if cpus.is_empty() {
        return Ok(0.0);
    }

    let total: f32 = cpus.iter().map(sysinfo::Cpu::cpu_usage).sum();
    let avg = total / cpus.len() as f32;

    // 値の妥当性チェック
    if avg.is_nan() || avg.is_infinite() {
        return Err(AppError::system_monitor("Invalid CPU usage value"));
    }

    // 0-100の範囲にクランプ
    Ok(avg.clamp(0.0, 100.0))
}

/// メモリ情報を取得（使用量, 総量）バイト単位
pub fn get_memory_info() -> Result<(u64, u64), AppError> {
    let mut sys = SYSTEM.lock()
        .map_err(|e| AppError::system_monitor(&format!("Failed to lock system mutex: {e}")))?;
    sys.refresh_memory();
    Ok((sys.used_memory(), sys.total_memory()))
}

/// CPUコア数を取得
pub fn get_cpu_core_count() -> Result<usize, AppError> {
    let sys = SYSTEM.lock()
        .map_err(|e| AppError::system_monitor(&format!("Failed to lock system mutex: {e}")))?;
    Ok(sys.cpus().len())
}

/// 各CPUコアの使用率を取得（0-100%のベクター）
pub fn get_per_core_cpu_usage() -> Result<Vec<f32>, AppError> {
    let mut sys = SYSTEM.lock()
        .map_err(|e| AppError::system_monitor(&format!("Failed to lock system mutex: {e}")))?;
    sys.refresh_cpu_usage();

    let usage: Vec<f32> = sys.cpus()
        .iter()
        .map(|cpu| {
            let usage = cpu.cpu_usage();
            if usage.is_nan() || usage.is_infinite() {
                0.0
            } else {
                usage.clamp(0.0, 100.0)
            }
        })
        .collect();

    Ok(usage)
}

/// 利用可能なメモリを取得（バイト単位）
pub fn get_available_memory() -> Result<u64, AppError> {
    let mut sys = SYSTEM.lock()
        .map_err(|e| AppError::system_monitor(&format!("Failed to lock system mutex: {e}")))?;
    sys.refresh_memory();
    Ok(sys.available_memory())
}

/// CPU名（ブランド名）を取得
///
/// 最初のCPUコアのブランド情報を返す。
/// システムにCPUが見つからない場合は "Unknown CPU" を返す。
pub fn get_cpu_name() -> Result<String, AppError> {
    let sys = SYSTEM.lock()
        .map_err(|e| AppError::system_monitor(&format!("Failed to lock system mutex: {e}")))?;

    // 最初のCPUコアのブランド名を取得
    let cpu_name = sys.cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string());

    Ok(cpu_name)
}
