// OBSパスワードのセキュアストレージ管理
//
// OSのキーリング（Windows Credential Manager、macOS Keychain、Linux Secret Service）
// を使用してパスワードを安全に保存する。
//
// プレーンテキスト設定ファイルからの移行もサポート。

use crate::error::AppError;

/// サービス名（キーリング登録用）
const SERVICE_NAME: &str = "obs-optimizer";

/// ユーザー名（キーリング登録用）
const USERNAME: &str = "obs_websocket";

/// キーリングエラーコード
pub const ERROR_CODE_KEYRING: &str = "KEYRING_ERROR";

/// キーリング関連のエラーを作成
fn keyring_error(msg: &str) -> AppError {
    AppError::new(ERROR_CODE_KEYRING, msg)
}

/// OBS WebSocketパスワードを安全に保存
///
/// OSのキーリング（Windows Credential Manager等）に保存する。
/// 既存のパスワードがある場合は上書きする。
///
/// # Arguments
/// * `password` - 保存するパスワード
///
/// # Returns
/// 成功時はOk(()), 失敗時はAppError
pub fn save_obs_password(password: &str) -> Result<(), AppError> {
    let entry = keyring::Entry::new(SERVICE_NAME, USERNAME)
        .map_err(|e| keyring_error(&format!("キーリングエントリの作成に失敗: {e}")))?;

    entry
        .set_password(password)
        .map_err(|e| keyring_error(&format!("パスワードの保存に失敗: {e}")))?;

    Ok(())
}

/// OBS WebSocketパスワードを取得
///
/// OSのキーリングからパスワードを取得する。
/// パスワードが保存されていない場合はNoneを返す。
///
/// # Returns
/// 保存されたパスワード（存在する場合）、またはNone
pub fn get_obs_password() -> Result<Option<String>, AppError> {
    let entry = keyring::Entry::new(SERVICE_NAME, USERNAME)
        .map_err(|e| keyring_error(&format!("キーリングエントリの作成に失敗: {e}")))?;

    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(keyring_error(&format!("パスワードの取得に失敗: {e}"))),
    }
}

/// OBS WebSocketパスワードを削除
///
/// OSのキーリングからパスワードを削除する。
/// パスワードが存在しない場合もエラーにはしない。
///
/// # Returns
/// 成功時はOk(()), 失敗時はAppError
pub fn delete_obs_password() -> Result<(), AppError> {
    let entry = keyring::Entry::new(SERVICE_NAME, USERNAME)
        .map_err(|e| keyring_error(&format!("キーリングエントリの作成に失敗: {e}")))?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        // パスワードが存在しない場合はエラーにしない
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(keyring_error(&format!("パスワードの削除に失敗: {e}"))),
    }
}

/// プレーンテキストからキーリングへの移行を試行
///
/// config.jsonに保存されたパスワードがある場合、キーリングに移行して
/// 設定ファイルからは削除する。
///
/// # Arguments
/// * `plaintext_password` - 設定ファイルから読み込んだパスワード
///
/// # Returns
/// 移行が成功した場合はOk(true)、パスワードがなかった場合はOk(false)
/// キーリングエラーの場合でも警告を出力してOk(false)を返す（クラッシュしない）
pub fn migrate_from_plaintext(plaintext_password: Option<&str>) -> Result<bool, AppError> {
    let Some(password) = plaintext_password else {
        return Ok(false);
    };

    if password.is_empty() {
        return Ok(false);
    }

    // キーリングへの保存を試行
    match save_obs_password(password) {
        Ok(()) => {
            tracing::info!(target: "credentials", "パスワードをキーリングに移行しました");
            Ok(true)
        },
        Err(e) => {
            // キーリングが利用できない場合は警告のみ（クラッシュしない）
            tracing::warn!(target: "credentials", error = %e, "キーリングへの移行に失敗");
            tracing::warn!(target: "credentials", "パスワードは設定ファイルに残ります");
            Ok(false)
        },
    }
}

