# OBS配信最適化ツール - 初心者向けUXギャップ分析と改善提案

## 実施日: 2025-12-24
## 対象ユーザー: 初心者VTuber「あかり」

---

## エグゼクティブサマリー

現状のOBS配信最適化ツールは技術的には必要なデータ（システムスペック、推奨設定、問題検出）をバックエンドで取得できているが、**フロントエンドで初心者に適した形で表示されていない**ため、ユーザーストーリーUS-001～US-004が未達成となっている。

### 重大なUXギャップ（3箇所）

1. **オンボーディング: Analysis.tsx（未実装）**
   - 初心者が「自分のPCのスペック」を確認できない
   - 推奨設定の内容が見えない（スコアと件数のみ）

2. **オンボーディング: AutoOptimize.tsx（未実装）**
   - プリセット選択肢だけで「何が変わるのか」が不明
   - 専門用語（解像度、ビットレート）の説明がない

3. **ダッシュボード: 推奨設定パネル不在**
   - オンボーディング後に推奨設定を再確認できない
   - 現在の設定と推奨設定の比較ができない

---

## 1. ペルソナ分析: 初心者VTuber「あかり」

### プロフィール
- **年齢**: 22歳
- **職業**: 大学生（配信は趣味）
- **配信歴**: 3ヶ月
- **技術レベル**: 低（OBSの基本操作のみ理解、設定ファイルは触ったことがない）
- **使用機材**:
  - PC: ミドルスペック（Ryzen 5 5600X、RTX 3060、16GB RAM）
  - 配信先: YouTube
  - 配信内容: 雑談、歌配信

### ニーズと不安
| ニーズ | 不安・障壁 |
|--------|-----------|
| ワンクリックで最適設定を適用したい | 「推奨設定」が何を変えるのか分からず怖い |
| 問題があれば日本語で具体的に教えてほしい | エラーメッセージが専門用語すぎて理解できない |
| 自分のPCに合った設定が知りたい | どのプリセット（軽量/標準/高品質）を選べばいいか分からない |
| 配信中にカクつくと焦る | 何が原因か、どこを直せばいいか分からない |

### 理想的なUXフロー（現状は実現できていない）
```
1. ツール起動 → 「まずOBSに接続してください」（明確な指示）
2. 接続完了 → 「あかりさんのPCはこんなスペックです」（親しみやすい言葉で確認）
3. 分析開始 → 「現在の設定では少し重いかもしれません」（具体的なフィードバック）
4. 推奨設定表示 → 「解像度を1080p→720pに下げると快適です」（理由と効果を説明）
5. ワンクリック適用 → 「設定を適用しました！試しに配信してみてください」（成功体験）
```

---

## 2. 現状のデータフロー分析

### バックエンド（Rust）で取得可能なデータ

| データ種別 | コマンド | 型定義 | 内容 |
|-----------|---------|-------|------|
| システム環境情報 | `analyze_settings` | `AnalysisResult.systemInfo` | `cpuModel`, `gpuModel`, `totalMemoryMb`, `availableMemoryMb` |
| 推奨設定リスト | `analyze_settings` | `AnalysisResult.recommendations` | `ObsSetting[]`（key, displayName, currentValue, recommendedValue, reason, priority） |
| 品質スコア | `analyze_settings` | `AnalysisResult.qualityScore` | 0-100の数値 |
| 推奨設定詳細 | `calculate_recommendations` | `RecommendedSettings` | video, audio, output設定 + reasons配列 |
| 現在のOBS設定 | `get_obs_settings_command` | `ObsSettings` | video, audio, output設定 |

### フロントエンドで表示されていないデータ

#### Analysis.tsx（オンボーディングステップ3）の欠落情報
```typescript
// 取得できているが表示されていない
interface AnalysisResult {
  systemInfo: {
    cpuModel: string;        // 例: "AMD Ryzen 5 5600X"
    gpuModel: string | null; // 例: "NVIDIA GeForce RTX 3060"
    totalMemoryMb: number;   // 例: 16384
    availableMemoryMb: number; // 例: 8192
  };
  recommendations: ObsSetting[]; // 推奨される設定変更のリスト
  qualityScore: number; // 0-100
  issueCount: number;
}
```

**現状の表示内容（推測）**:
- 「品質スコア: 65点」
- 「問題が3件見つかりました」

**初心者が知りたい情報（未表示）**:
- 自分のPCのCPU/GPU/メモリは何か
- 具体的にどの設定をどう変えるべきか
- なぜその変更が推奨されるのか

#### AutoOptimize.tsx（オンボーディングステップ4）の欠落情報
```typescript
// プリセット選択肢のみ表示（推測）
type OptimizationPreset = 'low' | 'medium' | 'high' | 'ultra' | 'custom';

// 表示されていない情報
interface RecommendedSettings {
  video: {
    outputWidth: number;      // 例: 1280
    outputHeight: number;     // 例: 720
    fps: number;              // 例: 30
    downscaleFilter: string;  // 例: "Lanczos"
  };
  audio: {
    sampleRate: number;       // 例: 48000
    bitrateKbps: number;      // 例: 160
  };
  output: {
    encoder: string;          // 例: "nvencH264"
    bitrateKbps: number;      // 例: 2500
    keyframeIntervalSecs: number; // 例: 2
    preset: string | null;    // 例: "quality"
    rateControl: string;      // 例: "CBR"
  };
  reasons: string[]; // 推奨理由のリスト
}
```

**現状の表示内容（推測）**:
- ラジオボタン: 「軽量」「標準」「高品質」「最高品質」
- ボタン: 「適用する」

**初心者が知りたい情報（未表示）**:
- 各プリセットで具体的にどんな設定になるのか
- 自分のPCに合ったプリセットはどれか
- 適用すると画質やPCの負荷がどう変わるのか

---

## 3. ユーザーストーリー達成度評価

### US-001: 初心者として、ツールを起動したらすぐに何をすべきかわかるようにしたい（Must）

| 評価基準 | 現状 | スコア |
|---------|------|--------|
| 初回起動時に明確なガイダンスがあるか | OnboardingWizardは存在するが、各ステップの説明が不十分 | 60% |
| 次のアクションが明示されているか | 「次へ」ボタンはあるが、「なぜこのステップが必要か」が不明 | 50% |
| 専門知識なしで進められるか | 「分析中...」などの表示だけで、初心者は待つだけしかできない | 40% |

**改善必要度**: 高

### US-002: 初心者として、専門用語ではなくわかりやすい言葉で状態を知りたい（Must）

| 評価基準 | 現状 | スコア |
|---------|------|--------|
| 専門用語の使用頻度 | 「ビットレート」「エンコーダー」「CBR」などそのまま表示（推測） | 30% |
| 平易な日本語での説明 | スコアと件数の数字のみ、文章での説明なし | 20% |
| 用語の補足説明 | なし | 0% |

**改善必要度**: 最高

### US-003: 初心者として、問題があればどう直せばいいか具体的に教えてほしい（Must）

| 評価基準 | 現状 | スコア |
|---------|------|--------|
| 問題の具体的な説明 | 「3件の問題」という数だけ | 20% |
| 解決方法の提示 | recommendations配列は取得済みだが表示されていない | 30% |
| ワンクリックでの修正 | AutoOptimizeでプリセット選択は可能だが、何が変わるか不明 | 40% |

**改善必要度**: 最高

### US-004: 初心者として、ワンクリックで推奨設定を適用したい（Should）

| 評価基準 | 現状 | スコア |
|---------|------|--------|
| ワンクリック適用ボタンの存在 | プリセット選択後の「適用」ボタンあり | 70% |
| 適用前のプレビュー | 設定内容の詳細が見えない | 20% |
| 適用後の確認 | フィードバックが不明 | 30% |

**改善必要度**: 中

---

## 4. UXギャップの詳細分析

### ギャップ1: オンボーディング - Analysis.tsx

#### 現状の問題点
```
┌─────────────────────────────────────┐
│  OBS設定を分析しています...         │
│                                     │
│  [ローディングスピナー]              │
│                                     │
│  ▼ 分析完了後                        │
│  品質スコア: 65点                    │
│  問題が3件見つかりました             │
│                                     │
│  [次へ] ボタン                       │
└─────────────────────────────────────┘
```

