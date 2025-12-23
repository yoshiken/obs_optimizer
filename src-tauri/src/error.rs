use serde::Serialize;

/// エラーコード定数
pub const ERROR_CODE_OBS_CONNECTION: &str = "OBS_CONNECTION";
#[allow(dead_code)]
pub const ERROR_CODE_OBS_STATE: &str = "OBS_STATE";
pub const ERROR_CODE_SYSTEM_MONITOR: &str = "SYSTEM_MONITOR";
pub const ERROR_CODE_IO: &str = "IO_ERROR";
pub const ERROR_CODE_JSON: &str = "JSON_ERROR";
pub const ERROR_CODE_TRAY: &str = "TRAY_ERROR";
pub const ERROR_CODE_WINDOW: &str = "WINDOW_ERROR";
pub const ERROR_CODE_CONFIG: &str = "CONFIG_ERROR";
pub const ERROR_CODE_DATABASE: &str = "DATABASE_ERROR";
pub const ERROR_CODE_EXPORT: &str = "EXPORT_ERROR";
pub const ERROR_CODE_ANALYZER: &str = "ANALYZER_ERROR";

/// アプリケーション全体で使用するエラー型
///
/// Tauri コマンドからフロントエンドに返されるエラーは
/// この型にシリアライズされる
#[derive(Debug, Serialize)]
pub struct AppError {
    code: String,
    message: String,
}

impl AppError {
    /// 新しいエラーを作成
    ///
    /// # Arguments
    /// * `code` - エラーコード(例: "`OBS_CONNECTION`")
    /// * `message` - エラーメッセージ
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
        }
    }

    /// エラーコードを取得
    pub fn code(&self) -> &str {
        &self.code
    }

    /// エラーメッセージを取得
    pub fn message(&self) -> &str {
        &self.message
    }

    /// OBS接続関連のエラーを作成
    pub fn obs_connection(msg: &str) -> Self {
        Self::new(ERROR_CODE_OBS_CONNECTION, msg)
    }

    // obs_state は obs/error.rs で定義されている

    /// システム監視関連のエラーを作成
    pub fn system_monitor(msg: &str) -> Self {
        Self::new(ERROR_CODE_SYSTEM_MONITOR, msg)
    }

    /// システムトレイ関連のエラーを作成
    pub fn tray_error(msg: &str) -> Self {
        Self::new(ERROR_CODE_TRAY, msg)
    }

    /// ウィンドウ操作関連のエラーを作成
    pub fn window_error(msg: &str) -> Self {
        Self::new(ERROR_CODE_WINDOW, msg)
    }

    /// 設定関連のエラーを作成
    pub fn config_error(msg: &str) -> Self {
        Self::new(ERROR_CODE_CONFIG, msg)
    }

    /// データベース関連のエラーを作成
    pub fn database_error(msg: &str) -> Self {
        Self::new(ERROR_CODE_DATABASE, msg)
    }

    /// エクスポート関連のエラーを作成
    pub fn export_error(msg: &str) -> Self {
        Self::new(ERROR_CODE_EXPORT, msg)
    }

    /// 分析関連のエラーを作成
    pub fn analyzer_error(msg: &str) -> Self {
        Self::new(ERROR_CODE_ANALYZER, msg)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

// std::io::Error からの変換
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::new(ERROR_CODE_IO, &err.to_string())
    }
}

// serde_json::Error からの変換
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::new(ERROR_CODE_JSON, &err.to_string())
    }
}

