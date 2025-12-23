import { useEffect, useState } from 'react';
import { useHistoryStore } from '../../stores/historyStore';
import type { SessionSummary } from '../../types/commands';

/**
 * 過去のセッション一覧
 * - セッション選択（最大2つ比較）
 * - 時間範囲フィルター
 */
export function SessionHistory() {
  const { sessions, selectedSessionIds, isLoading, error, loadSessions, selectSession, deselectSession, clearError } = useHistoryStore();
  const [timeFilter, setTimeFilter] = useState<'all' | '24h' | '7d' | '30d'>('all');

  useEffect(() => {
    void loadSessions();
  }, [loadSessions]);

  // 時間範囲フィルター
  const filteredSessions = sessions.filter((session) => {
    if (timeFilter === 'all') {
      return true;
    }

    const now = Date.now();
    const sessionTime = session.startTime;
    const diff = now - sessionTime;

    switch (timeFilter) {
      case '24h':
        return diff <= 24 * 60 * 60 * 1000;
      case '7d':
        return diff <= 7 * 24 * 60 * 60 * 1000;
      case '30d':
        return diff <= 30 * 24 * 60 * 60 * 1000;
      default:
        return true;
    }
  });

  // セッションクリック処理
  const handleSessionClick = (sessionId: string) => {
    if (selectedSessionIds.includes(sessionId)) {
      deselectSession(sessionId);
    } else {
      selectSession(sessionId);
    }
  };

  // 品質スコアの色
  const getQualityColor = (score: number): string => {
    if (score >= 90) {
      return 'text-green-600';
    }
    if (score >= 70) {
      return 'text-yellow-600';
    }
    return 'text-red-600';
  };

  // セッション期間のフォーマット
  const formatDuration = (startTime: number, endTime: number): string => {
    const duration = endTime - startTime;
    const hours = Math.floor(duration / (60 * 60 * 1000));
    const minutes = Math.floor((duration % (60 * 60 * 1000)) / (60 * 1000));
    return `${hours}時間${minutes}分`;
  };

  return (
    <div className="max-w-6xl mx-auto p-6">
      {/* ヘッダー */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-4">
          <h1 className="text-2xl font-bold text-gray-900">セッション履歴</h1>
          <button
            onClick={() => {
              void loadSessions();
            }}
            disabled={isLoading}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            aria-label="セッション一覧を再読み込み"
          >
            {isLoading ? '読み込み中...' : '更新'}
          </button>
        </div>

        {/* エラー表示 */}
        {error && (
          <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-4" role="alert">
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-2">
                <span className="text-red-600 font-semibold">エラー:</span>
                <span className="text-red-700">{error}</span>
              </div>
              <button
                onClick={clearError}
                className="text-red-600 hover:text-red-800"
                aria-label="エラーを閉じる"
              >
                ✕
              </button>
            </div>
          </div>
        )}

        {/* 選択状態の表示 */}
        {selectedSessionIds.length > 0 && (
          <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-4">
            <p className="text-sm text-blue-700">
              {selectedSessionIds.length}件のセッションを選択中
              {selectedSessionIds.length === 2 && ' (最大選択数)'}
            </p>
          </div>
        )}

        {/* 時間範囲フィルター */}
        <div className="bg-white rounded-lg shadow-sm p-4 mb-6">
          <h2 id="time-filter-label" className="text-sm font-semibold text-gray-700 mb-3">時間範囲</h2>
          <div className="flex gap-2" role="group" aria-labelledby="time-filter-label">
            <button
              onClick={() => setTimeFilter('all')}
              className={`px-4 py-2 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                timeFilter === 'all'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
              aria-pressed={timeFilter === 'all'}
              aria-label="すべての期間を表示"
            >
              すべて
            </button>
            <button
              onClick={() => setTimeFilter('24h')}
              className={`px-4 py-2 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                timeFilter === '24h'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
              aria-pressed={timeFilter === '24h'}
              aria-label="過去24時間を表示"
            >
              24時間
            </button>
            <button
              onClick={() => setTimeFilter('7d')}
              className={`px-4 py-2 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                timeFilter === '7d'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
              aria-pressed={timeFilter === '7d'}
              aria-label="過去7日間を表示"
            >
              7日間
            </button>
            <button
              onClick={() => setTimeFilter('30d')}
              className={`px-4 py-2 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                timeFilter === '30d'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
              aria-pressed={timeFilter === '30d'}
              aria-label="過去30日間を表示"
            >
              30日間
            </button>
          </div>
        </div>
      </div>

      {/* セッション一覧 */}
      <div className="space-y-3">
        {isLoading ? (
          <div className="text-center py-12" role="status" aria-live="polite">
            <div className="inline-block animate-spin rounded-full h-12 w-12 border-4 border-gray-300 border-t-blue-600" />
            <p className="mt-4 text-gray-600">セッションを読み込み中...</p>
          </div>
        ) : filteredSessions.length === 0 ? (
          <div className="bg-gray-50 border border-gray-200 rounded-lg p-8 text-center">
            <p className="text-gray-700 text-lg font-semibold">セッションが見つかりません</p>
            <p className="text-gray-600 text-sm mt-2">配信を開始すると履歴が記録されます</p>
          </div>
        ) : (
          <div role="list" aria-label="セッション一覧">
            {filteredSessions.map((session) => (
              <SessionCard
                key={session.sessionId}
                session={session}
                isSelected={selectedSessionIds.includes(session.sessionId)}
                onClick={() => handleSessionClick(session.sessionId)}
                getQualityColor={getQualityColor}
                formatDuration={formatDuration}
              />
            ))}
          </div>
        )}
      </div>

      {/* フィルター結果表示 */}
      {!isLoading && filteredSessions.length > 0 && (
        <div className="mt-4 text-sm text-gray-600 text-center">
          {filteredSessions.length} / {sessions.length} 件のセッションを表示中
        </div>
      )}
    </div>
  );
}

interface SessionCardProps {
  session: SessionSummary;
  isSelected: boolean;
  onClick: () => void;
  getQualityColor: (score: number) => string;
  formatDuration: (start: number, end: number) => string;
}

function SessionCard({ session, isSelected, onClick, getQualityColor, formatDuration }: SessionCardProps) {
  const startTime = new Date(session.startTime).toLocaleString('ja-JP');
  const duration = formatDuration(session.startTime, session.endTime);

  return (
    <button
      onClick={onClick}
      className={`w-full text-left p-4 rounded-lg border-2 transition-all focus:outline-none focus:ring-2 focus:ring-blue-500 ${
        isSelected
          ? 'border-blue-500 bg-blue-50 shadow-md'
          : 'border-gray-200 bg-white hover:border-gray-300 hover:shadow-sm'
      }`}
      aria-pressed={isSelected}
      aria-label={`セッション ${session.sessionId.substring(0, 8)}を${isSelected ? '選択解除' : '選択'}`}
    >
      <div className="flex items-start justify-between">
        {/* セッション情報 */}
        <div className="flex-1">
          <div className="flex items-center gap-2 mb-2">
            <h3 className="text-lg font-semibold text-gray-900">
              セッション {session.sessionId.substring(0, 8)}
            </h3>
            <span className={`text-sm font-bold ${getQualityColor(session.qualityScore)}`}>
              品質スコア: {session.qualityScore}
            </span>
          </div>
          <p className="text-sm text-gray-600 mb-3">
            {startTime} · {duration}
          </p>

          {/* メトリクス */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div>
              <p className="text-xs text-gray-500">平均CPU</p>
              <p className="text-sm font-semibold text-gray-900">{session.avgCpu.toFixed(1)}%</p>
            </div>
            <div>
              <p className="text-xs text-gray-500">平均GPU</p>
              <p className="text-sm font-semibold text-gray-900">{session.avgGpu.toFixed(1)}%</p>
            </div>
            <div>
              <p className="text-xs text-gray-500">ドロップフレーム</p>
              <p className="text-sm font-semibold text-gray-900">{session.totalDroppedFrames}</p>
            </div>
            <div>
              <p className="text-xs text-gray-500">最大ビットレート</p>
              <p className="text-sm font-semibold text-gray-900">
                {(session.peakBitrate / 1000).toFixed(1)} Mbps
              </p>
            </div>
          </div>
        </div>

        {/* 選択インジケーター */}
        {isSelected && (
          <div className="ml-4 flex-shrink-0">
            <div className="w-6 h-6 bg-blue-600 rounded-full flex items-center justify-center">
              <svg className="w-4 h-4 text-white" fill="currentColor" viewBox="0 0 20 20">
                <path
                  fillRule="evenodd"
                  d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                  clipRule="evenodd"
                />
              </svg>
            </div>
          </div>
        )}
      </div>
    </button>
  );
}