**初心者が困るポイント**:
1. 「65点」が良いのか悪いのか判断できない
2. 「3件の問題」が何なのか分からない
3. 自分のPCのスペックが分からないまま次に進む不安
4. このツールを信頼していいのか判断材料がない

#### 取得済みだが表示されていないデータ
```typescript
// AnalysisResult から取得可能
{
  systemInfo: {
    cpuModel: "AMD Ryzen 5 5600X",
    gpuModel: "NVIDIA GeForce RTX 3060",
    totalMemoryMb: 16384,
    availableMemoryMb: 8192
  },
  recommendations: [
    {
      key: "video_bitrate",
      displayName: "映像ビットレート",
      currentValue: 6000,
      recommendedValue: 2500,
      reason: "ネットワーク帯域を考慮すると、現在の設定では配信が不安定になる可能性があります",
      priority: "critical"
    },
    {
      key: "output_resolution",
      displayName: "出力解像度",
      currentValue: "1920x1080",
      recommendedValue: "1280x720",
      reason: "GPUの負荷を軽減し、フレームドロップを防ぐため",
      priority: "recommended"
    },
    // ... 他の推奨設定
  ]
}
```

#### 初心者が欲しい情報の優先順位
1. **自分のPCスペックの確認**（信頼構築）
   - 「あなたのPC: Ryzen 5 5600X、RTX 3060、メモリ16GB」
   - 「このスペックなら快適に配信できます！」

2. **スコアの意味の説明**（文脈提供）
   - 「65点: もう少し設定を調整すると安定します」
   - スコアレンジ: 0-40（要改善）、41-70（調整推奨）、71-100（良好）

3. **問題の具体的な内容**（課題の可視化）
   - 「映像ビットレートが高すぎます（現在6000 → 推奨2500）」
   - 「なぜ？ネットワーク帯域を考慮すると配信が途切れる可能性があります」

4. **解決の方向性**（次のステップへの期待）
   - 「次のステップで、これらの問題を自動的に修正できます」

### ギャップ2: オンボーディング - AutoOptimize.tsx

#### 現状の問題点（推測）
```
┌─────────────────────────────────────┐
│  推奨設定を適用                      │
│                                     │
│  プリセットを選択してください:       │
│  ○ 軽量（低負荷）                    │
│  ● 標準（推奨）                      │
│  ○ 高品質                           │
│  ○ 最高品質                         │
│                                     │
│  [適用する] ボタン                   │
└─────────────────────────────────────┘
```

**初心者が困るポイント**:
1. 「標準」が何を意味するのか不明（解像度？ビットレート？）
2. 自分のPCに「高品質」が使えるのか分からない
3. 「適用する」を押すと何が変わるのか怖い
4. 前のステップの「3件の問題」がどう解決されるのか不明

#### 改善すべき情報表示
```typescript
// calculate_recommendations() で取得可能
interface RecommendedSettings {
  video: {
    outputWidth: 1280,
    outputHeight: 720,
    fps: 30,
    downscaleFilter: "Lanczos"
  },
  output: {
    encoder: "nvencH264",
    bitrateKbps: 2500,
    // ...
  },
  reasons: [
    "RTX 3060のNVENCを使用することで、CPUへの負荷を大幅に削減できます",
    "720p 30fpsは安定した配信に最適なバランスです",
    "ビットレート2500kbpsはYouTube配信の推奨範囲内です"
  ],
  overallScore: 85
}
```

#### 初心者が欲しい情報
1. **各プリセットの詳細**（選択の根拠）
   - 「標準プリセット: 720p、30fps、ビットレート2500」
   - 「あなたのPCに最適です（スコア85点）」

2. **現在との比較**（変化の予測）
   - 「現在: 1080p 60fps → 変更後: 720p 30fps」
   - 「画質は少し下がりますが、配信の安定性が大幅に向上します」

3. **推奨理由の提示**（信頼性）
   - 「RTX 3060のハードウェアエンコーダーを活用します」
   - 「CPUの負荷が約40%削減され、ゲーム配信も快適になります」

4. **適用後の期待値**（安心感）
   - 「適用後も、いつでも元の設定に戻せます」
   - 「まずは1回配信して、問題なければこの設定をキープしましょう」

### ギャップ3: ダッシュボード - 推奨設定パネルの不在

#### 現状の問題点
オンボーディング完了後、ダッシュボード（App.tsx の DashboardTab）には以下のパネルのみ:
- ObsConnectionPanel（OBS接続）
- ObsStatusIndicator（接続状態）
- ObsStreamControls（配信開始/停止）
- MetricsPanel（CPU/GPU/メモリのリアルタイムメトリクス）
- ObsSceneSelector（シーン切り替え）

**欠落しているパネル**:
- 推奨設定の確認パネル
- 現在の設定と推奨設定の比較
- ワンクリック適用ボタン（OneClickApplyは「最適化タブ」にある）

#### 初心者のユースケース
```
シナリオ: 初回オンボーディングで推奨設定を適用したが、
        1週間後に「あれ、どんな設定にしたんだっけ？」と思い出せない

期待する動作:
1. ダッシュボードに「おすすめ設定」パネルがある
2. 現在適用中の設定（720p 30fps、ビットレート2500）が一目で分かる
3. 「この設定はあなたのPCに最適です」というメッセージで安心できる
4. 必要に応じて「再分析」「設定変更」ボタンから最適化タブに遷移できる

現状の動作:
1. ダッシュボードには推奨設定の情報なし
2. 「最適化タブ」を開かないと確認できない
3. MetricsPanelにはリアルタイムメトリクスのみ（推奨値との比較なし）
```

#### MetricsPanel の拡張必要性
現在のMetricsPanel（推測）:
- CPU使用率: 45%（リアルタイム）
- GPU使用率: 60%（リアルタイム）
- メモリ使用率: 50%（リアルタイム）

初心者が欲しい情報:
- CPU使用率: 45%（**推奨範囲: 60%以下** ✓正常）
- GPU使用率: 60%（**推奨範囲: 70%以下** ✓正常）
- 現在の設定: 720p 30fps、ビットレート2500（**あなたのPCに最適**）
- ステータス: 「配信に問題ありません」

---

## 5. 初心者向けUI改善提案

### 提案1: Analysis.tsx の全面刷新

#### 改善前（現状推測）
```
┌──────────────────────────────────────┐
│ OBS設定を分析しています...           │
│ [スピナー]                           │
│                                      │
│ ▼ 完了後                             │
│ 品質スコア: 65点                     │
│ 問題が3件見つかりました              │
│ [次へ]                               │
└──────────────────────────────────────┘
```

#### 改善後（推奨デザイン）
```
┌──────────────────────────────────────────────────────────┐
│ 📊 あなたのPC環境を分析しました                           │
├──────────────────────────────────────────────────────────┤
│                                                          │
│ 💻 あなたのPCスペック                                     │
│ ┌────────────────────────────────────────────────────┐  │
│ │ CPU:    AMD Ryzen 5 5600X (6コア)                  │  │
│ │ GPU:    NVIDIA GeForce RTX 3060                    │  │
│ │ メモリ: 16GB（現在8GB利用可能）                    │  │
│ │                                                    │  │
│ │ ✅ このスペックなら快適に配信できます！              │  │
│ └────────────────────────────────────────────────────┘  │
│                                                          │
│ 📈 現在の設定の診断結果                                   │
│ ┌────────────────────────────────────────────────────┐  │
│ │  品質スコア: 65点                                  │  │
│ │  ┌────────────────────────────────────┐            │  │
│ │  │████████████████░░░░░░░░░░░░░░░░░░░│ 65/100     │  │
│ │  └────────────────────────────────────┘            │  │
│ │                                                    │  │
│ │  評価: もう少し設定を調整すると安定します            │  │
│ │  （71点以上で「良好」になります）                  │  │
│ └────────────────────────────────────────────────────┘  │
│                                                          │
│ ⚠️ 見つかった問題（3件）                                  │
│ ┌────────────────────────────────────────────────────┐  │
│ │ 1. [重要] 映像ビットレートが高すぎます                │  │
│ │    現在: 6000 → おすすめ: 2500                      │  │
│ │    理由: ネットワーク帯域を考えると、配信が途切れる  │  │
│ │         可能性があります                           │  │
│ │                                                    │  │
│ │ 2. [推奨] 出力解像度を下げると快適です                │  │
│ │    現在: 1920x1080 → おすすめ: 1280x720            │  │
│ │    理由: GPUの負荷を軽減し、フレームドロップを      │  │
│ │         防ぎます                                   │  │
│ │                                                    │  │
│ │ 3. [任意] エンコーダーの最適化                        │  │
│ │    おすすめ: RTX 3060のNVENCを活用                 │  │
│ │    理由: CPU負荷を約40%削減できます                │  │
│ └────────────────────────────────────────────────────┘  │
│                                                          │
│ 💡 次のステップ                                           │
│ ┌────────────────────────────────────────────────────┐  │
│ │ これらの問題は、次のステップで自動的に修正できます。  │  │
│ │ あなたのPCに最適な設定を提案します！                │  │
│ └────────────────────────────────────────────────────┘  │
│                                                          │
│                                    [詳細を見る] [次へ]  │
└──────────────────────────────────────────────────────────┘
```

