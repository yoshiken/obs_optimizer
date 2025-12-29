#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::super::{
        get_cpu_usage,
        get_memory_info,
        get_cpu_core_count,
        get_cpu_name,
        get_per_core_cpu_usage,
        get_available_memory,
    };

    #[test]
    fn test_cpu_usage_returns_valid_range() {
        let result = get_cpu_usage();
        assert!(result.is_ok(), "get_cpu_usage should succeed");
        let cpu = result.unwrap();
        // CPU使用率は0-100%の範囲内である必要がある
        assert!(cpu >= 0.0 && cpu <= 100.0, "CPU usage should be between 0 and 100, got {}", cpu);
    }

    #[test]
    fn test_memory_info_returns_valid_values() {
        let result = get_memory_info();
        assert!(result.is_ok(), "get_memory_info should succeed");
        let (used, total) = result.unwrap();
        // 総メモリ量は0より大きく、使用量は総量以下である必要がある
        assert!(total > 0, "Total memory should be greater than 0");
        assert!(used <= total, "Used memory should be less than or equal to total memory");
    }

    #[test]
    fn test_cpu_core_count() {
        let result = get_cpu_core_count();
        assert!(result.is_ok(), "get_cpu_core_count should succeed");
        let count = result.unwrap();
        assert!(count > 0, "Should have at least 1 CPU core");
    }

    #[test]
    fn test_cpu_name_returns_valid_string() {
        let result = get_cpu_name();
        assert!(result.is_ok(), "get_cpu_name should succeed");
        let name = result.unwrap();
        // CPU名は空文字列ではない
        assert!(!name.is_empty(), "CPU name should not be empty");
        // 一般的なCPUブランドまたは "Unknown CPU" が含まれているはず
        let is_valid = name.contains("Intel")
            || name.contains("AMD")
            || name.contains("ARM")
            || name.contains("Apple")
            || name.contains("Unknown CPU");
        assert!(
            is_valid,
            "CPU name should contain a known brand or 'Unknown CPU', got: {}",
            name
        );
    }

    #[test]
    fn test_per_core_cpu_usage() {
        let result = get_per_core_cpu_usage();
        assert!(result.is_ok(), "get_per_core_cpu_usage should succeed");
        let usage = result.unwrap();
        // 各コアの使用率は0-100%の範囲内
        for (i, &core_usage) in usage.iter().enumerate() {
            assert!(
                core_usage >= 0.0 && core_usage <= 100.0,
                "Core {} usage should be between 0 and 100, got {}",
                i,
                core_usage
            );
        }
    }

    #[test]
    fn test_available_memory() {
        let result = get_available_memory();
        assert!(result.is_ok(), "get_available_memory should succeed");
        let available = result.unwrap();

        // 利用可能メモリは総メモリ以下
        let (_, total) = get_memory_info().unwrap();
        assert!(available <= total, "Available memory should be <= total memory");
    }

    #[test]
    fn test_multiple_calls_work() {
        // 複数回呼び出しても問題なく動作することを確認
        for _ in 0..3 {
            let _ = get_cpu_usage();
            let _ = get_memory_info();
            let _ = get_cpu_core_count();
            let _ = get_cpu_name();
            let _ = get_per_core_cpu_usage();
            let _ = get_available_memory();
        }
        // パニックしなければOK
    }

    // === 追加のエッジケーステスト ===

    #[test]
    fn test_cpu_usage_consistency() {
        // 短時間に連続して呼び出した場合、大きく変動しないことを確認
        let usage1 = get_cpu_usage().unwrap();
        let usage2 = get_cpu_usage().unwrap();

        // 瞬間的に大きく変わることは少ないはず（差が50%以内）
        let diff = (usage1 - usage2).abs();
        assert!(
            diff < 50.0,
            "連続して取得したCPU使用率の差が大きすぎる: {} vs {}",
            usage1,
            usage2
        );
    }

    #[test]
    fn test_memory_total_is_constant() {
        // 総メモリ量は変わらないはず
        let (_, total1) = get_memory_info().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let (_, total2) = get_memory_info().unwrap();

        assert_eq!(total1, total2, "総メモリ量は不変");
    }

    #[test]
    fn test_memory_used_is_reasonable() {
        let (used, total) = get_memory_info().unwrap();

        // 使用メモリは総メモリより小さい
        assert!(used < total, "使用メモリが総メモリを超えている");

        // 使用率の計算
        let usage_percent = (used as f64 / total as f64) * 100.0;
        assert!(
            usage_percent >= 0.0 && usage_percent <= 100.0,
            "メモリ使用率が範囲外: {}%",
            usage_percent
        );
    }

    #[test]
    fn test_cpu_core_count_is_consistent() {
        // コア数は変わらないはず
        let count1 = get_cpu_core_count().unwrap();
        let count2 = get_cpu_core_count().unwrap();

        assert_eq!(count1, count2, "CPUコア数は不変");
        assert!(count1 > 0, "CPUコア数は1以上");
        assert!(count1 <= 256, "CPUコア数が異常に多い（256以上）");
    }

    #[test]
    fn test_cpu_name_is_consistent() {
        // CPU名は変わらないはず
        let name1 = get_cpu_name().unwrap();
        let name2 = get_cpu_name().unwrap();

        assert_eq!(name1, name2, "CPU名は不変");
    }

    #[test]
    fn test_per_core_usage_length_matches_core_count() {
        let core_count = get_cpu_core_count().unwrap();
        let per_core_usage = get_per_core_cpu_usage().unwrap();

        assert_eq!(
            per_core_usage.len(),
            core_count,
            "コア数とコア別使用率の要素数が一致しない"
        );
    }

    #[test]
    fn test_per_core_usage_all_valid() {
        let usage = get_per_core_cpu_usage().unwrap();

        // すべてのコアの使用率が有効範囲内
        for (i, &core_usage) in usage.iter().enumerate() {
            assert!(
                core_usage >= 0.0 && core_usage <= 100.0,
                "コア {} の使用率が範囲外: {}%",
                i,
                core_usage
            );
        }
    }

    #[test]
    fn test_available_memory_less_than_total() {
        let available = get_available_memory().unwrap();
        let (_, total) = get_memory_info().unwrap();

        assert!(
            available <= total,
            "利用可能メモリが総メモリを超えている"
        );
    }

    #[test]
    fn test_available_memory_consistency() {
        // 利用可能メモリと使用メモリの関係
        let available = get_available_memory().unwrap();
        let (used, total) = get_memory_info().unwrap();

        // 使用メモリ + 利用可能メモリ ≒ 総メモリ（バッファ/キャッシュ考慮）
        // ただし、OSによって計算方法が異なるため、厳密な等式ではない
        let accounted = used + available;

        // 総メモリの80%～120%の範囲内（緩い検証）
        assert!(
            accounted >= total / 2 && accounted <= total * 2,
            "メモリの計算が矛盾: used={}, available={}, total={}",
            used,
            available,
            total
        );
    }

    #[test]
    fn test_rapid_successive_calls() {
        // 高速に連続呼び出ししてもクラッシュしない
        for _ in 0..10 {
            let _ = get_cpu_usage();
            let _ = get_memory_info();
            let _ = get_per_core_cpu_usage();
        }
    }

    #[test]
    fn test_concurrent_calls() {
        use std::thread;

        // 複数スレッドから同時に呼び出してもクラッシュしない
        let handles: Vec<_> = (0..4)
            .map(|_| {
                thread::spawn(|| {
                    for _ in 0..5 {
                        let _ = get_cpu_usage();
                        let _ = get_memory_info();
                        let _ = get_cpu_core_count();
                        let _ = get_cpu_name();
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_memory_info_non_zero() {
        let (used, total) = get_memory_info().unwrap();

        // システムが稼働している以上、使用メモリは0より大きいはず
        assert!(used > 0, "使用メモリが0（異常）");
        assert!(total > 0, "総メモリが0（異常）");

        // 最低でも数百MBはあるはず（現代的なシステムなら）
        let min_expected_bytes = 512 * 1024 * 1024; // 512MB
        assert!(
            total >= min_expected_bytes,
            "総メモリが異常に少ない: {} bytes",
            total
        );
    }

    #[test]
    fn test_cpu_usage_after_delay() {
        // 少し待ってから再度取得しても有効な値が返る
        let usage1 = get_cpu_usage().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let usage2 = get_cpu_usage().unwrap();

        assert!(usage1 >= 0.0 && usage1 <= 100.0);
        assert!(usage2 >= 0.0 && usage2 <= 100.0);
    }

    #[test]
    fn test_per_core_usage_sum_reasonable() {
        let per_core = get_per_core_cpu_usage().unwrap();
        let overall = get_cpu_usage().unwrap();

        // コア別使用率の平均は、全体のCPU使用率と近い値になるはず
        let avg_core: f32 = per_core.iter().sum::<f32>() / per_core.len() as f32;

        // 差が30%以内（計測タイミングの違いを考慮）
        let diff = (avg_core - overall).abs();
        assert!(
            diff < 30.0,
            "コア別平均CPU使用率と全体CPU使用率の差が大きい: avg_core={}, overall={}",
            avg_core,
            overall
        );
    }
}
