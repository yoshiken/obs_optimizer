import { useEffect, useState } from 'react';
import { useProfileStore } from '../../stores/profileStore';
import type { StreamingPlatform, StreamingStyle } from '../../types/commands';

interface ProfileListProps {
  /** プロファイル適用時のコールバック */
  onApplied?: () => void;
  /** プロファイル削除時のコールバック */
  onDeleted?: () => void;
  /** エラー発生時のコールバック */
  onError?: (error: string) => void;
}

/**
 * プロファイル一覧コンポーネント
 * 保存されたプロファイルの表示・選択・適用・削除機能を提供します
 */
export function ProfileList({ onApplied, onDeleted, onError }: ProfileListProps) {
  const {
    profiles,
    selectedProfileId,
    isLoading,
    error,
    loadProfiles,
    applyProfile,
    deleteProfile,
    setSelectedProfile,
    clearError,
  } = useProfileStore();

  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);
  const [isApplying, setIsApplying] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  // 初回読み込み
  useEffect(() => {
    void loadProfiles();
  }, [loadProfiles]);

  // エラーハンドリング
  useEffect(() => {
    if (error) {
      onError?.(error);
      clearError();
    }
  }, [error, onError, clearError]);

  // プロファイル適用
  const handleApply = async (id: string) => {
    setIsApplying(true);
    try {
      await applyProfile(id);
      onApplied?.();
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      onError?.(errorMessage);
    } finally {
      setIsApplying(false);
    }
  };

  // プロファイル削除
  const handleDelete = async (id: string) => {
    setIsDeleting(true);
    try {
      await deleteProfile(id);
      onDeleted?.();
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      onError?.(errorMessage);
    } finally {
      setIsDeleting(false);
      setConfirmDeleteId(null);
    }
  };

  // プラットフォーム名の取得
  const getPlatformName = (platform: StreamingPlatform): string => {
    const names: Record<StreamingPlatform, string> = {
      youTube: 'YouTube',
      twitch: 'Twitch',
      nicoNico: 'ニコニコ生放送',
      twitCasting: 'ツイキャス',
      other: 'その他',
    };
    return names[platform];
  };

  // スタイル名の取得
  const getStyleName = (style: StreamingStyle): string => {
    const names: Record<StreamingStyle, string> = {
      talk: '雑談',
      gaming: 'ゲーム',
      music: '音楽',
      art: 'お絵描き',
      other: 'その他',
    };
    return names[style];
  };

  // 日時のフォーマット
  const formatDate = (timestamp: number): string => {
    const date = new Date(timestamp);
    return date.toLocaleString('ja-JP', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="space-y-4">
      {/* ヘッダー */}
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
          プロファイル
        </h3>
        <button
          onClick={() => void loadProfiles()}
          disabled={isLoading}
          className="text-sm text-blue-600 dark:text-blue-400 hover:underline
                     disabled:opacity-50"
          aria-label="プロファイル一覧を再読み込み"
        >
          {isLoading ? '読み込み中...' : '更新'}
        </button>
      </div>

      {/* プロファイル一覧 */}
      {profiles.length === 0 ? (
        <p className="text-gray-500 dark:text-gray-400 text-sm text-center py-8">
          {isLoading ? '読み込み中...' : 'プロファイルがありません'}
        </p>
      ) : (
        <ul className="space-y-3" role="list" aria-label="プロファイル一覧">
          {profiles.map((profile) => (
            <li
              key={profile.id}
              className={`bg-gray-50 dark:bg-gray-800 rounded-lg p-4
                         border-2 transition-colors
                         ${
                           selectedProfileId === profile.id
                             ? 'border-blue-500 dark:border-blue-400'
                             : 'border-gray-200 dark:border-gray-700'
                         }`}
              onClick={() => setSelectedProfile(profile.id)}
              role="button"
              tabIndex={0}
              aria-selected={selectedProfileId === profile.id}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  setSelectedProfile(profile.id);
                }
              }}
            >
              <div className="flex justify-between items-start gap-4">
                <div className="flex-1 min-w-0">
                  <h4 className="font-semibold text-gray-900 dark:text-white">
                    {profile.name}
                  </h4>
                  <p className="text-sm text-gray-600 dark:text-gray-300 mt-1">
                    {profile.description}
                  </p>
                  <div className="flex gap-2 mt-2 flex-wrap">
                    <span className="text-xs px-2 py-1 bg-blue-100 dark:bg-blue-900
                                     text-blue-800 dark:text-blue-100 rounded">
                      {getPlatformName(profile.platform)}
                    </span>
                    <span className="text-xs px-2 py-1 bg-green-100 dark:bg-green-900
                                     text-green-800 dark:text-green-100 rounded">
                      {getStyleName(profile.style)}
                    </span>
                  </div>
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-2">
                    更新: {formatDate(profile.updatedAt)}
                  </p>
                </div>

                <div className="flex flex-col gap-2">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      void handleApply(profile.id);
                    }}
                    disabled={isApplying || isLoading}
                    className="px-3 py-1 text-sm bg-blue-600 text-white rounded
                               hover:bg-blue-700 disabled:opacity-50
                               transition-colors whitespace-nowrap
                               focus:outline-none focus:ring-2 focus:ring-blue-500"
                    aria-label={`${profile.name}を適用`}
                  >
                    適用
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      setConfirmDeleteId(profile.id);
                    }}
                    disabled={isDeleting || isLoading}
                    className="px-3 py-1 text-sm bg-red-600 text-white rounded
                               hover:bg-red-700 disabled:opacity-50
                               transition-colors whitespace-nowrap
                               focus:outline-none focus:ring-2 focus:ring-red-500"
                    aria-label={`${profile.name}を削除`}
                  >
                    削除
                  </button>
                </div>
              </div>
            </li>
          ))}
        </ul>
      )}

      {/* 削除確認ダイアログ */}
      {confirmDeleteId && (
        <div
          className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
          role="dialog"
          aria-labelledby="delete-dialog-title"
          aria-modal="true"
        >
          <div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
            <h2
              id="delete-dialog-title"
              className="text-xl font-bold mb-4 text-gray-900 dark:text-white"
            >
              プロファイルを削除しますか?
            </h2>
            <p className="text-gray-600 dark:text-gray-300 mb-6">
              この操作は取り消せません。削除されたプロファイルは復元できません。
            </p>

            <div className="flex gap-3 justify-end">
              <button
                onClick={() => setConfirmDeleteId(null)}
                disabled={isDeleting}
                className="px-4 py-2 text-gray-700 dark:text-gray-300
                           bg-gray-200 dark:bg-gray-700 rounded
                           hover:bg-gray-300 dark:hover:bg-gray-600
                           disabled:opacity-50 transition-colors"
                aria-label="キャンセル"
              >
                キャンセル
              </button>
              <button
                onClick={() => void handleDelete(confirmDeleteId)}
                disabled={isDeleting}
                className="px-4 py-2 bg-red-600 text-white rounded
                           hover:bg-red-700 disabled:opacity-50
                           transition-colors
                           focus:outline-none focus:ring-2 focus:ring-red-500"
                aria-label="削除を確定"
              >
                {isDeleting ? '削除中...' : '削除'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
