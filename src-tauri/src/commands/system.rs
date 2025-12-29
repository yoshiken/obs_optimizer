use serde::Serialize;
use crate::error::AppError;
use crate::monitor::{GpuMetrics, NetworkMetrics, ObsProcessMetrics};
use crate::services::system_monitor_service;

// ========================================
// 型定義（contracts/api.md に準拠）
// ========================================

/// CPU使用状況のメトリクス
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuMetrics {
    /// 平均CPU使用率（0-100%）
    pub usage_percent: f32,
    /// CPUコア数
    pub core_count: usize,
    /// 各コアの使用率
    pub per_core_usage: Vec<f32>,
    /// CPUモデル名（ブランド名）
    pub cpu_name: String,
}

/// メモリ使用状況のメトリクス
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryMetrics {
    /// 総メモリ容量（バイト）
    pub total_bytes: u64,
    /// 使用中のメモリ（バイト）
    pub used_bytes: u64,
    /// 利用可能なメモリ（バイト）
    pub available_bytes: u64,
    /// メモリ使用率（0-100%）
    pub usage_percent: f32,
}

/// システム全体のメトリクス（契約準拠）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMetrics {
    /// CPU情報
    pub cpu: CpuMetrics,
    /// メモリ情報
    pub memory: MemoryMetrics,
    /// GPU情報（取得できない場合はnull）
    pub gpu: Option<GpuMetrics>,
    /// ネットワーク情報
    pub network: NetworkMetrics,
}

/// レガシー形式のシステムメトリクス（後方互換性用）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacySystemMetrics {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
}

// ========================================
// Tauriコマンド
// ========================================

/// システムメトリクスを取得（契約準拠の完全版）
///
/// CPU、メモリ、GPU、ネットワークの詳細情報を返す
#[tauri::command]
pub async fn get_system_metrics() -> Result<SystemMetrics, AppError> {
    // サービス層経由で各メトリクスを取得し、コマンド用の型に変換
    let service = system_monitor_service();

    let cpu_usage = service.get_cpu_usage()?;
    let core_count = service.get_cpu_core_count()?;
    let per_core_usage = service.get_per_core_cpu_usage()?;
    let cpu_name = service.get_cpu_name()?;

    let (memory_used, memory_total) = service.get_memory_info()?;
    let memory_available = service.get_available_memory()?;
    let memory_usage_percent = if memory_total > 0 {
        (memory_used as f64 / memory_total as f64 * 100.0) as f32
    } else {
        0.0
    };

    let gpu = service.get_gpu_metrics()?;
    let network = service.get_network_metrics()?;

    Ok(SystemMetrics {
        cpu: CpuMetrics {
            usage_percent: cpu_usage,
            core_count,
            per_core_usage,
            cpu_name,
        },
        memory: MemoryMetrics {
            total_bytes: memory_total,
            used_bytes: memory_used,
            available_bytes: memory_available,
            usage_percent: memory_usage_percent,
        },
        gpu,
        network,
    })
}

/// OBSプロセスのメトリクスを取得
#[tauri::command]
pub async fn get_process_metrics() -> Result<ObsProcessMetrics, AppError> {
    let service = system_monitor_service();
    service.get_obs_process_metrics()
}

/// レガシー形式のシステムメトリクスを取得（後方互換性用）
///
/// 既存のフロントエンドコードとの互換性を維持するために提供
#[tauri::command]
pub async fn get_legacy_system_metrics() -> Result<LegacySystemMetrics, AppError> {
    let service = system_monitor_service();
    let cpu_usage = service.get_cpu_usage()?;
    let (memory_used, memory_total) = service.get_memory_info()?;

    Ok(LegacySystemMetrics {
        cpu_usage,
        memory_used,
        memory_total,
    })
}
