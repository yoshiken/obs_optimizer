import { useHistoryStore } from '../../stores/historyStore';
import type { SessionSummary } from '../../types/commands';

/**
 * 2セッションの並列比較
 * - 各メトリクスの差分表示
 * - 改善/悪化の視覚化
 */
export function SessionComparison() {
  const { sessions, selectedSessionIds } = useHistoryStore();

  // 選択されたセッションを取得
  const session1 = sessions.find((s) => s.sessionId === selectedSessionIds[0]);
  const session2 = sessions.find((s) => s.sessionId === selectedSessionIds[1]);

  if (selectedSessionIds.length < 2) {
    return (
      <div className="max-w-6xl mx-auto p-6">
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-8 text-center">
          <p className="text-blue-700 text-lg font-semibold">セッション比較</p>
          <p className="text-blue-600 text-sm mt-2">
            比較するには2つのセッションを選択してください
          </p>
        </div>
      </div>
    );
  }

  if (!session1 || !session2) {
    return (
      <div className="max-w-6xl mx-auto p-6">
        <div className="bg-red-50 border border-red-200 rounded-lg p-8 text-center">
          <p className="text-red-700 text-lg font-semibold">エラー</p>
          <p className="text-red-600 text-sm mt-2">選択されたセッションが見つかりません</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-6xl mx-auto p-6">
      <h1 className="text-2xl font-bold text-gray-900 mb-6">セッション比較</h1>

      {/* セッション情報ヘッダー */}
      <div className="grid grid-cols-2 gap-6 mb-6">
        <SessionHeader session={session1} label="セッション 1" />
        <SessionHeader session={session2} label="セッション 2" />
      </div>

      {/* メトリクス比較 */}
      <div className="space-y-4">
        <MetricComparison
          label="品質スコア"
          value1={session1.qualityScore}
          value2={session2.qualityScore}
          unit=""
          higherIsBetter
        />
        <MetricComparison
          label="平均CPU使用率"
          value1={session1.avgCpu}
          value2={session2.avgCpu}
          unit="%"
          higherIsBetter={false}
        />
        <MetricComparison
          label="平均GPU使用率"
          value1={session1.avgGpu}
          value2={session2.avgGpu}
          unit="%"
          higherIsBetter={false}
        />
        <MetricComparison
          label="ドロップフレーム数"
          value1={session1.totalDroppedFrames}
          value2={session2.totalDroppedFrames}
          unit="フレーム"
          higherIsBetter={false}
        />
        <MetricComparison
          label="最大ビットレート"
          value1={session1.peakBitrate / 1000}
          value2={session2.peakBitrate / 1000}
          unit="Mbps"
          higherIsBetter
          decimals={1}
        />
      </div>

      {/* 総合評価 */}
      <div className="mt-8 bg-gray-50 border border-gray-200 rounded-lg p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">総合評価</h2>
        <ComparisonSummary session1={session1} session2={session2} />
      </div>
    </div>
  );
}

interface SessionHeaderProps {
  session: SessionSummary;
  label: string;
}

function SessionHeader({ session, label }: SessionHeaderProps) {
  const startTime = new Date(session.startTime).toLocaleString('ja-JP');
  const endTime = new Date(session.endTime).toLocaleString('ja-JP');

  return (
    <div className="bg-white border border-gray-200 rounded-lg p-4">
      <h2 className="text-sm font-semibold text-gray-600 mb-2">{label}</h2>
      <p className="text-lg font-bold text-gray-900 mb-2">
        {session.sessionId.substring(0, 8)}
      </p>
      <div className="text-sm text-gray-600 space-y-1">
        <p>開始: {startTime}</p>
        <p>終了: {endTime}</p>
      </div>
    </div>
  );
}

interface MetricComparisonProps {
  label: string;
  value1: number;
  value2: number;
  unit: string;
  higherIsBetter: boolean;
  decimals?: number;
}

function MetricComparison({ label, value1, value2, unit, higherIsBetter, decimals = 0 }: MetricComparisonProps) {
  const diff = value2 - value1;
  const diffPercent = value1 !== 0 ? ((diff / value1) * 100) : 0;

  // 改善/悪化の判定
  const isImprovement = higherIsBetter ? diff > 0 : diff < 0;
  const isDegradation = higherIsBetter ? diff < 0 : diff > 0;

  const diffColor = isImprovement
    ? 'text-green-600'
    : isDegradation
      ? 'text-red-600'
      : 'text-gray-600';

  const bgColor = isImprovement
    ? 'bg-green-50 border-green-200'
    : isDegradation
      ? 'bg-red-50 border-red-200'
      : 'bg-gray-50 border-gray-200';

  const formatValue = (value: number): string => {
    return value.toFixed(decimals);
  };

  return (
    <div className={`border rounded-lg p-4 ${bgColor}`}>
      <h3 className="text-sm font-semibold text-gray-700 mb-3">{label}</h3>
      <div className="grid grid-cols-3 gap-4 items-center">
        {/* セッション1の値 */}
        <div className="text-center">
          <p className="text-xs text-gray-500 mb-1">セッション 1</p>
          <p className="text-2xl font-bold text-gray-900">
            {formatValue(value1)}
            <span className="text-sm font-normal text-gray-600 ml-1">{unit}</span>
          </p>
        </div>

        {/* 差分表示 */}
        <div className="text-center">
          <div className={`inline-flex items-center gap-1 ${diffColor}`}>
            {diff !== 0 && (
              <span className="text-lg">
                {isImprovement ? '↑' : isDegradation ? '↓' : '→'}
              </span>
            )}
            <div>
              <p className="text-xl font-bold">
                {diff > 0 ? '+' : ''}
                {formatValue(diff)}
              </p>
              {diffPercent !== 0 && Math.abs(diffPercent) !== Infinity && (
                <p className="text-xs">
                  ({diffPercent > 0 ? '+' : ''}
                  {diffPercent.toFixed(1)}%)
                </p>
              )}
            </div>
          </div>
        </div>

        {/* セッション2の値 */}
        <div className="text-center">
          <p className="text-xs text-gray-500 mb-1">セッション 2</p>
          <p className="text-2xl font-bold text-gray-900">
            {formatValue(value2)}
            <span className="text-sm font-normal text-gray-600 ml-1">{unit}</span>
          </p>
        </div>
      </div>
    </div>
  );
}

interface ComparisonSummaryProps {
  session1: SessionSummary;
  session2: SessionSummary;
}

function ComparisonSummary({ session1, session2 }: ComparisonSummaryProps) {
  const improvements: string[] = [];
  const degradations: string[] = [];

  // 品質スコア
  if (session2.qualityScore > session1.qualityScore) {
    improvements.push(
      `品質スコアが ${(session2.qualityScore - session1.qualityScore).toFixed(0)} ポイント向上`
    );
  } else if (session2.qualityScore < session1.qualityScore) {
    degradations.push(
      `品質スコアが ${(session1.qualityScore - session2.qualityScore).toFixed(0)} ポイント低下`
    );
  }

  // CPU使用率
  if (session2.avgCpu < session1.avgCpu) {
    improvements.push(
      `CPU使用率が ${(session1.avgCpu - session2.avgCpu).toFixed(1)}% 削減`
    );
  } else if (session2.avgCpu > session1.avgCpu) {
    degradations.push(
      `CPU使用率が ${(session2.avgCpu - session1.avgCpu).toFixed(1)}% 増加`
    );
  }

  // GPU使用率
  if (session2.avgGpu < session1.avgGpu) {
    improvements.push(
      `GPU使用率が ${(session1.avgGpu - session2.avgGpu).toFixed(1)}% 削減`
    );
  } else if (session2.avgGpu > session1.avgGpu) {
    degradations.push(
      `GPU使用率が ${(session2.avgGpu - session1.avgGpu).toFixed(1)}% 増加`
    );
  }

  // ドロップフレーム
  if (session2.totalDroppedFrames < session1.totalDroppedFrames) {
    improvements.push(
      `ドロップフレームが ${session1.totalDroppedFrames - session2.totalDroppedFrames} フレーム削減`
    );
  } else if (session2.totalDroppedFrames > session1.totalDroppedFrames) {
    degradations.push(
      `ドロップフレームが ${session2.totalDroppedFrames - session1.totalDroppedFrames} フレーム増加`
    );
  }

  if (improvements.length === 0 && degradations.length === 0) {
    return (
      <p className="text-gray-600">セッション間で顕著な違いはありません</p>
    );
  }

  return (
    <div className="space-y-4">
      {improvements.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-green-700 mb-2">改善点</h3>
          <ul className="list-disc list-inside space-y-1">
            {improvements.map((item, index) => (
              <li key={index} className="text-sm text-green-600">
                {item}
              </li>
            ))}
          </ul>
        </div>
      )}

      {degradations.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-red-700 mb-2">悪化点</h3>
          <ul className="list-disc list-inside space-y-1">
            {degradations.map((item, index) => (
              <li key={index} className="text-sm text-red-600">
                {item}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