#### 実装仕様

**コンポーネント構造**:
```tsx
// src/features/onboarding/Analysis.tsx
function Analysis() {
  const [result, setResult] = useState<AnalysisResult | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const analyze = async () => {
      const data = await invoke<AnalysisResult>('analyze_settings');
      setResult(data);
      setLoading(false);
    };
    analyze();
  }, []);

  if (loading) return <AnalysisLoading />;
  if (!result) return <AnalysisError />;

  return (
    <div className="space-y-6">
      <Header title="あなたのPC環境を分析しました" icon="📊" />

      <SystemSpecCard systemInfo={result.systemInfo} />

      <QualityScoreCard
        score={result.qualityScore}
        issueCount={result.issueCount}
      />

      <RecommendationsList
        recommendations={result.recommendations}
      />

      <NextStepGuide />
    </div>
  );
}
```

**子コンポーネント詳細**:

```tsx
// SystemSpecCard - PCスペック表示
interface SystemSpecCardProps {
  systemInfo: SystemInfo;
}

function SystemSpecCard({ systemInfo }: SystemSpecCardProps) {
  // メモリをGBに変換
  const totalMemoryGB = Math.round(systemInfo.totalMemoryMb / 1024);
  const availableMemoryGB = Math.round(systemInfo.availableMemoryMb / 1024);

  // スペック判定（簡易版）
  const isGoodSpec = totalMemoryGB >= 16 && systemInfo.gpuModel !== null;

  return (
    <Card>
      <CardHeader>
        <h3>💻 あなたのPCスペック</h3>
      </CardHeader>
      <CardBody>
        <dl className="space-y-2">
          <SpecRow label="CPU" value={systemInfo.cpuModel} />
          <SpecRow
            label="GPU"
            value={systemInfo.gpuModel || "情報取得できず"}
          />
          <SpecRow
            label="メモリ"
            value={`${totalMemoryGB}GB（現在${availableMemoryGB}GB利用可能）`}
          />
        </dl>

        {isGoodSpec && (
          <Alert variant="success" className="mt-4">
            ✅ このスペックなら快適に配信できます！
          </Alert>
        )}
      </CardBody>
    </Card>
  );
}
```

```tsx
// QualityScoreCard - スコア表示
interface QualityScoreCardProps {
  score: number;
  issueCount: number;
}

function QualityScoreCard({ score, issueCount }: QualityScoreCardProps) {
  // スコアに応じた評価テキスト
  const getScoreEvaluation = (score: number): {
    text: string;
    color: string;
    bgColor: string;
  } => {
    if (score >= 71) {
      return {
        text: "良好です！このまま配信できます",
        color: "text-green-600",
        bgColor: "bg-green-100"
      };
    } else if (score >= 41) {
      return {
        text: "もう少し設定を調整すると安定します",
        color: "text-yellow-600",
        bgColor: "bg-yellow-100"
      };
    } else {
      return {
        text: "設定の見直しをおすすめします",
        color: "text-red-600",
        bgColor: "bg-red-100"
      };
    }
  };

  const evaluation = getScoreEvaluation(score);
  const progressPercentage = Math.min(score, 100);

  return (
    <Card>
      <CardHeader>
        <h3>📈 現在の設定の診断結果</h3>
      </CardHeader>
      <CardBody>
        <div className="space-y-4">
          {/* スコアバー */}
          <div>
            <div className="flex justify-between mb-2">
              <span className="text-sm font-medium">品質スコア</span>
              <span className="text-2xl font-bold">{score}点</span>
            </div>

            <div className="w-full bg-gray-200 rounded-full h-4">
              <div
                className={`h-4 rounded-full transition-all ${
                  score >= 71 ? 'bg-green-500' :
                  score >= 41 ? 'bg-yellow-500' : 'bg-red-500'
                }`}
                style={{ width: `${progressPercentage}%` }}
              />
            </div>

            {/* スコアレンジ凡例 */}
            <div className="flex justify-between text-xs text-gray-500 mt-1">
              <span>0-40: 要改善</span>
              <span>41-70: 調整推奨</span>
              <span>71-100: 良好</span>
            </div>
          </div>

          {/* 評価メッセージ */}
          <Alert variant={evaluation.bgColor} className={evaluation.color}>
            評価: {evaluation.text}
            {score < 71 && (
              <span className="block text-sm mt-1">
                （71点以上で「良好」になります）
              </span>
            )}
          </Alert>
        </div>
      </CardBody>
    </Card>
  );
}
```

```tsx
// RecommendationsList - 推奨設定リスト
interface RecommendationsListProps {
  recommendations: ObsSetting[];
}

function RecommendationsList({ recommendations }: RecommendationsListProps) {
  // 優先度順にソート
  const sortedRecs = [...recommendations].sort((a, b) => {
    const priorityOrder = { critical: 0, recommended: 1, optional: 2 };
    return priorityOrder[a.priority] - priorityOrder[b.priority];
  });

  // 優先度ラベルの日本語化
  const getPriorityLabel = (priority: ObsSetting['priority']) => {
    switch (priority) {
      case 'critical': return { text: '重要', color: 'text-red-600', icon: '🔴' };
      case 'recommended': return { text: '推奨', color: 'text-yellow-600', icon: '🟡' };
      case 'optional': return { text: '任意', color: 'text-blue-600', icon: '🔵' };
    }
  };

  // 値を初心者向けに変換
  const formatValue = (value: string | number | boolean): string => {
    if (typeof value === 'boolean') {
      return value ? '有効' : '無効';
    }
    if (typeof value === 'number') {
      // ビットレートなどの数値を読みやすく
      if (value >= 1000) {
        return `${value.toLocaleString()}`;
      }
      return String(value);
    }
    return String(value);
  };

  return (
    <Card>
      <CardHeader>
        <h3>⚠️ 見つかった問題（{recommendations.length}件）</h3>
      </CardHeader>
      <CardBody>
        <div className="space-y-4">
          {sortedRecs.map((rec, index) => {
            const priority = getPriorityLabel(rec.priority);

            return (
              <div
                key={rec.key}
                className="border-l-4 border-gray-300 pl-4 py-2"
                style={{
                  borderLeftColor:
                    rec.priority === 'critical' ? '#dc2626' :
                    rec.priority === 'recommended' ? '#ca8a04' : '#2563eb'
                }}
              >
                {/* タイトル */}
                <div className="flex items-center gap-2 mb-2">
                  <span>{priority.icon}</span>
                  <span className={`font-semibold ${priority.color}`}>
                    [{priority.text}]
                  </span>
                  <span className="font-medium">{rec.displayName}</span>
                </div>

                {/* 現在値 → 推奨値 */}
                <div className="text-sm mb-2">
                  <span className="text-gray-600">現在: </span>
                  <span className="font-mono">{formatValue(rec.currentValue)}</span>
                  <span className="mx-2">→</span>
                  <span className="text-gray-600">おすすめ: </span>
                  <span className="font-mono text-green-600 font-semibold">
                    {formatValue(rec.recommendedValue)}
                  </span>
                </div>

                {/* 理由 */}
                <div className="text-sm text-gray-700">
                  <span className="font-medium">理由: </span>
                  {rec.reason}
                </div>
              </div>
            );
          })}
        </div>
      </CardBody>
    </Card>
  );
}
```

