#[cfg(test)]
mod tests {
    use super::super::{
        get_cpu_usage,
        get_memory_info,
        get_cpu_core_count,
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
            let _ = get_per_core_cpu_usage();
            let _ = get_available_memory();
        }
        // パニックしなければOK
    }
}