// nvml_wrapper::error::NvmlError からの変換
impl From<nvml_wrapper::error::NvmlError> for AppError {
    fn from(err: nvml_wrapper::error::NvmlError) -> Self {
        use nvml_wrapper::error::NvmlError;

        match err {
            NvmlError::DriverNotLoaded => {
                Self::system_monitor("NVIDIA driver not loaded")
            }
            NvmlError::NoPermission => {
                Self::system_monitor("No permission to access GPU")
            }
            NvmlError::NotSupported => {
                Self::system_monitor("Operation not supported on this GPU")
            }
            NvmlError::InvalidArg => {
                Self::system_monitor("Invalid argument to NVML")
            }
            _ => {
                Self::system_monitor(&format!("NVML error: {err:?}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_new() {
        let error = AppError::new("TEST_CODE", "Test message");
        assert_eq!(error.code(), "TEST_CODE");
        assert_eq!(error.message(), "Test message");
    }

    #[test]
    fn test_obs_connection_error() {
        let error = AppError::obs_connection("Connection failed");
        assert_eq!(error.code(), ERROR_CODE_OBS_CONNECTION);
        assert_eq!(error.message(), "Connection failed");
    }

    #[test]
    fn test_system_monitor_error() {
        let error = AppError::system_monitor("Monitor error");
        assert_eq!(error.code(), ERROR_CODE_SYSTEM_MONITOR);
        assert_eq!(error.message(), "Monitor error");
    }

    #[test]
    fn test_tray_error() {
        let error = AppError::tray_error("Tray error");
        assert_eq!(error.code(), ERROR_CODE_TRAY);
        assert_eq!(error.message(), "Tray error");
    }

    #[test]
    fn test_window_error() {
        let error = AppError::window_error("Window error");
        assert_eq!(error.code(), ERROR_CODE_WINDOW);
        assert_eq!(error.message(), "Window error");
    }

    #[test]
    fn test_config_error() {
        let error = AppError::config_error("Config error");
        assert_eq!(error.code(), ERROR_CODE_CONFIG);
        assert_eq!(error.message(), "Config error");
    }

    #[test]
    fn test_database_error() {
        let error = AppError::database_error("Database error");
        assert_eq!(error.code(), ERROR_CODE_DATABASE);
        assert_eq!(error.message(), "Database error");
    }

    #[test]
    fn test_export_error() {
        let error = AppError::export_error("Export error");
        assert_eq!(error.code(), ERROR_CODE_EXPORT);
        assert_eq!(error.message(), "Export error");
    }

    #[test]
    fn test_analyzer_error() {
        let error = AppError::analyzer_error("Analyzer error");
        assert_eq!(error.code(), ERROR_CODE_ANALYZER);
        assert_eq!(error.message(), "Analyzer error");
    }

    #[test]
    fn test_error_display() {
        let error = AppError::new("CODE", "message");
        let display = format!("{error}");
        assert_eq!(display, "[CODE] message");
    }

    #[test]
    fn test_error_debug() {
        let error = AppError::new("CODE", "message");
        let debug = format!("{error:?}");
        assert!(debug.contains("CODE"));
        assert!(debug.contains("message"));
    }

    #[test]
    fn test_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let app_error = AppError::from(io_error);
        assert_eq!(app_error.code(), ERROR_CODE_IO);
        assert!(app_error.message().contains("File not found"));
    }

    #[test]
    fn test_from_json_error() {
        let json_str = r#"{"invalid": json"#;
        let json_error = serde_json::from_str::<serde_json::Value>(json_str)
            .expect_err("should be error");
        let app_error = AppError::from(json_error);
        assert_eq!(app_error.code(), ERROR_CODE_JSON);
    }

    #[test]
    fn test_error_serialization() {
        let error = AppError::new("TEST", "test message");
        let json = serde_json::to_string(&error).expect("serialization failed");
        assert!(json.contains("TEST"));
        assert!(json.contains("test message"));
    }

    #[test]
    fn test_nvml_driver_not_loaded() {
        let nvml_error = nvml_wrapper::error::NvmlError::DriverNotLoaded;
        let app_error = AppError::from(nvml_error);
        assert_eq!(app_error.code(), ERROR_CODE_SYSTEM_MONITOR);
        assert!(app_error.message().contains("driver"));
    }

    #[test]
    fn test_nvml_no_permission() {
        let nvml_error = nvml_wrapper::error::NvmlError::NoPermission;
        let app_error = AppError::from(nvml_error);
        assert_eq!(app_error.code(), ERROR_CODE_SYSTEM_MONITOR);
        assert!(app_error.message().contains("permission"));
    }

    #[test]
    fn test_nvml_not_supported() {
        let nvml_error = nvml_wrapper::error::NvmlError::NotSupported;
        let app_error = AppError::from(nvml_error);
        assert_eq!(app_error.code(), ERROR_CODE_SYSTEM_MONITOR);
        assert!(app_error.message().contains("not supported"));
    }

    #[test]
    fn test_nvml_invalid_arg() {
        let nvml_error = nvml_wrapper::error::NvmlError::InvalidArg;
        let app_error = AppError::from(nvml_error);
        assert_eq!(app_error.code(), ERROR_CODE_SYSTEM_MONITOR);
        assert!(app_error.message().contains("Invalid argument"));
    }

    #[test]
    fn test_error_code_constants() {
        assert_eq!(ERROR_CODE_OBS_CONNECTION, "OBS_CONNECTION");
        assert_eq!(ERROR_CODE_OBS_STATE, "OBS_STATE");
        assert_eq!(ERROR_CODE_SYSTEM_MONITOR, "SYSTEM_MONITOR");
        assert_eq!(ERROR_CODE_IO, "IO_ERROR");
        assert_eq!(ERROR_CODE_JSON, "JSON_ERROR");
        assert_eq!(ERROR_CODE_TRAY, "TRAY_ERROR");
        assert_eq!(ERROR_CODE_WINDOW, "WINDOW_ERROR");
        assert_eq!(ERROR_CODE_CONFIG, "CONFIG_ERROR");
        assert_eq!(ERROR_CODE_DATABASE, "DATABASE_ERROR");
        assert_eq!(ERROR_CODE_EXPORT, "EXPORT_ERROR");
        assert_eq!(ERROR_CODE_ANALYZER, "ANALYZER_ERROR");
    }

    #[test]
    fn test_error_implements_std_error() {
        let error = AppError::new("CODE", "message");
        // std::error::Error traitを実装していることを確認
        let _: &dyn std::error::Error = &error;
    }
}