```tsx
// NextStepGuide - 次のステップへの誘導
function NextStepGuide() {
  return (
    <Card variant="info">
      <CardBody>
        <div className="flex items-start gap-3">
          <span className="text-2xl">💡</span>
          <div>
            <h4 className="font-semibold mb-1">次のステップ</h4>
            <p className="text-sm">
              これらの問題は、次のステップで自動的に修正できます。<br />
              あなたのPCに最適な設定を提案します！
            </p>
          </div>
        </div>
      </CardBody>
    </Card>
  );
}
```

#### デザイントークン（Tailwind CSS）
```css
/* 色の定義 */
--color-critical: #dc2626 (red-600)
--color-recommended: #ca8a04 (yellow-600)
--color-optional: #2563eb (blue-600)
--color-success: #16a34a (green-600)

/* スペーシング */
--space-card-padding: 1.5rem (p-6)
--space-section-gap: 1.5rem (space-y-6)
--space-item-gap: 1rem (space-y-4)
```

---

### 提案2: AutoOptimize.tsx の詳細表示追加

#### 改善前（現状推測）
```
┌──────────────────────────────────────┐
│ 推奨設定を適用                        │
│                                      │
│ プリセットを選択してください:         │
│ ○ 軽量（低負荷）                      │
│ ● 標準（推奨）                        │
│ ○ 高品質                             │
│ ○ 最高品質                           │
│                                      │
│ [適用する]                           │
└──────────────────────────────────────┘
```

#### 改善後（推奨デザイン）
```
┌────────────────────────────────────────────────────────────────┐
│ ⚙️ あなたに最適な設定を適用                                     │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│ プリセットを選択してください:                                   │
│                                                                │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │ ○ 軽量（低負荷モード）                                      │ │
│ │   480p 30fps、ビットレート1500                              │ │
│ │   スコア予測: 75点                                          │ │
│ │   📌 低スペックPCや、安定性を最優先したい場合におすすめ      │ │
│ └────────────────────────────────────────────────────────────┘ │
│                                                                │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │ ● 標準（あなたにおすすめ！） ⭐                             │ │
│ │   720p 30fps、ビットレート2500                              │ │
│ │   スコア予測: 85点                                          │ │
│ │   📌 RTX 3060に最適。画質と安定性のバランスが良い設定       │ │
│ └────────────────────────────────────────────────────────────┘ │
│                                                                │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │ ○ 高品質                                                    │ │
│ │   1080p 30fps、ビットレート4000                             │ │
│ │   スコア予測: 70点                                          │ │
│ │   📌 高画質配信したい場合。ネットワーク回線が速い必要あり    │ │
│ └────────────────────────────────────────────────────────────┘ │
│                                                                │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │ ○ 最高品質                                                  │ │
│ │   1080p 60fps、ビットレート6000                             │ │
│ │   スコア予測: 55点                                          │ │
│ │   ⚠️ あなたのネットワーク環境では推奨されません              │ │
│ └────────────────────────────────────────────────────────────┘ │
│                                                                │
│ ─────────────────────────────────────────────────────────────  │
│                                                                │
│ 🔍 選択中のプリセット詳細: 標準                                │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │ 📹 映像設定                                                 │ │
│ │   • 解像度: 1280x720 (現在: 1920x1080から変更)            │ │
│ │   • フレームレート: 30fps (現在: 60fpsから変更)            │ │
│ │   • フィルター: Lanczos（高品質な縮小）                    │ │
│ │                                                            │ │
│ │ 🎵 音声設定                                                 │ │
│ │   • サンプルレート: 48000Hz                                │ │
│ │   • ビットレート: 160kbps                                  │ │
│ │                                                            │ │
│ │ 🎬 エンコード設定                                           │ │
│ │   • エンコーダー: NVENC H.264 (RTX 3060のGPU使用)         │ │
│ │   • ビットレート: 2500kbps (現在: 6000kbpsから変更)       │ │
│ │   • プリセット: Quality（画質優先）                        │ │
│ │   • レート制御: CBR（固定ビットレート）                    │ │
│ └────────────────────────────────────────────────────────────┘ │
│                                                                │
│ 💡 この設定を選ぶ理由:                                          │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │ ✓ RTX 3060のNVENCを使用することで、CPUへの負荷を           │ │
│ │   大幅に削減できます                                       │ │
│ │ ✓ 720p 30fpsは安定した配信に最適なバランスです             │ │
│ │ ✓ ビットレート2500kbpsはYouTube配信の推奨範囲内です        │ │
│ └────────────────────────────────────────────────────────────┘ │
│                                                                │
│ 📊 変更内容のまとめ:                                            │
│ ┌────────────────────────────────────────────────────────────┐ │
│ │ 解像度:       1080p → 720p  (画質: 少し下がる)            │ │
│ │ フレームレート: 60fps → 30fps (滑らかさ: 少し下がる)       │ │
│ │ ビットレート:  6000 → 2500   (配信の安定性: 大幅に向上)   │ │
│ │ CPU負荷:      高い → 低い    (約40%削減)                  │ │
│ └────────────────────────────────────────────────────────────┘ │
│                                                                │
│ ℹ️ ご注意:                                                      │
│ • 適用後も、いつでも元の設定に戻せます                          │ │
│ • まずは1回配信して、問題なければこの設定をキープしましょう    │ │
│                                                                │
│                                  [詳細を見る] [戻る] [適用する] │
└────────────────────────────────────────────────────────────────┘
```

#### 実装仕様

**コンポーネント構造**:
```tsx
// src/features/onboarding/AutoOptimize.tsx
function AutoOptimize() {
  const [selectedPreset, setSelectedPreset] = useState<OptimizationPreset>('medium');
  const [currentSettings, setCurrentSettings] = useState<ObsSettings | null>(null);
  const [recommendations, setRecommendations] = useState<Record<OptimizationPreset, RecommendedSettings>>({});
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadData = async () => {
      // 現在の設定を取得
      const current = await invoke<ObsSettings>('get_obs_settings_command');
      setCurrentSettings(current);

      // 各プリセットの推奨設定を取得（並列）
      const presets: OptimizationPreset[] = ['low', 'medium', 'high', 'ultra'];
      const results = await Promise.all(
        presets.map(async (preset) => {
          // プリセットに応じたパラメータで算出
          const rec = await invoke<RecommendedSettings>('calculate_custom_recommendations', {
            platform: 'youTube',
            style: 'talk',
            networkSpeedMbps: getNetworkSpeedForPreset(preset)
          });
          return { preset, rec };
        })
      );

      const recMap = results.reduce((acc, { preset, rec }) => {
        acc[preset] = rec;
        return acc;
      }, {} as Record<OptimizationPreset, RecommendedSettings>);

      setRecommendations(recMap);

      // 最も高スコアのプリセットをデフォルト選択
      const bestPreset = getBestPreset(recMap);
      setSelectedPreset(bestPreset);

      setLoading(false);
    };

    loadData();
  }, []);

  if (loading) return <LoadingSpinner />;

  return (
    <div className="space-y-6">
      <Header title="あなたに最適な設定を適用" icon="⚙️" />

      <PresetSelector
        presets={['low', 'medium', 'high', 'ultra']}
        selected={selectedPreset}
        onSelect={setSelectedPreset}
        recommendations={recommendations}
      />

      <PresetDetailView
        preset={selectedPreset}
        currentSettings={currentSettings}
        recommendedSettings={recommendations[selectedPreset]}
      />

      <ReasonsCard reasons={recommendations[selectedPreset].reasons} />

      <ChangesSummaryCard
        current={currentSettings}
        recommended={recommendations[selectedPreset]}
      />

      <NoticeCard />

      <ActionButtons
        onBack={handleBack}
        onApply={() => handleApply(selectedPreset)}
      />
    </div>
  );
}

// ヘルパー関数
function getNetworkSpeedForPreset(preset: OptimizationPreset): number {
  switch (preset) {
    case 'low': return 3;
    case 'medium': return 5;
    case 'high': return 10;
    case 'ultra': return 20;
    default: return 5;
  }
}

function getBestPreset(recommendations: Record<OptimizationPreset, RecommendedSettings>): OptimizationPreset {
  let bestPreset: OptimizationPreset = 'medium';
  let bestScore = 0;

  Object.entries(recommendations).forEach(([preset, rec]) => {
    if (rec.overallScore > bestScore) {
      bestScore = rec.overallScore;
      bestPreset = preset as OptimizationPreset;
    }
  });

  return bestPreset;
}
```

