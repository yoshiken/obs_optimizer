// システム監視サービス - ビジネスロジック層
//
// CPU、メモリ、GPU、ネットワーク、プロセス監視のサービスレイヤー。
// Tauriコマンドから呼び出され、monitorモジュールを利用する。
//
// 設計方針:
// - 既存のmonitorモジュールの関数を統一的なAPIで提供
// - エラーハンドリングとバリデーションを一元化
// - 将来的なキャッシング、レート制限のフックポイントを提供

use crate::error::AppError;
use crate::monitor::{self, GpuMetrics, NetworkMetrics, ObsProcessMetrics};

/// システム監視サービスのインスタンス
///
/// `グローバルなsysinfo::Systemへのアクセスを提供する薄いラッパー`。
/// monitorモジュールが既にグローバルなMutex<System>を管理しているため、
/// このサービスはアクセスポイントとして機能する。
#[derive(Clone, Copy)]
pub struct SystemMonitorService;

impl Default for SystemMonitorService {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMonitorService {
    /// `新しいSystemMonitorServiceインスタンスを作成`
    ///
    /// このサービスはステートレスなので、複数回呼び出しても問題ない
    pub const fn new() -> Self {
        Self
    }

    /// CPU使用率を取得
    ///
    /// # Returns
    /// CPU使用率（0-100%）
    pub fn get_cpu_usage(&self) -> Result<f32, AppError> {
        monitor::get_cpu_usage()
    }

    /// メモリ情報を取得
    ///
    /// # Returns
    /// (使用中メモリ, 総メモリ) のタプル（バイト単位）
    pub fn get_memory_info(&self) -> Result<(u64, u64), AppError> {
        monitor::get_memory_info()
    }

    /// CPUコア数を取得
    ///
    /// # Returns
    /// 論理CPUコア数
    pub fn get_cpu_core_count(&self) -> Result<usize, AppError> {
        monitor::get_cpu_core_count()
    }

    /// 各CPUコアの使用率を取得
    ///
    /// # Returns
    /// 各コアの使用率（0-100%）の配列
    pub fn get_per_core_cpu_usage(&self) -> Result<Vec<f32>, AppError> {
        monitor::get_per_core_cpu_usage()
    }

    /// 利用可能なメモリを取得
    ///
    /// # Returns
    /// 利用可能なメモリ量（バイト単位）
    pub fn get_available_memory(&self) -> Result<u64, AppError> {
        monitor::get_available_memory()
    }

    /// GPUメトリクスを取得
    ///
    /// # Returns
    /// GPU情報（取得できない場合はNone）
    pub fn get_gpu_metrics(&self) -> Result<Option<GpuMetrics>, AppError> {
        monitor::gpu::get_gpu_metrics()
    }

    /// ネットワークメトリクスを取得
    ///
    /// # Returns
    /// ネットワーク情報（受信/送信バイト数、パケット数など）
    pub fn get_network_metrics(&self) -> Result<NetworkMetrics, AppError> {
        monitor::network::get_network_metrics()
    }

    /// OBSプロセスのメトリクスを取得
    ///
    /// # Returns
    /// OBSプロセスの情報（CPU使用率、メモリ使用量など）
    pub fn get_obs_process_metrics(&self) -> Result<ObsProcessMetrics, AppError> {
        monitor::process::get_obs_process_metrics()
    }

    /// 包括的なシステムメトリクスを取得（将来使用予定）
    ///
    /// CPU、メモリ、GPU、ネットワークの全情報を一度に取得する
    ///
    /// # Returns
    /// システム全体のメトリクス
    #[allow(dead_code)]
    pub fn get_all_metrics(&self) -> Result<SystemMetrics, AppError> {
        // CPU情報を収集
        let cpu_usage = self.get_cpu_usage()?;
        let core_count = self.get_cpu_core_count()?;
        let per_core_usage = self.get_per_core_cpu_usage()?;

        let cpu = CpuMetrics {
            usage_percent: cpu_usage,
            core_count,
            per_core_usage,
        };

        // メモリ情報を収集
        let (memory_used, memory_total) = self.get_memory_info()?;
        let memory_available = self.get_available_memory()?;
        let memory_usage_percent = if memory_total > 0 {
            (memory_used as f64 / memory_total as f64 * 100.0) as f32
        } else {
            0.0
        };

        let memory = MemoryMetrics {
            total_bytes: memory_total,
            used_bytes: memory_used,
            available_bytes: memory_available,
            usage_percent: memory_usage_percent.clamp(0.0, 100.0),
        };

        // GPU情報を収集（取得できない場合はNone）
        let gpu = self.get_gpu_metrics()?;

        // ネットワーク情報を収集
        let network = self.get_network_metrics()?;

        Ok(SystemMetrics {
            cpu,
            memory,
            gpu,
            network,
        })
    }
}

// ========================================
// 型定義（将来使用予定 - commands/system.rsと共通化すべき）
// ========================================

/// CPU使用状況のメトリクス（将来使用予定）
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuMetrics {
    /// 平均CPU使用率（0-100%）
    pub usage_percent: f32,
    /// CPUコア数
    pub core_count: usize,
    /// 各コアの使用率
    pub per_core_usage: Vec<f32>,
}

/// メモリ使用状況のメトリクス（将来使用予定）
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize)]
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

/// システム全体のメトリクス（将来使用予定）
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize)]
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

/// `グローバルなSystemMonitorServiceインスタンスを取得`
///
/// このサービスはステートレスなので、常に新しいインスタンスを返す
///
/// # Returns
/// `SystemMonitorServiceインスタンス`
pub const fn system_monitor_service() -> SystemMonitorService {
    SystemMonitorService::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_monitor_service_new() {
        let service = SystemMonitorService::new();
        let _ = service; // コンパイル確認
    }

    #[test]
    fn test_get_cpu_usage() {
        let service = system_monitor_service();
        let result = service.get_cpu_usage();
        assert!(result.is_ok());
        let usage = result.unwrap();
        assert!(usage >= 0.0 && usage <= 100.0);
    }

    #[test]
    fn test_get_memory_info() {
        let service = system_monitor_service();
        let result = service.get_memory_info();
        assert!(result.is_ok());
        let (used, total) = result.unwrap();
        assert!(used <= total);
    }

    #[test]
    fn test_get_cpu_core_count() {
        let service = system_monitor_service();
        let result = service.get_cpu_core_count();
        assert!(result.is_ok());
        let cores = result.unwrap();
        assert!(cores > 0);
    }

    #[test]
    fn test_get_per_core_cpu_usage() {
        let service = system_monitor_service();
        let result = service.get_per_core_cpu_usage();
        assert!(result.is_ok());
        let usage = result.unwrap();
        assert!(!usage.is_empty());
        for cpu in usage {
            assert!(cpu >= 0.0 && cpu <= 100.0);
        }
    }

    #[test]
    fn test_get_all_metrics() {
        let service = system_monitor_service();
        let result = service.get_all_metrics();
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.cpu.usage_percent >= 0.0);
        assert!(metrics.memory.total_bytes > 0);
    }
}
