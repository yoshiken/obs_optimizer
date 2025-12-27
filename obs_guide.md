# OBS設定とPCスペックの関係性ガイド

> **更新日**: 2025-12-27
> **目的**: PCスペックに応じたOBS設定の推奨値をまとめたリファレンス

---

## 1. エンコーダー選択（映像エンコーダー）

| スペック条件 | 推奨エンコーダー | 理由 |
|-------------|-----------------|------|
| **RTX 20/30/40/50シリーズ** | NVENC (H.264/HEVC/AV1) | 専用チップで5-10% GPU負荷のみ、x264 slowと同等品質 |
| **GTX 16シリーズ** | NVENC (H.264) | Turing世代のNVENC対応 |
| **AMD RX 6000/7000** | AMF (H.264) | VCN3/4対応、NVENCには劣るが実用的 |
| **Intel Arc** | QSV (AV1) | AV1ハードウェアエンコード対応 |
| **CPU 8コア以上** | x264 (medium) | ソフトエンコードで最高品質、CPU負荷大 |
| **CPU 6コア** | x264 (veryfast) | 品質と負荷のバランス |
| **CPU 4コア以下** | NVENC必須 | x264は負荷が高すぎる |

**参考**:
- [OBS NVENC Guide](https://www.nvidia.com/en-us/geforce/guides/broadcasting-guide/)
- [NVENC vs x264](https://blog.eklipse.gg/tools/nvenc-vs-x264.html)

---

## 2. CPU使用プリセット（x264使用時）

| CPUコア数 | 推奨プリセット | 備考 |
|----------|--------------|------|
| 16コア以上 | slow | 高品質、80%程度CPU使用 |
| 8コア | medium | 品質と負荷のバランス |
| 6コア | veryfast | スイートスポット（OBSデフォルト） |
| 4コア | superfast/ultrafast | 品質低下あり、NVENC推奨 |

**重要**: mediumより遅いプリセットは収穫逓減。配信では差がわかりにくい。

**参考**:
- [OBS x264 Guide](https://obsproject.com/blog/streaming-with-x264)
- [x264 Preset Discussion](https://obsproject.com/forum/threads/what-cpu-can-comfortably-record-in-x264-medium-preset.70284/)

---

## 3. 解像度・FPS・ビットレート

| GPU/CPUティア | 推奨解像度 | 推奨FPS | ビットレート |
|--------------|----------|--------|-------------|
| TierS (RTX 4080/4090/5080) | 1080p / 1440p | 60fps | 8,000-12,000kbps |
| TierA (RTX 4070/3080/3090) | 1080p | 60fps | 6,000-8,000kbps |
| TierB (RTX 3070/4060) | 1080p | 60fps | 6,000kbps |
| TierC (RTX 3060/2070) | 1080p / 720p | 60fps | 4,500-6,000kbps |
| TierD (GTX 1060/CPU only) | 720p | 30fps | 3,500-4,500kbps |

**参考**:
- [Best OBS Settings 2025](https://www.obsbot.com/blog/live-streaming/obs-setting-for-streaming)
- [Dacast OBS Guide](https://www.dacast.com/blog/best-obs-studio-settings/)

---

## 4. メモリ（RAM）

| RAM容量 | 推奨用途 | 注意点 |
|--------|---------|-------|
| 8GB | 720p配信のみ、ブラウザ制限 | 80%超で問題発生 |
| 16GB | 1080p60配信+ゲーム+Discord | ほとんどのユーザーに十分 |
| 32GB | 4K/1440p配信、複数アプリ、AAA級ゲーム | プロ配信者向け |

**影響する設定**: 解像度、同時起動アプリ数、ブラウザソース数

**参考**:
- [OBS RAM Requirements](https://blinksandbuttons.net/how-much-ram-do-you-need-for-obs/)
- [RAM for Streaming 2025](https://battleforgepc.com/article/how-much-ram-do-you-really-need-for-streaming-and-heavy-multitasking-in-2025/)

---

## 5. 縮小フィルタ（Downscale Filter）

| フィルタ | サンプル数 | GPU負荷 | 推奨用途 |
|---------|----------|--------|---------|
| Point | 1 | 最低 | 使用非推奨 |
| Bilinear | 4 | 低 | 低スペック時のみ |
| Bicubic | 16 | 中 | **ゲーム配信（推奨）** |
| Lanczos | 32 | やや高 | カメラ映像、実写 |

**注意**: 50%未満のダウンスケールではOBSがBilinearを強制

**参考**: [Downscale Filter Discussion](https://obsproject.com/forum/threads/bicubic-vs-lanczos-downscale-filter-performance.70407/)

---

## 6. プロセス優先度

| 状況 | 推奨設定 |
|------|---------|
| 通常配信 | Normal（デフォルト） |
| 重いゲーム+配信 | Above Normal |
| x264使用時のエンコーダーラグ | High |
| 最終手段 | Real-time（注意して使用） |

**参考**: [OBS Process Priority](https://obsproject.com/forum/threads/is-there-a-way-to-give-obs-more-process-priority.174910/)

---

## 7. サンプルレート（音声）

| 設定 | 推奨 |
|------|------|
| 48kHz | **推奨**（Windows/デバイスのデフォルト、リサンプル回避） |
| 44.1kHz | 音楽制作環境のみ |

**参考**: [OBS Sample Rate Discussion](https://obsproject.com/forum/threads/what-sample-frequesncy-48-or-44-1-is-better-for-streaming.144291/)

---

## 実装メモ：スペック→設定の統合判断システム

### 現状の実装状況

| コンポーネント | 実装状態 | 詳細 |
|--------------|---------|------|
| GPU判定 | 完了 | 世代×グレード → EffectiveTier |
| CPU判定 | 簡易 | コア数のみで判定 |
| メモリ判定 | 未実装 | 取得のみ、推奨に未反映 |
| UI表示 | 未実装 | スペック→設定の関係表示なし |

### 統合判定の設計案

```
SystemCapability {
    gpu_tier: EffectiveTier (TierS/A/B/C/D/E)
    cpu_tier: CpuTier (High/Mid/Low)
    memory_tier: MemoryTier (High/Mid/Low)
    overall_tier: 最も低いティアに合わせる
}
```

### ティア定義

**CpuTier**:
- High: 8コア以上 → x264 medium可
- Mid: 6コア → x264 veryfast
- Low: 4コア以下 → NVENC必須

**MemoryTier**:
- High: 32GB以上 → 1440p/4K対応
- Mid: 16GB → 1080p60標準
- Low: 8GB以下 → 720p推奨

### 対象ファイル

| ファイル | 変更内容 |
|---------|---------|
| `src-tauri/src/services/optimizer.rs` | CPU/メモリ判定追加、統合スコア計算 |
| `src-tauri/src/services/gpu_detection.rs` | CpuTier, MemoryTierの追加 |
| `src-tauri/src/commands/analyzer.rs` | システム能力情報をAPIレスポンスに追加 |
| `src/types/commands.ts` | 型定義追加 |
| `src/features/optimization/RecommendedSettingsPanel.tsx` | スペック→設定関係表示UI |

---

## スペック非依存の推奨設定

以下はPCスペックに関係なく、ベストプラクティスとして推奨される設定。

### 8. キーフレーム間隔（Keyframe Interval）

| 用途 | 推奨値 | 理由 |
|------|-------|------|
| 配信 | **2秒** | プラットフォーム標準、視聴者が最大2秒で追いつける |
| 録画 | 2-4秒 | 低い値は品質向上、ファイルサイズ増加 |

**参考**: [Dacast OBS Settings](https://www.dacast.com/blog/best-obs-studio-settings/)

---

### 9. レート制御（Rate Control）

| 用途 | 推奨方式 | 理由 |
|------|---------|------|
| 配信 | **CBR** | 一定ビットレートで安定配信、視聴者の回線品質差に対応 |
| 録画 | **CQP/CRF** | 品質ベース、ビットレート無駄なし |

**録画時のCQP値**:
| 値 | 品質 | 用途 |
|----|------|------|
| 22 | 許容範囲 | 一般的 |
| 16 | 高品質/視覚的ロスレス | 推奨 |
| 12 | 最高品質 | 編集用（再エンコード劣化対策） |

**参考**: [Castr Best OBS Settings](https://castr.com/blog/best-obs-settings-streaming-recording/)

---

### 10. 音声ビットレート（Audio Bitrate）

| 用途 | 推奨値 | 備考 |
|------|-------|------|
| トーク配信 | 128-160 Kbps | 最低限の品質 |
| ゲーム配信 | 160 Kbps | プラットフォーム推奨 |
| 音楽配信 | 192-320 Kbps | 高品質サウンド |

**参考**: [OWN3D Best OBS Settings](https://www.own3d.tv/en/blog/obs-studio/best-obs-settings-for-streaming/)

---

### 11. カラーフォーマット・カラースペース

| 設定項目 | SDR配信（推奨） | HDR配信 |
|---------|---------------|---------|
| カラーフォーマット | **NV12** | P010 |
| カラースペース | **Rec.709** | Rec.2100 (PQ) |
| カラーレンジ | **Partial（Limited）** | - |

**注意**:
- NV12 = 4:2:0サブサンプリング（配信では強制）
- I444 = 4:4:4（CPU負荷+3-4%増、録画向け）
- sRGB = PCゲーム録画向け、Rec.709 = それ以外

**参考**: [OBS Color Settings Guide](https://obsproject.com/forum/resources/obs-studio-color-space-color-format-color-range-settings-guide-test-charts.442/)

---

### 12. プロファイル（H.264 Profile）

| プロファイル | 推奨 | 用途 |
|------------|------|------|
| **High** | 推奨 | ハードウェアアクセラレーション最大活用 |
| Main | 予備 | ラグ・フレームドロップ時に下げる |
| Baseline | 非推奨 | ビデオ会議用、品質低い |

**参考**: [NVIDIA NVENC Guide](https://www.nvidia.com/en-us/geforce/guides/broadcasting-guide/)

---

### 13. Bフレーム・Look-ahead・Psycho Visual

| 設定 | 配信 | 録画 | 理由 |
|------|------|------|------|
| **Bフレーム数** | 2 | 4 | 配信は一定値推奨、録画は高品質 |
| **Look-ahead** | OFF | ON | 配信はBフレーム数固定推奨 |
| **Psycho Visual Tuning** | ON | OFF | 配信はビットレート制限下で有効、録画はCQPで不要 |

**高動きコンテンツ（FPS等）**:
- Look-ahead: OFF
- Bフレーム: 2
- Psycho Visual: ON

**低動きコンテンツ**:
- Look-ahead: ON
- Bフレーム: 4
- Psycho Visual: ON

**参考**:
- [Mobcrush Advanced OBS Settings](https://blog.mobcrush.com/advanced-obs-settings-what-they-are-and-how-to-use-them-3bffd9995030)
- [Addie's Analog Emporium](https://www.answeroverflow.com/m/1143966673561460858)

---

## 設定カテゴリ別：スペック依存性まとめ

| 設定 | スペック依存 | 推奨値固定 |
|------|------------|-----------|
| エンコーダー | GPU/CPU | - |
| x264プリセット | CPU | - |
| 解像度・FPS | GPU/CPU/RAM | - |
| ビットレート | GPU Tier | - |
| 縮小フィルタ | GPU | - |
| プロセス優先度 | CPU負荷 | - |
| サンプルレート | - | 48kHz |
| キーフレーム間隔 | - | 2秒 |
| レート制御 | - | CBR（配信）/CQP（録画） |
| 音声ビットレート | - | 160kbps |
| カラーフォーマット | - | NV12 |
| カラースペース | - | Rec.709 |
| プロファイル | - | High |
| Bフレーム | - | 2（配信） |
| Look-ahead | - | OFF（配信） |
| Psycho Visual | - | ON（配信） |