**子コンポーネント: PresetSelector**:
```tsx
interface PresetSelectorProps {
  presets: OptimizationPreset[];
  selected: OptimizationPreset;
  onSelect: (preset: OptimizationPreset) => void;
  recommendations: Record<OptimizationPreset, RecommendedSettings>;
}

function PresetSelector({ presets, selected, onSelect, recommendations }: PresetSelectorProps) {
  const getPresetLabel = (preset: OptimizationPreset) => {
    switch (preset) {
      case 'low': return '軽量（低負荷モード）';
      case 'medium': return '標準';
      case 'high': return '高品質';
      case 'ultra': return '最高品質';
      default: return preset;
    }
  };

  const getPresetDescription = (preset: OptimizationPreset) => {
    switch (preset) {
      case 'low':
        return '低スペックPCや、安定性を最優先したい場合におすすめ';
      case 'medium':
        return 'RTX 3060に最適。画質と安定性のバランスが良い設定';
      case 'high':
        return '高画質配信したい場合。ネットワーク回線が速い必要あり';
      case 'ultra':
        return 'ハイエンドPC専用。超高画質配信';
      default:
        return '';
    }
  };

  const getPresetSummary = (preset: OptimizationPreset, rec: RecommendedSettings): string => {
    return `${rec.video.outputHeight}p ${rec.video.fps}fps、ビットレート${rec.output.bitrateKbps}`;
  };

  // 最高スコアのプリセットを判定
  const bestPreset = Object.entries(recommendations).reduce(
    (best, [preset, rec]) => rec.overallScore > best.score
      ? { preset: preset as OptimizationPreset, score: rec.overallScore }
      : best,
    { preset: 'medium' as OptimizationPreset, score: 0 }
  ).preset;

  return (
    <div>
      <label className="block text-sm font-medium mb-4">
        プリセットを選択してください:
      </label>

      <div className="space-y-3">
        {presets.map((preset) => {
          const rec = recommendations[preset];
          if (!rec) return null;

          const isSelected = preset === selected;
          const isBest = preset === bestPreset;
          const isNotRecommended = rec.overallScore < 60;

          return (
            <button
              key={preset}
              onClick={() => onSelect(preset)}
              className={`
                w-full text-left p-4 rounded-lg border-2 transition-all
                ${isSelected
                  ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                  : 'border-gray-300 hover:border-gray-400 bg-white dark:bg-gray-800'
                }
                ${isNotRecommended ? 'opacity-75' : ''}
              `}
            >
              <div className="flex items-start gap-3">
                {/* ラジオボタン */}
                <div className="mt-1">
                  <div className={`
                    w-5 h-5 rounded-full border-2 flex items-center justify-center
                    ${isSelected ? 'border-blue-500' : 'border-gray-400'}
                  `}>
                    {isSelected && (
                      <div className="w-3 h-3 rounded-full bg-blue-500" />
                    )}
                  </div>
                </div>

                <div className="flex-1">
                  {/* タイトル */}
                  <div className="flex items-center gap-2 mb-1">
                    <span className="font-semibold">
                      {getPresetLabel(preset)}
                    </span>
                    {isBest && (
                      <span className="text-xs bg-yellow-100 text-yellow-800 px-2 py-0.5 rounded">
                        あなたにおすすめ！ ⭐
                      </span>
                    )}
                  </div>

                  {/* サマリー */}
                  <div className="text-sm text-gray-600 dark:text-gray-400 mb-1">
                    {getPresetSummary(preset, rec)}
                  </div>

                  {/* スコア */}
                  <div className="text-sm mb-2">
                    <span className="text-gray-500">スコア予測: </span>
                    <span className={`font-semibold ${
                      rec.overallScore >= 80 ? 'text-green-600' :
                      rec.overallScore >= 60 ? 'text-yellow-600' : 'text-red-600'
                    }`}>
                      {rec.overallScore}点
                    </span>
                  </div>

                  {/* 説明 */}
                  <div className="text-sm text-gray-700 dark:text-gray-300">
                    <span>📌 </span>
                    {isNotRecommended ? (
                      <span className="text-red-600">
                        ⚠️ あなたのネットワーク環境では推奨されません
                      </span>
                    ) : (
                      getPresetDescription(preset)
                    )}
                  </div>
                </div>
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
```

**子コンポーネント: PresetDetailView**:
```tsx
interface PresetDetailViewProps {
  preset: OptimizationPreset;
  currentSettings: ObsSettings | null;
  recommendedSettings: RecommendedSettings;
}

function PresetDetailView({ preset, currentSettings, recommendedSettings }: PresetDetailViewProps) {
  const getPresetLabel = (p: OptimizationPreset) => {
    // 同じロジック
  };

  return (
    <Card>
      <CardHeader>
        <h3>🔍 選択中のプリセット詳細: {getPresetLabel(preset)}</h3>
      </CardHeader>
      <CardBody className="space-y-4">
        {/* 映像設定 */}
        <div>
          <h4 className="font-semibold mb-2">📹 映像設定</h4>
          <ul className="space-y-1 text-sm">
            <SettingItem
              label="解像度"
              value={`${recommendedSettings.video.outputWidth}x${recommendedSettings.video.outputHeight}`}
              currentValue={currentSettings ? `${currentSettings.video.outputWidth}x${currentSettings.video.outputHeight}` : null}
            />
            <SettingItem
              label="フレームレート"
              value={`${recommendedSettings.video.fps}fps`}
              currentValue={currentSettings ? `${currentSettings.video.fpsNumerator}fps` : null}
            />
            <SettingItem
              label="フィルター"
              value={`${recommendedSettings.video.downscaleFilter}（高品質な縮小）`}
            />
          </ul>
        </div>

        {/* 音声設定 */}
        <div>
          <h4 className="font-semibold mb-2">🎵 音声設定</h4>
          <ul className="space-y-1 text-sm">
            <SettingItem
              label="サンプルレート"
              value={`${recommendedSettings.audio.sampleRate}Hz`}
            />
            <SettingItem
              label="ビットレート"
              value={`${recommendedSettings.audio.bitrateKbps}kbps`}
            />
          </ul>
        </div>

        {/* エンコード設定 */}
        <div>
          <h4 className="font-semibold mb-2">🎬 エンコード設定</h4>
          <ul className="space-y-1 text-sm">
            <SettingItem
              label="エンコーダー"
              value={`${getEncoderDisplayName(recommendedSettings.output.encoder)} (RTX 3060のGPU使用)`}
              currentValue={currentSettings ? currentSettings.output.encoder : null}
            />
            <SettingItem
              label="ビットレート"
              value={`${recommendedSettings.output.bitrateKbps}kbps`}
              currentValue={currentSettings ? `${currentSettings.output.bitrateKbps}kbps` : null}
            />
            <SettingItem
              label="プリセット"
              value={`${recommendedSettings.output.preset}（画質優先）`}
            />
            <SettingItem
              label="レート制御"
              value={`${recommendedSettings.output.rateControl}（固定ビットレート）`}
            />
          </ul>
        </div>
      </CardBody>
    </Card>
  );
}

// ヘルパー: 設定項目の表示
interface SettingItemProps {
  label: string;
  value: string;
  currentValue?: string | null;
}

function SettingItem({ label, value, currentValue }: SettingItemProps) {
  const hasChanged = currentValue && currentValue !== value;

  return (
    <li className="flex items-start gap-2">
      <span className="text-gray-600">• {label}: </span>
      <span>
        {value}
        {hasChanged && (
          <span className="text-xs text-gray-500 ml-2">
            (現在: {currentValue}から変更)
          </span>
        )}
      </span>
    </li>
  );
}

function getEncoderDisplayName(encoder: string): string {
  if (encoder.includes('nvenc')) return 'NVENC H.264';
  if (encoder.includes('quicksync')) return 'Intel Quick Sync';
  if (encoder.includes('amd')) return 'AMD VCE';
  if (encoder.includes('x264')) return 'x264 (ソフトウェア)';
  return encoder;
}
```

