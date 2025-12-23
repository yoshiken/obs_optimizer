// ========================================
// 型定義
// ========================================

interface QualityScoreProps {
  score: number; // 0-100
}

// ========================================
// ヘルパー関数
// ========================================

/** スコアに応じた色を取得 */
function getScoreColor(score: number): {
  stroke: string;
  text: string;
  bg: string;
} {
  if (score >= 80) {
    return {
      stroke: 'stroke-green-500',
      text: 'text-green-700',
      bg: 'bg-green-50',
    };
  }
  if (score >= 60) {
    return {
      stroke: 'stroke-yellow-500',
      text: 'text-yellow-700',
      bg: 'bg-yellow-50',
    };
  }
  return {
    stroke: 'stroke-red-500',
    text: 'text-red-700',
    bg: 'bg-red-50',
  };
}

/** スコアの評価ラベル */
function getScoreLabel(score: number): string {
  if (score >= 90) {return '優秀';}
  if (score >= 80) {return '良好';}
  if (score >= 60) {return '改善の余地あり';}
  if (score >= 40) {return '要改善';}
  return '大幅な改善が必要';
}

// ========================================
// コンポーネント
// ========================================

/**
 * 品質スコア表示（円グラフ）
 *
 * 0-100のスコアを視覚的に表示
 *
 * @example
 * <QualityScore score={75} />
 */
export function QualityScore({ score }: QualityScoreProps) {
  const { stroke, text, bg } = getScoreColor(score);

  // SVG円グラフの計算
  const radius = 80;
  const circumference = 2 * Math.PI * radius;
  const strokeDashoffset = circumference - (score / 100) * circumference;

  return (
    <div className={`${bg} rounded-lg p-6 border-2 ${stroke.replace('stroke-', 'border-')}`}>
      <h3 className="text-lg font-semibold text-gray-900 mb-4 text-center">
        品質スコア
      </h3>

      <div className="flex flex-col items-center">
        {/* 円グラフ */}
        <div className="relative w-48 h-48">
          <svg className="w-full h-full transform -rotate-90" viewBox="0 0 200 200">
            {/* 背景の円 */}
            <circle
              cx="100"
              cy="100"
              r={radius}
              fill="none"
              stroke="currentColor"
              strokeWidth="16"
              className="text-gray-200"
            />
            {/* スコアの円 */}
            <circle
              cx="100"
              cy="100"
              r={radius}
              fill="none"
              stroke="currentColor"
              strokeWidth="16"
              strokeLinecap="round"
              className={stroke}
              style={{
                strokeDasharray: circumference,
                strokeDashoffset: strokeDashoffset,
                transition: 'stroke-dashoffset 1s ease-in-out',
              }}
            />
          </svg>

          {/* 中央のスコア表示 */}
          <div className="absolute inset-0 flex flex-col items-center justify-center">
            <span className={`text-4xl font-bold ${text}`}>{score}</span>
            <span className="text-sm text-gray-600">/ 100</span>
          </div>
        </div>

        {/* 評価ラベル */}
        <div className="mt-4 text-center">
          <span className={`text-lg font-semibold ${text}`}>
            {getScoreLabel(score)}
          </span>
        </div>
      </div>
    </div>
  );
}
