// OBS WebSocket関連のエラーマッピング
//
// obwsクレートのエラーをAppErrorに変換するための実装

use crate::error::AppError;

/// OBS関連のエラーコード
pub mod error_codes {
    /// 認証エラー
    pub const OBS_AUTH: &str = "OBS_AUTH";
    /// 通信エラー
    pub const OBS_COMMUNICATION: &str = "OBS_COMMUNICATION";
    /// タイムアウトエラー
    pub const OBS_TIMEOUT: &str = "OBS_TIMEOUT";
    /// 状態エラー (未接続時のコマンド実行など)
    pub const OBS_STATE: &str = "OBS_STATE";
    /// OBSからのリクエストエラー
    pub const OBS_REQUEST: &str = "OBS_REQUEST";
    /// シリアライズ/デシリアライズエラー
    pub const OBS_SERIALIZATION: &str = "OBS_SERIALIZATION";
    /// バージョン互換性エラー
    pub const OBS_VERSION: &str = "OBS_VERSION";
    /// 不明なエラー
    pub const OBS_UNKNOWN: &str = "OBS_UNKNOWN";
}

/// `OBS固有のAppErrorファクトリ関数`
impl AppError {
    /// 認証エラーを作成
    pub fn obs_auth(msg: &str) -> Self {
        Self::new(error_codes::OBS_AUTH, msg)
    }

    /// 通信エラーを作成
    pub fn obs_communication(msg: &str) -> Self {
        Self::new(error_codes::OBS_COMMUNICATION, msg)
    }

    /// タイムアウトエラーを作成
    pub fn obs_timeout(msg: &str) -> Self {
        Self::new(error_codes::OBS_TIMEOUT, msg)
    }

    /// 状態エラーを作成
    pub fn obs_state(msg: &str) -> Self {
        Self::new(error_codes::OBS_STATE, msg)
    }

    /// リクエストエラーを作成
    pub fn obs_request(msg: &str) -> Self {
        Self::new(error_codes::OBS_REQUEST, msg)
    }

    /// シリアライズエラーを作成
    pub fn obs_serialization(msg: &str) -> Self {
        Self::new(error_codes::OBS_SERIALIZATION, msg)
    }

    /// バージョンエラーを作成
    pub fn obs_version(msg: &str) -> Self {
        Self::new(error_codes::OBS_VERSION, msg)
    }
}

/// obwsのエラーをAppErrorに変換
impl From<obws::error::Error> for AppError {
    fn from(err: obws::error::Error) -> Self {
        // エラーメッセージから種類を推定してマッピング
        let err_str = err.to_string();
        let err_lower = err_str.to_lowercase();

        if err_lower.contains("timeout") {
            Self::obs_timeout(&err_str)
        } else if err_lower.contains("connect") || err_lower.contains("connection") {
            Self::obs_connection(&err_str)
        } else if err_lower.contains("auth") || err_lower.contains("password") || err_lower.contains("handshake") {
            Self::obs_auth(&err_str)
        } else if err_lower.contains("version") {
            Self::obs_version(&err_str)
        } else if err_lower.contains("serialize") || err_lower.contains("deserialize") {
            Self::obs_serialization(&err_str)
        } else if err_lower.contains("disconnect") {
            Self::obs_state(&err_str)
        } else {
            Self::new(error_codes::OBS_UNKNOWN, &err_str)
        }
    }
}

/// OBS操作の結果型エイリアス
pub type ObsResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_constants() {
        assert_eq!(error_codes::OBS_CONNECTION, "OBS_CONNECTION");
        assert_eq!(error_codes::OBS_AUTH, "OBS_AUTH");
        assert_eq!(error_codes::OBS_COMMUNICATION, "OBS_COMMUNICATION");
        assert_eq!(error_codes::OBS_VERSION, "OBS_VERSION");
    }

    #[test]
    fn test_app_error_factory_methods() {
        let auth_error = AppError::obs_auth("認証に失敗しました");
        assert_eq!(auth_error.code(), error_codes::OBS_AUTH);
        assert!(auth_error.message().contains("認証に失敗しました"));

        let timeout_error = AppError::obs_timeout("接続がタイムアウトしました");
        assert_eq!(timeout_error.code(), error_codes::OBS_TIMEOUT);

        let state_error = AppError::obs_state("接続されていません");
        assert_eq!(state_error.code(), error_codes::OBS_STATE);

        let version_error = AppError::obs_version("バージョン不一致");
        assert_eq!(version_error.code(), error_codes::OBS_VERSION);
    }
}