**子コンポーネント: ReasonsCard**:
```tsx
interface ReasonsCardProps {
  reasons: string[];
}

function ReasonsCard({ reasons }: ReasonsCardProps) {
  return (
    <Card variant="info">
      <CardHeader>
        <h3>💡 この設定を選ぶ理由:</h3>
      </CardHeader>
      <CardBody>
        <ul className="space-y-2 text-sm">
          {reasons.map((reason, index) => (
            <li key={index} className="flex items-start gap-2">
              <span className="text-green-600">✓</span>
              <span>{reason}</span>
            </li>
          ))}
        </ul>
      </CardBody>
    </Card>
  );
}
```

**子コンポーネント: ChangesSummaryCard**:
```tsx
interface ChangesSummaryCardProps {
  current: ObsSettings | null;
  recommended: RecommendedSettings;
}

function ChangesSummaryCard({ current, recommended }: ChangesSummaryCardProps) {
  if (!current) return null;

  const changes = [
    {
      label: '解像度',
      from: `${current.video.outputWidth}x${current.video.outputHeight}`,
      to: `${recommended.video.outputWidth}x${recommended.video.outputHeight}`,
      impact: '画質: 少し下がる'
    },
    {
      label: 'フレームレート',
      from: `${current.video.fpsNumerator}fps`,
      to: `${recommended.video.fps}fps`,
      impact: '滑らかさ: 少し下がる'
    },
    {
      label: 'ビットレート',
      from: String(current.output.bitrateKbps),
      to: String(recommended.output.bitrateKbps),
      impact: '配信の安定性: 大幅に向上'
    },
    {
      label: 'CPU負荷',
      from: '高い',
      to: '低い',
      impact: '約40%削減'
    }
  ];

  return (
    <Card>
      <CardHeader>
        <h3>📊 変更内容のまとめ:</h3>
      </CardHeader>
      <CardBody>
        <table className="w-full text-sm">
          <tbody>
            {changes.map((change, index) => (
              <tr key={index} className="border-b last:border-b-0">
                <td className="py-2 font-medium">{change.label}:</td>
                <td className="py-2 text-gray-600">{change.from} → {change.to}</td>
                <td className="py-2 text-gray-500">({change.impact})</td>
              </tr>
            ))}
          </tbody>
        </table>
      </CardBody>
    </Card>
  );
}
```

---

### 提案3: ダッシュボードに「おすすめ設定」パネル追加

#### 配置場所
```tsx
// App.tsx の DashboardTab に追加
function DashboardTab() {
  return (
    <div className="space-y-6">
      <ObsConnectionPanel />

      {/* 新規追加: おすすめ設定パネル */}
      <RecommendedSettingsPanel />

      <section className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="space-y-6">
          <ObsStatusIndicator />
          <ObsStreamControls />
        </div>
        <div>
          <MetricsPanel />
        </div>
      </section>

      <ObsSceneSelector />
    </div>
  );
}
```

#### RecommendedSettingsPanel デザイン
```
┌────────────────────────────────────────────────────────────┐
│ ⭐ あなたのPCにおすすめの設定                                │
├────────────────────────────────────────────────────────────┤
│                                                            │
│ 💻 現在適用中の設定                                         │
│ ┌────────────────────────────────────────────────────────┐ │
│ │ 解像度: 720p 30fps │ ビットレート: 2500kbps             │ │
│ │ エンコーダー: NVENC H.264 (GPUエンコード)              │ │
│ │ 品質スコア: 85点  ✅ この設定はあなたのPCに最適です      │ │
│ └────────────────────────────────────────────────────────┘ │
│                                                            │
│ 📊 現在のパフォーマンス                                     │
│ ┌────────────────────────────────────────────────────────┐ │
│ │ CPU使用率:   45%  [████████████░░░░░░░░] (推奨: 60%以下)│ │
│ │ GPU使用率:   60%  [████████████████░░░░] (推奨: 70%以下)│ │
│ │ メモリ使用率: 50%  [██████████████░░░░░░] (推奨: 80%以下)│ │
│ │                                                        │ │
│ │ ✅ すべての値が推奨範囲内です。配信に問題ありません      │ │
│ └────────────────────────────────────────────────────────┘ │
│                                                            │
│                     [設定を再分析] [最適化タブを開く]       │
└────────────────────────────────────────────────────────────┘
```

#### 実装仕様
```tsx
// src/features/dashboard/RecommendedSettingsPanel.tsx
function RecommendedSettingsPanel() {
  const [currentSettings, setCurrentSettings] = useState<ObsSettings | null>(null);
  const [recommendations, setRecommendations] = useState<RecommendedSettings | null>(null);
  const metrics = useMetricsStore((state) => state.metrics);
  const navigate = useNavigate(); // タブ遷移用

  useEffect(() => {
    const loadSettings = async () => {
      const settings = await invoke<ObsSettings>('get_obs_settings_command');
      setCurrentSettings(settings);

      const recs = await invoke<RecommendedSettings>('calculate_recommendations');
      setRecommendations(recs);
    };

    loadSettings();
  }, []);

  const handleReanalyze = async () => {
    // 再分析
    const recs = await invoke<RecommendedSettings>('calculate_recommendations');
    setRecommendations(recs);
  };

  const handleGoToOptimization = () => {
    // 最適化タブに遷移（App.tsxのsetActiveTab経由）
    navigate('/optimization');
  };

  if (!currentSettings || !recommendations) {
    return <LoadingCard />;
  }

  return (
    <Card>
      <CardHeader>
        <h2 className="flex items-center gap-2">
          <span>⭐</span>
          <span>あなたのPCにおすすめの設定</span>
        </h2>
      </CardHeader>
      <CardBody className="space-y-4">
        {/* 現在の設定サマリー */}
        <CurrentSettingsSummary
          settings={currentSettings}
          score={recommendations.overallScore}
        />

        {/* パフォーマンスメトリクス */}
        {metrics && (
          <PerformanceMetricsWithRecommendations metrics={metrics} />
        )}

        {/* アクションボタン */}
        <div className="flex gap-2 justify-end">
          <Button variant="outline" onClick={handleReanalyze}>
            設定を再分析
          </Button>
          <Button variant="primary" onClick={handleGoToOptimization}>
            最適化タブを開く
          </Button>
        </div>
      </CardBody>
    </Card>
  );
}
```

**子コンポーネント: CurrentSettingsSummary**:
```tsx
interface CurrentSettingsSummaryProps {
  settings: ObsSettings;
  score: number;
}

function CurrentSettingsSummary({ settings, score }: CurrentSettingsSummaryProps) {
  const isGoodScore = score >= 71;

  return (
    <div>
      <h3 className="font-semibold mb-2">💻 現在適用中の設定</h3>
      <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg space-y-2">
        <div className="flex flex-wrap gap-4 text-sm">
          <div>
            <span className="text-gray-600">解像度: </span>
            <span className="font-mono">
              {settings.video.outputWidth}x{settings.video.outputHeight} {settings.video.fpsNumerator}fps
            </span>
          </div>
          <div>
            <span className="text-gray-600">ビットレート: </span>
            <span className="font-mono">{settings.output.bitrateKbps}kbps</span>
          </div>
        </div>

        <div className="text-sm">
          <span className="text-gray-600">エンコーダー: </span>
          <span>{getEncoderDisplayName(settings.output.encoder)} (GPUエンコード)</span>
        </div>

        <div className="flex items-center gap-2 text-sm mt-2">
          <span className="text-gray-600">品質スコア: </span>
          <span className={`font-bold ${isGoodScore ? 'text-green-600' : 'text-yellow-600'}`}>
            {score}点
          </span>
          {isGoodScore && (
            <span className="text-green-600">
              ✅ この設定はあなたのPCに最適です
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
```

