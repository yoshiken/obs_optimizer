/**
 * エラーメッセージ翻訳
 *
 * 技術的なエラーメッセージを初心者向けに翻訳し、解決のヒントを提供
 */

export interface TranslatedError {
  message: string;
  hint?: string;
}

/**
 * エラーメッセージパターンとその翻訳
 */
const ERROR_PATTERNS: Array<{
  pattern: RegExp | string;
  message: string;
  hint?: string;
}> = [
  // 接続エラー
  {
    pattern: /connection refused|ECONNREFUSED/i,
    message: 'OBSに接続できませんでした',
    hint: 'OBSが起動しているか、WebSocketサーバーが有効になっているか確認してください',
  },
  {
    pattern: /timeout|timed out/i,
    message: '接続がタイムアウトしました',
    hint: 'ホスト名とポート番号が正しいか確認してください',
  },
  {
    pattern: /authentication failed|invalid password/i,
    message: 'パスワードが間違っています',
    hint: 'OBSの設定で指定したパスワードを入力してください',
  },
  {
    pattern: /websocket server is not enabled/i,
    message: 'OBSのWebSocketサーバーが有効になっていません',
    hint: 'OBS → ツール → WebSocketサーバー設定 から有効にしてください',
  },

  // ネットワークエラー
  {
    pattern: /network error|ENETUNREACH/i,
    message: 'ネットワークエラーが発生しました',
    hint: 'ネットワーク接続を確認してください',
  },
  {
    pattern: /host not found|ENOTFOUND/i,
    message: 'ホストが見つかりませんでした',
    hint: 'ホスト名が正しいか確認してください（ローカルの場合は「localhost」）',
  },

  // OBS操作エラー
  {
    pattern: /already streaming/i,
    message: 'すでに配信中です',
  },
  {
    pattern: /not streaming/i,
    message: '配信していません',
  },
  {
    pattern: /already recording/i,
    message: 'すでに録画中です',
  },
  {
    pattern: /not recording/i,
    message: '録画していません',
  },
  {
    pattern: /scene not found/i,
    message: '指定したシーンが見つかりませんでした',
    hint: 'OBSでシーンが削除されていないか確認してください',
  },

  // 権限エラー
  {
    pattern: /permission denied|EACCES/i,
    message: 'アクセス権限がありません',
    hint: '管理者権限で実行してみてください',
  },

  // リソースエラー
  {
    pattern: /insufficient resources|out of memory/i,
    message: 'システムリソースが不足しています',
    hint: '他のアプリケーションを閉じて、メモリを確保してください',
  },
];

/**
 * 技術的エラーメッセージを初心者向けに翻訳
 *
 * @param error - エラーオブジェクトまたはエラーメッセージ文字列
 * @returns 翻訳されたエラーメッセージとヒント
 */
export function translateError(error: unknown): TranslatedError {
  // エラーメッセージを文字列化
  let errorMessage = '';
  if (error instanceof Error) {
    errorMessage = error.message;
  } else if (typeof error === 'string') {
    errorMessage = error;
  } else {
    errorMessage = String(error);
  }

  // パターンマッチングで翻訳
  for (const pattern of ERROR_PATTERNS) {
    const regex =
      pattern.pattern instanceof RegExp
        ? pattern.pattern
        : new RegExp(pattern.pattern, 'i');

    if (regex.test(errorMessage)) {
      return {
        message: pattern.message,
        hint: pattern.hint,
      };
    }
  }

  // マッチするパターンがない場合は元のメッセージをそのまま返す
  // trim()で空白のみの文字列も処理
  return {
    message: errorMessage.trim() || '不明なエラーが発生しました',
  };
}

/**
 * エラーメッセージとヒントを1つの文字列に結合
 *
 * @param translatedError - 翻訳されたエラー
 * @returns 結合された文字列
 */
export function formatError(translatedError: TranslatedError): string {
  if (translatedError.hint) {
    return `${translatedError.message}。${translatedError.hint}`;
  }
  return translatedError.message;
}

/**
 * エラーを翻訳してフォーマット（1行で取得）
 *
 * @param error - エラーオブジェクトまたはエラーメッセージ文字列
 * @returns フォーマットされたエラーメッセージ
 */
export function getTranslatedErrorMessage(error: unknown): string {
  return formatError(translateError(error));
}
