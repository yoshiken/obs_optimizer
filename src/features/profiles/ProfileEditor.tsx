import { useEffect, useState } from 'react';
import { useProfileStore } from '../../stores/profileStore';
import type {
  ProfileSettings,
  SettingsProfile,
  StreamingPlatform,
  StreamingStyle,
} from '../../types/commands';

interface ProfileEditorProps {
  /** 編集対象のプロファイル（nullの場合は新規作成） */
  profileId?: string | null;
  /** 初期設定値（現在のOBS設定など） */
  initialSettings?: ProfileSettings;
  /** 保存完了時のコールバック */
  onSaved?: () => void;
  /** キャンセル時のコールバック */
  onCancel?: () => void;
  /** エラー発生時のコールバック */
  onError?: (error: string) => void;
}

/**
 * プロファイル編集コンポーネント
 * プロファイルの作成・編集機能を提供します
 */
export function ProfileEditor({
  profileId,
  initialSettings,
  onSaved,
  onCancel,
  onError,
}: ProfileEditorProps) {
  const { profiles, saveProfile } = useProfileStore();

  // フォーム状態
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [platform, setPlatform] = useState<StreamingPlatform>('youTube');
  const [style, setStyle] = useState<StreamingStyle>('talk');
  const [settings, setSettings] = useState<ProfileSettings>(
    initialSettings || getDefaultSettings()
  );
  const [isSaving, setIsSaving] = useState(false);

  // 編集モードの場合、既存プロファイルをロード
  useEffect(() => {
    if (profileId) {
      const profile = profiles.find((p) => p.id === profileId);
      if (profile) {
        setName(profile.name);
        setDescription(profile.description);
        setPlatform(profile.platform);
        setStyle(profile.style);
        setSettings(profile.settings);
      }
    }
  }, [profileId, profiles]);

  // 保存処理
  const handleSave = async () => {
    // バリデーション
    if (!name.trim()) {
      onError?.('プロファイル名を入力してください');
      return;
    }

    setIsSaving(true);
    try {
      const now = Date.now();
      const profile: SettingsProfile = {
        id: profileId || `profile_${now}`,
        name: name.trim(),
        description: description.trim(),
        platform,
        style,
        settings,
        createdAt: profileId
          ? profiles.find((p) => p.id === profileId)?.createdAt || now
          : now,
        updatedAt: now,
      };

      await saveProfile(profile);
      onSaved?.();
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      onError?.(errorMessage);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* ヘッダー */}
      <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
        {profileId ? 'プロファイルを編集' : '新規プロファイル'}
      </h3>

      {/* 基本情報 */}
      <div className="space-y-4">
        <div>
          <label
            htmlFor="profile-name"
            className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
          >
            プロファイル名 <span className="text-red-500">*</span>
          </label>
          <input
            id="profile-name"
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="例: YouTube高画質配信"
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
                       rounded-lg bg-white dark:bg-gray-800
                       text-gray-900 dark:text-white
                       focus:outline-none focus:ring-2 focus:ring-blue-500"
            aria-required="true"
          />
        </div>

        <div>
          <label
            htmlFor="profile-description"
            className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
          >
            説明
          </label>
          <textarea
            id="profile-description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="プロファイルの説明を入力"
            rows={3}
            className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
                       rounded-lg bg-white dark:bg-gray-800
                       text-gray-900 dark:text-white
                       focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label
              htmlFor="platform-select"
              className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
            >
              プラットフォーム
            </label>
            <select
              id="platform-select"
              value={platform}
              onChange={(e) => setPlatform(e.target.value as StreamingPlatform)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
                         rounded-lg bg-white dark:bg-gray-800
                         text-gray-900 dark:text-white
                         focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="youTube">YouTube</option>
              <option value="twitch">Twitch</option>
              <option value="nicoNico">ニコニコ生放送</option>
              <option value="other">その他</option>
            </select>
          </div>

          <div>
            <label
              htmlFor="style-select"
              className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
            >
              配信スタイル
            </label>
            <select
              id="style-select"
              value={style}
              onChange={(e) => setStyle(e.target.value as StreamingStyle)}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600
                         rounded-lg bg-white dark:bg-gray-800
                         text-gray-900 dark:text-white
                         focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="talk">雑談</option>
              <option value="gaming">ゲーム</option>
              <option value="music">音楽</option>
              <option value="art">お絵描き</option>
              <option value="other">その他</option>
            </select>
          </div>
        </div>
      </div>

      {/* 設定プレビュー */}
      <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 space-y-3">
        <h4 className="font-medium text-gray-900 dark:text-white">設定プレビュー</h4>

        <div className="grid grid-cols-2 gap-4 text-sm">
          <div>
            <p className="text-gray-600 dark:text-gray-400">解像度</p>
            <p className="text-gray-900 dark:text-white font-medium">
              {settings.video.outputWidth}x{settings.video.outputHeight}
            </p>
          </div>
          <div>
            <p className="text-gray-600 dark:text-gray-400">FPS</p>
            <p className="text-gray-900 dark:text-white font-medium">
              {settings.video.fps}
            </p>
          </div>
          <div>
            <p className="text-gray-600 dark:text-gray-400">音声サンプルレート</p>
            <p className="text-gray-900 dark:text-white font-medium">
              {settings.audio.sampleRate} Hz
            </p>
          </div>
          <div>
            <p className="text-gray-600 dark:text-gray-400">音声ビットレート</p>
            <p className="text-gray-900 dark:text-white font-medium">
              {settings.audio.bitrateKbps} kbps
            </p>
          </div>
          <div>
            <p className="text-gray-600 dark:text-gray-400">映像ビットレート</p>
            <p className="text-gray-900 dark:text-white font-medium">
              {settings.output.bitrateKbps} kbps
            </p>
          </div>
          <div>
            <p className="text-gray-600 dark:text-gray-400">エンコーダー</p>
            <p className="text-gray-900 dark:text-white font-medium">
              {settings.output.encoder}
            </p>
          </div>
          <div>
            <p className="text-gray-600 dark:text-gray-400">キーフレーム間隔</p>
            <p className="text-gray-900 dark:text-white font-medium">
              {settings.output.keyframeIntervalSecs}秒
            </p>
          </div>
          <div>
            <p className="text-gray-600 dark:text-gray-400">ダウンスケールフィルタ</p>
            <p className="text-gray-900 dark:text-white font-medium">
              {settings.video.downscaleFilter}
            </p>
          </div>
        </div>
      </div>

      {/* アクションボタン */}
      <div className="flex gap-3 justify-end">
        <button
          onClick={onCancel}
          disabled={isSaving}
          className="px-4 py-2 text-gray-700 dark:text-gray-300
                     bg-gray-200 dark:bg-gray-700 rounded
                     hover:bg-gray-300 dark:hover:bg-gray-600
                     disabled:opacity-50 transition-colors"
          aria-label="キャンセル"
        >
          キャンセル
        </button>
        <button
          onClick={() => void handleSave()}
          disabled={isSaving || !name.trim()}
          className="px-4 py-2 bg-blue-600 text-white rounded
                     hover:bg-blue-700 disabled:opacity-50
                     transition-colors
                     focus:outline-none focus:ring-2 focus:ring-blue-500"
          aria-label="保存"
        >
          {isSaving ? '保存中...' : '保存'}
        </button>
      </div>
    </div>
  );
}

// デフォルト設定値
function getDefaultSettings(): ProfileSettings {
  return {
    video: {
      outputWidth: 1920,
      outputHeight: 1080,
      fps: 30,
      downscaleFilter: 'bicubic',
    },
    audio: {
      sampleRate: 48000,
      bitrateKbps: 192,
    },
    output: {
      encoder: 'x264',
      bitrateKbps: 6000,
      keyframeIntervalSecs: 2,
      preset: 'veryfast',
      rateControl: 'CBR',
    },
  };
}