**子コンポーネント: PerformanceMetricsWithRecommendations**:
```tsx
interface PerformanceMetricsWithRecommendationsProps {
  metrics: SystemMetrics;
}

function PerformanceMetricsWithRecommendations({ metrics }: PerformanceMetricsWithRecommendationsProps) {
  const thresholds = {
    cpu: 60,
    gpu: 70,
    memory: 80
  };

  const cpuOk = metrics.cpu.usagePercent <= thresholds.cpu;
  const gpuOk = !metrics.gpu || metrics.gpu.usagePercent <= thresholds.gpu;
  const memoryOk = metrics.memory.usagePercent <= thresholds.memory;

  const allOk = cpuOk && gpuOk && memoryOk;

  return (
    <div>
      <h3 className="font-semibold mb-2">📊 現在のパフォーマンス</h3>
      <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg space-y-3">
        {/* CPU */}
        <MetricRow
          label="CPU使用率"
          value={metrics.cpu.usagePercent}
          threshold={thresholds.cpu}
          unit="%"
        />

        {/* GPU */}
        {metrics.gpu && (
          <MetricRow
            label="GPU使用率"
            value={metrics.gpu.usagePercent}
            threshold={thresholds.gpu}
            unit="%"
          />
        )}

        {/* メモリ */}
        <MetricRow
          label="メモリ使用率"
          value={metrics.memory.usagePercent}
          threshold={thresholds.memory}
          unit="%"
        />

        {/* ステータスメッセージ */}
        <div className="mt-3 pt-3 border-t border-gray-200 dark:border-gray-700">
          {allOk ? (
            <p className="text-sm text-green-600 flex items-center gap-2">
              <span>✅</span>
              <span>すべての値が推奨範囲内です。配信に問題ありません</span>
            </p>
          ) : (
            <p className="text-sm text-yellow-600 flex items-center gap-2">
              <span>⚠️</span>
              <span>一部のメトリクスが推奨範囲を超えています。設定の見直しを検討してください</span>
            </p>
          )}
        </div>
      </div>
    </div>
  );
}

interface MetricRowProps {
  label: string;
  value: number;
  threshold: number;
  unit: string;
}

function MetricRow({ label, value, threshold, unit }: MetricRowProps) {
  const percentage = Math.min((value / 100) * 100, 100);
  const isOk = value <= threshold;

  return (
    <div>
      <div className="flex justify-between text-sm mb-1">
        <span className="text-gray-600">{label}:</span>
        <span className={`font-semibold ${isOk ? 'text-green-600' : 'text-yellow-600'}`}>
          {value.toFixed(1)}{unit}
        </span>
      </div>

      <div className="flex items-center gap-2">
        {/* プログレスバー */}
        <div className="flex-1 bg-gray-200 dark:bg-gray-700 rounded-full h-3">
          <div
            className={`h-3 rounded-full transition-all ${
              isOk ? 'bg-green-500' : 'bg-yellow-500'
            }`}
            style={{ width: `${percentage}%` }}
          />
        </div>

        {/* 推奨値 */}
        <span className="text-xs text-gray-500 whitespace-nowrap">
          (推奨: {threshold}{unit}以下)
        </span>
      </div>
    </div>
  );
}
```

---

## 6. 専門用語の平易化ガイドライン

### 用語変換テーブル

| 専門用語 | 初心者向け表現 | 補足説明（ツールチップ） |
|---------|---------------|-------------------------|
| ビットレート | データ量 | 1秒間に送るデータの量。数値が大きいほど高画質ですが、ネットワーク回線が遅いと配信が途切れます |
| エンコーダー | 変換方式 | 映像を配信用のデータに変換する仕組み。GPUを使うとPCへの負荷が減ります |
| NVENC | GPUエンコード | NVIDIA製GPUの高速変換機能 |
| 解像度 | 画面のサイズ | 1080pは高画質、720pは標準画質。数値が小さいほどPCへの負荷が減ります |
| FPS (フレームレート) | 滑らかさ | 1秒間の画面の枚数。60fpsは滑らか、30fpsは標準 |
| CBR | 固定データ量 | データ量を一定に保つ方式。配信サイトが推奨している安定した方式です |
| Lanczos | 高品質フィルター | 解像度を下げるときに画質を保つ技術 |
| CPU使用率 | PCの脳みその忙しさ | 100%に近いとPCが重くなります。60%以下が理想 |
| GPU使用率 | グラボの忙しさ | 映像処理を担当。70%以下が理想 |
| メモリ使用率 | 作業台の使用状況 | 80%以下が理想。超えるとPCが不安定になります |
| フレームドロップ | コマ落ち | 映像がカクカクする現象。視聴者に悪影響 |

### 表示例
```tsx
// ツールチップ付き用語表示コンポーネント
interface TermWithTooltipProps {
  term: string;
  simple: string;
  tooltip: string;
}

function TermWithTooltip({ term, simple, tooltip }: TermWithTooltipProps) {
  return (
    <span className="inline-flex items-center gap-1">
      <span>{simple}</span>
      <Tooltip content={tooltip}>
        <button className="text-xs text-gray-400 hover:text-gray-600">
          <InfoIcon className="w-3 h-3" />
        </button>
      </Tooltip>
    </span>
  );
}

// 使用例
<TermWithTooltip
  term="bitrate"
  simple="データ量"
  tooltip="1秒間に送るデータの量。数値が大きいほど高画質ですが、ネットワーク回線が遅いと配信が途切れます"
/>
```

---

## 7. 実装優先順位とロードマップ

### Phase 1: 必須改善（US-001, US-002, US-003達成）

| タスク | 対象ファイル | 工数見積 | 依存関係 |
|-------|------------|---------|---------|
| 1-1. Analysis.tsx 全面刷新 | `src/features/onboarding/Analysis.tsx` | 8h | なし |
| 1-2. 共通UIコンポーネント作成 | `src/components/Card.tsx`, `Alert.tsx` | 4h | なし |
| 1-3. 用語変換ユーティリティ | `src/utils/terminology.ts` | 2h | なし |
| 1-4. AutoOptimize.tsx 詳細表示追加 | `src/features/onboarding/AutoOptimize.tsx` | 6h | 1-2 |

**合計工数**: 20h（約3日）

### Phase 2: 推奨改善（US-004達成、UX向上）

| タスク | 対象ファイル | 工数見積 | 依存関係 |
|-------|------------|---------|---------|
| 2-1. RecommendedSettingsPanel作成 | `src/features/dashboard/RecommendedSettingsPanel.tsx` | 5h | Phase 1完了 |
| 2-2. MetricsPanel拡張（推奨値比較） | `src/features/monitor/components/MetricsPanel.tsx` | 3h | 2-1 |
| 2-3. App.tsx ダッシュボード統合 | `src/App.tsx` | 2h | 2-1, 2-2 |

**合計工数**: 10h（約1.5日）

### Phase 3: 洗練（UX最適化）

| タスク | 対象ファイル | 工数見積 | 依存関係 |
|-------|------------|---------|---------|
| 3-1. ツールチップコンポーネント | `src/components/Tooltip.tsx` | 3h | なし |
| 3-2. 全画面への用語置換適用 | 全コンポーネント | 4h | 3-1 |
| 3-3. アニメーション追加 | 各コンポーネント | 3h | Phase 2完了 |
| 3-4. ユーザビリティテスト | - | 4h | Phase 2完了 |

**合計工数**: 14h（約2日）

**総工数**: 44h（約6.5日）

---

## 8. 成果物チェックリスト

### デザイン成果物
- [x] UXギャップ分析レポート（本ドキュメント）
- [ ] ワイヤーフレーム（Figma or 手書き）
- [ ] デザインシステム定義（色、タイポグラフィ、スペーシング）
- [ ] コンポーネント仕様書

### 実装成果物（フロントエンド担当）
- [ ] Analysis.tsx リニューアル
- [ ] AutoOptimize.tsx 詳細表示追加
- [ ] RecommendedSettingsPanel 新規作成
- [ ] 共通UIコンポーネント（Card, Alert, Tooltip）
- [ ] 用語変換ユーティリティ

### 実装成果物（バックエンド担当）
- [ ] （不要 - 既存APIで対応可能）

### ドキュメント
- [x] UX改善提案書
- [ ] ユーザーテストシナリオ
- [ ] 実装ガイドライン

---

## 9. ユーザビリティテスト計画

### テストシナリオ: 初心者VTuber「あかり」のタスク

