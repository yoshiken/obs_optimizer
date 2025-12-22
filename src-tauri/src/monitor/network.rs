// ネットワーク監視モジュール
//
// sysinfo クレートを使用してネットワーク帯域を取得

use serde::Serialize;
use sysinfo::Networks;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;
use crate::error::AppError;

/// ネットワーク使用状況のメトリクス
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NetworkMetrics {
    /// アップロード速度（バイト/秒）
    pub upload_bytes_per_sec: u64,
    /// ダウンロード速度（バイト/秒）
    pub download_bytes_per_sec: u64,
}

/// 前回のネットワーク統計を保持する構造体
struct NetworkState {
    networks: Networks,
    last_update: Instant,
    last_rx_total: u64,
    last_tx_total: u64,
}

impl NetworkState {
    fn new() -> Self {
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh();

        let (rx_total, tx_total) = Self::calculate_totals(&networks);

        Self {
            networks,
            last_update: Instant::now(),
            last_rx_total: rx_total,
            last_tx_total: tx_total,
        }
    }

    fn calculate_totals(networks: &Networks) -> (u64, u64) {
        let mut rx_total = 0u64;
        let mut tx_total = 0u64;

        for (_name, data) in networks.iter() {
            rx_total = rx_total.saturating_add(data.total_received());
            tx_total = tx_total.saturating_add(data.total_transmitted());
        }

        (rx_total, tx_total)
    }

    /// ネットワーク速度を計算（バイト/秒）
    fn get_speeds(&mut self) -> (u64, u64) {
        self.networks.refresh();

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);

        // 最低100ms経過していないと計算が不正確
        if elapsed < Duration::from_millis(100) {
            return (0, 0);
        }

        let (rx_total, tx_total) = Self::calculate_totals(&self.networks);

        let elapsed_secs = elapsed.as_secs_f64();

        // 差分から速度を計算
        let rx_diff = rx_total.saturating_sub(self.last_rx_total);
        let tx_diff = tx_total.saturating_sub(self.last_tx_total);

        let download_speed = (rx_diff as f64 / elapsed_secs) as u64;
        let upload_speed = (tx_diff as f64 / elapsed_secs) as u64;

        // 状態を更新
        self.last_update = now;
        self.last_rx_total = rx_total;
        self.last_tx_total = tx_total;

        (download_speed, upload_speed)
    }
}

// グローバルなネットワーク状態インスタンス
static NETWORK_STATE: Lazy<Mutex<NetworkState>> = Lazy::new(|| {
    Mutex::new(NetworkState::new())
});

/// ネットワーク使用状況を取得
///
/// 前回の呼び出しからの差分を計算して速度を算出する
/// 最初の呼び出しでは0を返す可能性がある
pub fn get_network_metrics() -> Result<NetworkMetrics, AppError> {
    let mut state = NETWORK_STATE.lock()
        .map_err(|e| AppError::system_monitor(&format!("Failed to lock network state: {}", e)))?;

    let (download_speed, upload_speed) = state.get_speeds();

    Ok(NetworkMetrics {
        upload_bytes_per_sec: upload_speed,
        download_bytes_per_sec: download_speed,
    })
}

/// ネットワークインターフェース名のリストを取得
#[allow(dead_code)]
pub fn get_network_interfaces() -> Result<Vec<String>, AppError> {
    let state = NETWORK_STATE.lock()
        .map_err(|e| AppError::system_monitor(&format!("Failed to lock network state: {}", e)))?;

    let names: Vec<String> = state.networks.iter()
        .map(|(name, _)| name.to_string())
        .collect();

    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_get_network_metrics_returns_valid_struct() {
        let result = get_network_metrics();
        assert!(result.is_ok());

        let metrics = result.unwrap();
        // 速度は非負
        assert!(metrics.upload_bytes_per_sec < u64::MAX);
        assert!(metrics.download_bytes_per_sec < u64::MAX);
    }

    #[test]
    fn test_multiple_calls_calculate_speed() {
        // 最初の呼び出し（ベースライン確立）
        let _ = get_network_metrics();

        // 少し待機
        sleep(Duration::from_millis(150));

        // 2回目の呼び出しで速度が計算される
        let result = get_network_metrics();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_network_interfaces() {
        let result = get_network_interfaces();
        assert!(result.is_ok());
        // 少なくとも1つのインターフェースがあるはず（lo, eth0など）
        // ただしCI環境では0の可能性もある
    }
}
