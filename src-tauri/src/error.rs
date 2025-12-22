use serde::Serialize;

/// エラーコード定数
pub const ERROR_CODE_OBS_CONNECTION: &str = "OBS_CONNECTION";
pub const ERROR_CODE_SYSTEM_MONITOR: &str = "SYSTEM_MONITOR";
pub const ERROR_CODE_IO: &str = "IO_ERROR";
pub const ERROR_CODE_JSON: &str = "JSON_ERROR";
pub const ERROR_CODE_TRAY: &str = "TRAY_ERROR";
pub const ERROR_CODE_WINDOW: &str = "WINDOW_ERROR";

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