#### シナリオ1: 初回起動から推奨設定適用まで
```
前提条件: ツールを初めて起動する

期待される行動フロー:
1. オンボーディングウィザードが表示される
2. OBS接続の手順が理解できる
3. 分析結果で自分のPCスペックを確認できる
4. 「3件の問題」の内容を理解できる
5. 推奨設定の詳細を確認できる
6. 安心して「適用する」ボタンを押せる
7. 適用完了のフィードバックを受け取る

成功基準:
- すべてのステップを10分以内に完了できる
- 「怖い」「分からない」と感じる箇所がない
- 適用後に「これでいいのか」という不安がない
```

#### シナリオ2: ダッシュボードでの設定確認
```
前提条件: オンボーディング完了後、1週間経過

期待される行動フロー:
1. ダッシュボードを開く
2. 現在の設定を一目で確認できる
3. パフォーマンスメトリクスが推奨範囲内か確認できる
4. 必要に応じて再分析できる

成功基準:
- 現在の設定を5秒以内に確認できる
- メトリクスが正常かどうか判断できる
```

### テスト手法
- **ユーザビリティテスト**: 初心者3名に実際に操作してもらう
- **ヒューリスティック評価**: UX専門家によるレビュー
- **A/Bテスト**: 旧UIと新UIの比較（スコア表示のみ vs 詳細表示）

### 評価指標
| 指標 | 目標値 |
|------|--------|
| タスク完了率 | 100% |
| タスク完了時間 | 10分以内 |
| 主観的満足度（5段階） | 4.0以上 |
| 「分からない」と感じた回数 | 0回 |
| ヘルプドキュメント参照回数 | 0回（UIだけで理解できる） |

---

## 10. リスクと対策

### リスク1: 情報過多による認知負荷の増加
**内容**: 詳細情報を表示しすぎて、逆に初心者が混乱する

**対策**:
- デフォルトは必要最小限の情報のみ表示
- 「詳細を見る」ボタンで段階的開示
- 重要度に応じた視覚的優先順位（色、サイズ、配置）

### リスク2: バックエンドデータの不足
**内容**: `systemInfo`や`recommendations`が期待通りに返ってこない

**対策**:
- フロントエンドでのフォールバック処理
- ローディング状態とエラー状態の適切なハンドリング
- モックデータでの開発（Storybook活用）

### リスク3: 用語の平易化による正確性の低下
**内容**: 専門用語を簡略化しすぎて、意味が不正確になる

**対策**:
- ツールチップで正式な用語も併記
- 用語変換テーブルを専門家がレビュー
- 上級者向けに「専門用語モード」を用意（設定で切り替え可能）

---

## 11. まとめ

### 現状の問題（3つの重大ギャップ）
1. **Analysis.tsx**: スペック・推奨設定が表示されず、初心者が判断できない
2. **AutoOptimize.tsx**: プリセットの中身が見えず、何が変わるか分からない
3. **ダッシュボード**: オンボーディング後に推奨設定を確認する手段がない

### 改善提案（3つのソリューション）
1. **Analysis.tsx 全面刷新**: PCスペック表示 + 推奨設定詳細 + スコア解説
2. **AutoOptimize.tsx 詳細表示**: プリセット比較 + 設定内容可視化 + 変更影響説明
3. **RecommendedSettingsPanel 新規作成**: ダッシュボードに常設 + メトリクス比較

### 期待される効果
- **US-001達成**: 各ステップで次に何をすべきか明確に分かる
- **US-002達成**: 専門用語を平易な日本語に置換 + ツールチップで補足
- **US-003達成**: 問題の内容と解決策が具体的に分かる
- **US-004達成**: プリセットの詳細を確認した上で安心して適用できる

### 次のアクション
1. **フロントエンド開発者**: Phase 1（Analysis.tsx, AutoOptimize.tsx）の実装開始
2. **バックエンド開発者**: `analyze_settings`と`calculate_recommendations`のレスポンス確認
3. **デザイナー**: Figmaでのワイヤーフレーム作成（オプション）
4. **プロジェクトマネージャー**: ユーザビリティテスト計画の策定

---

## 付録A: データフロー図

```
┌─────────────────────────────────────────────────────────────┐
│ オンボーディングフロー（改善後）                              │
└─────────────────────────────────────────────────────────────┘

Step 1: Welcome
   ↓
Step 2: OBS接続
   ↓
   invoke('connect_obs', params)
   ↓
Step 3: Analysis (改善対象)
   ↓
   invoke('analyze_settings')
   ↓
   AnalysisResult {
     systemInfo: { cpuModel, gpuModel, totalMemoryMb, ... }
     recommendations: ObsSetting[]
     qualityScore: number
     issueCount: number
   }
   ↓
   表示:
   - PCスペックカード
   - スコア可視化
   - 推奨設定リスト（詳細付き）
   ↓
Step 4: AutoOptimize (改善対象)
   ↓
   invoke('calculate_recommendations') （各プリセット）
   ↓
   RecommendedSettings {
     video: { outputWidth, outputHeight, fps, ... }
     audio: { sampleRate, bitrateKbps }
     output: { encoder, bitrateKbps, ... }
     reasons: string[]
     overallScore: number
   }
   ↓
   表示:
   - プリセット比較（スコア付き）
   - 選択中プリセットの詳細
   - 推奨理由
   - 変更内容サマリー
   ↓
   invoke('apply_optimization', { preset })
   ↓
Step 5: 完了
```

---

## 付録B: コンポーネント階層図

```
Analysis.tsx (改善後)
├── Header
├── SystemSpecCard
│   ├── SpecRow (CPU)
│   ├── SpecRow (GPU)
│   ├── SpecRow (メモリ)
│   └── Alert (スペック判定)
├── QualityScoreCard
│   ├── スコアバー（プログレスバー）
│   ├── スコアレンジ凡例
│   └── Alert (評価メッセージ)
├── RecommendationsList
│   └── RecommendationItem[] (ObsSetting毎)
│       ├── 優先度バッジ
│       ├── 現在値 → 推奨値
│       └── 理由テキスト
└── NextStepGuide

AutoOptimize.tsx (改善後)
├── Header
├── PresetSelector
│   └── PresetOption[] (low, medium, high, ultra)
│       ├── ラジオボタン
│       ├── タイトル + おすすめバッジ
│       ├── サマリー（解像度、ビットレート）
│       ├── スコア予測
│       └── 説明文
├── PresetDetailView
│   ├── 映像設定セクション
│   │   └── SettingItem[]
│   ├── 音声設定セクション
│   │   └── SettingItem[]
│   └── エンコード設定セクション
│       └── SettingItem[]
├── ReasonsCard
│   └── 理由リスト（reasons[]）
├── ChangesSummaryCard
│   └── 変更内容テーブル
├── NoticeCard
└── ActionButtons

RecommendedSettingsPanel.tsx (新規)
├── Header
├── CurrentSettingsSummary
│   ├── 設定サマリー（解像度、ビットレート等）
│   └── スコア表示
├── PerformanceMetricsWithRecommendations
│   ├── MetricRow (CPU)
│   ├── MetricRow (GPU)
│   ├── MetricRow (メモリ)
│   └── ステータスメッセージ
└── ActionButtons
```

---

## 付録C: 型定義の整合性チェック

### 必要な型（すべて`src/types/commands.ts`に存在）

- [x] `AnalysisResult`
- [x] `SystemInfo`
- [x] `ObsSetting`
- [x] `RecommendedSettings`
- [x] `ObsSettings`
- [x] `SystemMetrics`
- [x] `OptimizationPreset`

### バックエンドコマンド（すべて`contracts/api.md`に定義済み）

- [x] `analyze_settings(): Promise<AnalysisResult>`
- [x] `calculate_recommendations(): Promise<RecommendedSettings>`
- [x] `calculate_custom_recommendations(params): Promise<RecommendedSettings>`
- [x] `get_obs_settings_command(): Promise<ObsSettings>`
- [x] `apply_optimization(params): Promise<OptimizationResult>`

**結論**: バックエンド側の変更は不要。フロントエンドのUI実装のみで対応可能。

---

**作成者**: UI/UX Designer (Claude Agent)
**レビュー依頼先**: frontend-developer, backend-architect, product-owner
**最終更新**: 2025-12-24
