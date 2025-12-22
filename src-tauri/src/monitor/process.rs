// プロセス監視モジュール
//
// OBSプロセスのリソース使用状況を監視

use serde::Serialize;
use sysinfo::System;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::error::AppError;

/// プロセスのリソース使用状況
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMetrics {
    /// プロセス名
    pub name: String,
    /// プロセスID
    pub pid: u32,
    /// CPU使用率（0-100%、コア数で正規化前の値）
    pub cpu_usage: f32,
    /// メモリ使用量（バイト）
    pub memory_bytes: u64,
    /// プロセスが存在するかどうか
    pub is_running: bool,
}

/// OBSプロセス固有のメトリクス
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ObsProcessMetrics {
    /// OBSメインプロセスのメトリクス
    pub main_process: Option<ProcessMetrics>,
    /// 合計CPU使用率
    pub total_cpu_usage: f32,
    /// 合計メモリ使用量（バイト）
    pub total_memory_bytes: u64,
}

// プロセス監視用のSystemインスタンス
// monitor/mod.rsのSYSTEMとは別に保持（プロセス更新は重いため）
static PROCESS_SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| {
    Mutex::new(System::new_all())
});

// OBSの実行ファイル名パターン
const OBS_PROCESS_NAMES: &[&str] = &[
    "obs64.exe",
    "obs32.exe",
    "obs",
    "obs-studio",
];

/// プロセス名がOBSかどうかを判定
fn is_obs_process(name: &str) -> bool {
    let lower_name = name.to_lowercase();
    OBS_PROCESS_NAMES.iter().any(|pattern| lower_name.contains(pattern))
}

/// 指定プロセス名のメトリクスを取得
pub fn get_process_by_name(process_name: &str) -> Result<Option<ProcessMetrics>, AppError> {
    let mut sys = PROCESS_SYSTEM.lock()
        .map_err(|e| AppError::system_monitor(&format!("Failed to lock process system: {}", e)))?;

    sys.refresh_processes();

    for (pid, process) in sys.processes() {
        if process.name().to_lowercase().contains(&process_name.to_lowercase()) {
            return Ok(Some(ProcessMetrics {
                name: process.name().to_string(),
                pid: pid.as_u32(),
                cpu_usage: process.cpu_usage(),
                memory_bytes: process.memory(),
                is_running: true,
            }));
        }
    }

    Ok(None)
}

/// OBSプロセスのメトリクスを取得
pub fn get_obs_process_metrics() -> Result<ObsProcessMetrics, AppError> {
    let mut sys = PROCESS_SYSTEM.lock()
        .map_err(|e| AppError::system_monitor(&format!("Failed to lock process system: {}", e)))?;

    sys.refresh_processes();

    let mut main_process: Option<ProcessMetrics> = None;
    let mut total_cpu = 0.0f32;
    let mut total_memory = 0u64;

    for (pid, process) in sys.processes() {
        let name = process.name().to_string();

        if is_obs_process(&name) {
            let cpu = process.cpu_usage();
            let memory = process.memory();

            total_cpu += cpu;
            total_memory = total_memory.saturating_add(memory);

            // メインプロセス（最もメモリを使用しているもの）を記録
            if main_process.is_none() ||
               main_process.as_ref().map_or(0, |p| p.memory_bytes) < memory {
                main_process = Some(ProcessMetrics {
                    name: name.clone(),
                    pid: pid.as_u32(),
                    cpu_usage: cpu,
                    memory_bytes: memory,
                    is_running: true,
                });
            }
        }
    }

    Ok(ObsProcessMetrics {
        main_process,
        total_cpu_usage: total_cpu,
        total_memory_bytes: total_memory,
    })
}

/// 全プロセスの中からCPU使用率上位N件を取得
#[allow(dead_code)]
pub fn get_top_processes_by_cpu(limit: usize) -> Result<Vec<ProcessMetrics>, AppError> {
    let mut sys = PROCESS_SYSTEM.lock()
        .map_err(|e| AppError::system_monitor(&format!("Failed to lock process system: {}", e)))?;

    sys.refresh_processes();

    let mut processes: Vec<ProcessMetrics> = sys.processes()
        .iter()
        .map(|(pid, process)| ProcessMetrics {
            name: process.name().to_string(),
            pid: pid.as_u32(),
            cpu_usage: process.cpu_usage(),
            memory_bytes: process.memory(),
            is_running: true,
        })
        .collect();

    // CPU使用率でソート（降順）
    // partial_cmp: NaN/Infinityの場合はEqualとして扱う（それらの値は出現しない想定）
    processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));

    Ok(processes.into_iter().take(limit).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_obs_process() {
        assert!(is_obs_process("obs64.exe"));
        assert!(is_obs_process("OBS64.EXE"));
        assert!(is_obs_process("obs-studio"));
        assert!(!is_obs_process("chrome.exe"));
        assert!(!is_obs_process("notepad"));
    }

    #[test]
    fn test_get_obs_process_metrics_returns_valid_struct() {
        let result = get_obs_process_metrics();
        assert!(result.is_ok());

        let metrics = result.unwrap();
        // OBSが実行されていなければmain_processはNone
        // 値の妥当性のみチェック
        assert!(metrics.total_cpu_usage >= 0.0);
    }

    #[test]
    fn test_get_top_processes_by_cpu() {
        let result = get_top_processes_by_cpu(5);
        assert!(result.is_ok());

        let processes = result.unwrap();
        assert!(processes.len() <= 5);

        // CPUでソートされていることを確認
        for i in 1..processes.len() {
            assert!(processes[i - 1].cpu_usage >= processes[i].cpu_usage);
        }
    }

    #[test]
    fn test_get_process_by_name_nonexistent() {
        let result = get_process_by_name("nonexistent_process_12345");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