/// キーリングが利用可能かチェック
///
/// テストやデバッグ用。実際の保存/取得を試みずに利用可能性を確認。
///
/// # Returns
/// キーリングが利用可能ならtrue
#[allow(dead_code)]
pub fn is_keyring_available() -> bool {
    keyring::Entry::new(SERVICE_NAME, USERNAME).is_ok()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // 注意: これらのテストは実際のOSキーリングを使用する
    // CI環境ではキーリングが利用できない場合がある

    /// テスト用のユニークなエントリを作成
    fn test_entry() -> keyring::Entry {
        // テストごとにユニークなユーザー名を使用してテスト間の干渉を防ぐ
        let test_user = format!("test_user_{}", std::process::id());
        keyring::Entry::new(SERVICE_NAME, &test_user).unwrap()
    }

    /// テスト後のクリーンアップ
    fn cleanup_test_entry(entry: &keyring::Entry) {
        let _ = entry.delete_credential();
    }

    #[test]
    fn test_keyring_entry_creation() {
        // キーリングエントリが作成できることを確認
        let result = keyring::Entry::new(SERVICE_NAME, USERNAME);
        assert!(result.is_ok(), "キーリングエントリが作成できること");
    }

    #[test]
    fn test_save_and_get_password() {
        let entry = test_entry();
        let test_password = "test_password_12345";

        // 保存
        let save_result = entry.set_password(test_password);
        if save_result.is_err() {
            // キーリングが利用できない環境（CI等）ではスキップ
            eprintln!("[SKIP] キーリングが利用できません");
            return;
        }

        // 取得
        let get_result = entry.get_password();
        assert!(get_result.is_ok(), "パスワードが取得できること");
        assert_eq!(get_result.unwrap(), test_password);

        // クリーンアップ
        cleanup_test_entry(&entry);
    }

    #[test]
    fn test_delete_password() {
        let entry = test_entry();
        let test_password = "test_password_to_delete";

        // 保存
        if entry.set_password(test_password).is_err() {
            eprintln!("[SKIP] キーリングが利用できません");
            return;
        }

        // 削除
        let delete_result = entry.delete_credential();
        assert!(delete_result.is_ok(), "パスワードが削除できること");

        // 削除後の取得
        let get_result = entry.get_password();
        assert!(
            matches!(get_result, Err(keyring::Error::NoEntry)),
            "削除後はNoEntryエラーになること"
        );
    }

    #[test]
    fn test_get_nonexistent_password() {
        // 存在しないパスワードを取得
        let entry = keyring::Entry::new(SERVICE_NAME, "nonexistent_user_12345").unwrap();

        let result = entry.get_password();
        assert!(
            matches!(result, Err(keyring::Error::NoEntry)),
            "存在しないパスワードはNoEntryエラーになること"
        );
    }

    #[test]
    fn test_save_obs_password_function() {
        // 関数インターフェースのテスト
        let result = save_obs_password("test_function_password");

        // キーリングが利用できない場合はスキップ
        if result.is_err() {
            let err_msg = result.as_ref().unwrap_err().message();
            if err_msg.contains("失敗") {
                eprintln!("[SKIP] キーリングが利用できません: {err_msg}");
                return;
            }
        }

        assert!(result.is_ok(), "save_obs_password が成功すること");

        // クリーンアップ
        let _ = delete_obs_password();
    }

    #[test]
    fn test_get_obs_password_function() {
        // まず保存を試みる
        if save_obs_password("test_get_password").is_err() {
            eprintln!("[SKIP] キーリングが利用できません");
            return;
        }

        let result = get_obs_password();
        assert!(result.is_ok(), "get_obs_password が成功すること");
        assert_eq!(result.unwrap(), Some("test_get_password".to_string()));

        // クリーンアップ
        let _ = delete_obs_password();
    }

    #[test]
    fn test_delete_obs_password_function() {
        // まず保存
        if save_obs_password("test_delete_password").is_err() {
            eprintln!("[SKIP] キーリングが利用できません");
            return;
        }

        // 削除
        let delete_result = delete_obs_password();
        assert!(delete_result.is_ok(), "delete_obs_password が成功すること");

        // 削除後の確認
        let get_result = get_obs_password();
        assert!(get_result.is_ok());
        assert_eq!(get_result.unwrap(), None, "削除後はNoneが返ること");
    }

    #[test]
    fn test_delete_nonexistent_password() {
        // 存在しないパスワードの削除はエラーにならない
        // まず確実に削除
        let _ = delete_obs_password();

        // 再度削除してもエラーにならない
        let result = delete_obs_password();
        // キーリングが利用できない場合はスキップ
        if let Err(e) = &result {
            if e.message().contains("作成に失敗") {
                eprintln!("[SKIP] キーリングが利用できません");
                return;
            }
        }
        assert!(result.is_ok(), "存在しないパスワードの削除もOkになること");
    }

    #[test]
    fn test_migrate_from_plaintext_with_password() {
        // 移行テスト（パスワードあり）
        let result = migrate_from_plaintext(Some("migration_test_password"));

        // キーリングが利用できない場合はfalseが返る
        assert!(result.is_ok());

        if result.as_ref().is_ok_and(|v| *v) {
            // 移行成功時は取得できる
            let get_result = get_obs_password();
            assert!(get_result.is_ok());
            assert_eq!(
                get_result.unwrap(),
                Some("migration_test_password".to_string())
            );

            // クリーンアップ
            let _ = delete_obs_password();
        }
    }

    #[test]
    fn test_migrate_from_plaintext_without_password() {
        // 移行テスト（パスワードなし）
        let result = migrate_from_plaintext(None);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "パスワードがない場合はfalseが返ること");
    }

    #[test]
    fn test_migrate_from_plaintext_empty_password() {
        // 移行テスト（空パスワード）
        let result = migrate_from_plaintext(Some(""));
        assert!(result.is_ok());
        assert!(!result.unwrap(), "空パスワードはfalseが返ること");
    }

    #[test]
    fn test_is_keyring_available() {
        // キーリング利用可能性チェック
        // 結果に関わらずクラッシュしないこと
        let _available = is_keyring_available();
        // 単にパニックしないことを確認
    }

    #[test]
    fn test_password_overwrite() {
        // パスワード上書きテスト
        if save_obs_password("first_password").is_err() {
            eprintln!("[SKIP] キーリングが利用できません");
            return;
        }

        // 上書き
        let overwrite_result = save_obs_password("second_password");
        assert!(overwrite_result.is_ok(), "パスワード上書きが成功すること");

        // 新しいパスワードが取得できる
        let get_result = get_obs_password();
        assert!(get_result.is_ok());
        assert_eq!(
            get_result.unwrap(),
            Some("second_password".to_string()),
            "上書き後は新しいパスワードが返ること"
        );

        // クリーンアップ
        let _ = delete_obs_password();
    }

    #[test]
    fn test_special_characters_in_password() {
        // 特殊文字を含むパスワード
        let special_password = "p@$$w0rd!#%^&*()_+-=[]{}|;':\",./<>?";

        if save_obs_password(special_password).is_err() {
            eprintln!("[SKIP] キーリングが利用できません");
            return;
        }

        let get_result = get_obs_password();
        assert!(get_result.is_ok());
        assert_eq!(
            get_result.unwrap(),
            Some(special_password.to_string()),
            "特殊文字を含むパスワードが正しく保存・取得できること"
        );

        // クリーンアップ
        let _ = delete_obs_password();
    }

    #[test]
    fn test_unicode_password() {
        // ユニコード文字を含むパスワード
        let unicode_password = "パスワード123";

        if save_obs_password(unicode_password).is_err() {
            eprintln!("[SKIP] キーリングが利用できません");
            return;
        }

        let get_result = get_obs_password();
        assert!(get_result.is_ok());
        assert_eq!(
            get_result.unwrap(),
            Some(unicode_password.to_string()),
            "ユニコード文字を含むパスワードが正しく保存・取得できること"
        );

        // クリーンアップ
        let _ = delete_obs_password();
    }

    #[test]
    fn test_long_password() {
        // 長いパスワード（256文字）
        let long_password: String = (0..256).map(|i| ((i % 26) as u8 + b'a') as char).collect();

        if save_obs_password(&long_password).is_err() {
            eprintln!("[SKIP] キーリングが利用できません");
            return;
        }

        let get_result = get_obs_password();
        assert!(get_result.is_ok());
        assert_eq!(
            get_result.unwrap(),
            Some(long_password),
            "長いパスワードが正しく保存・取得できること"
        );

        // クリーンアップ
        let _ = delete_obs_password();
    }

    #[test]
    fn test_error_code_constant() {
        assert_eq!(ERROR_CODE_KEYRING, "KEYRING_ERROR");
    }

    #[test]
    fn test_keyring_error_format() {
        let error = keyring_error("テストエラー");
        assert_eq!(error.code(), ERROR_CODE_KEYRING);
        assert_eq!(error.message(), "テストエラー");
    }
}
