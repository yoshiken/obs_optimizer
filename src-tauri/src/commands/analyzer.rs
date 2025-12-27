// 問題分析コマンド
//
// システムメトリクスとOBS統計を分析して問題を検出するTauriコマンド

use crate::error::AppError;
use crate::services::analyzer::{ProblemAnalyzer, ProblemReport};
use crate::services::system::system_monitor_service;
use crate::services::optimizer::RecommendationEngine;
use crate::storage::metrics_history::SystemMetricsSnapshot;
use crate::monitor::get_memory_info;
use crate::obs::get_obs_settings;
use crate::storage::config::{load_config, StreamingPlatform, StreamingStyle};
use crate::commands::utils::get_hardware_info;
use serde::{Deserialize, Serialize};

/// 問題分析リクエスト
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeProblemsRequest {
    /// エンコーダータイプ
    pub encoder_type: String,
    /// 目標ビットレート（kbps）
    pub target_bitrate: u64,
}

/// 問題分析結果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeProblemsResponse {
    /// 検出された問題
    pub problems: Vec<ProblemReport>,
    /// 総合評価スコア（0-100）
    pub overall_score: f64,
}

/// OBS設定分析結果（analyze_settings用）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResult {
    /// 品質スコア（0-100）
    pub quality_score: u8,
    /// 検出された問題の数
    pub issue_count: usize,
    /// 推奨設定変更リスト
    pub recommendations: Vec<ObsSetting>,
    /// システム環境情報
    pub system_info: SystemInfo,
    /// 分析日時（Unixタイムスタンプ）
    pub analyzed_at: i64,
    /// 初心者向けサマリー
    pub summary: AnalysisSummary,
}

/// 分析サマリー（初心者向け）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisSummary {
    /// 初心者向けの一言説明
    pub headline: String,
    /// 推奨プリセット（low/medium/high/ultra）
    pub recommended_preset: String,
    /// 主要な推奨値（キー項目のみ）
    pub key_recommendations: Vec<KeyRecommendation>,
}

/// 主要な推奨項目（初心者向け）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyRecommendation {
    /// 項目ラベル
    pub label: String,
    /// 推奨値
    pub value: String,
    /// 初心者向けの簡潔な理由
    pub reason_simple: String,
}

/// OBS設定項目
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsSetting {
    /// 設定項目キー
    pub key: String,
    /// 表示名
    pub display_name: String,
    /// 現在の値
    pub current_value: serde_json::Value,
    /// 推奨値
    pub recommended_value: serde_json::Value,
    /// 変更理由
    pub reason: String,
    /// 優先度
    pub priority: String, // "critical" | "recommended" | "optional"
}

/// 設定分析リクエスト（オプショナルパラメータ付き）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeSettingsRequest {
    /// 配信プラットフォーム（省略時は設定ファイルから取得）
    pub platform: Option<StreamingPlatform>,
    /// 配信スタイル（省略時は設定ファイルから取得）
    pub style: Option<StreamingStyle>,
    /// ネットワーク速度（Mbps、省略時は設定ファイルから取得）
    pub network_speed_mbps: Option<f64>,
}

/// システム環境情報
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    /// CPUモデル名
    pub cpu_model: String,
    /// GPUモデル名
    pub gpu_model: Option<String>,
    /// 総メモリ容量（MB）
    pub total_memory_mb: u64,
    /// 利用可能メモリ（MB）
    pub available_memory_mb: u64,
}

/// 現在の問題を分析
///
/// システムメトリクスとOBS状態を分析して、パフォーマンス問題を検出する
///
/// # Arguments
/// * `request` - 分析リクエスト（エンコーダータイプ、目標ビットレート）
///
/// # Returns
/// 検出された問題のリスト
#[tauri::command]
pub async fn analyze_problems(request: AnalyzeProblemsRequest) -> Result<AnalyzeProblemsResponse, AppError> {
    let analyzer = ProblemAnalyzer::new();
    let service = system_monitor_service();

    // 現在のシステムメトリクスを取得
    let cpu_usage = service.get_cpu_usage()?;
    let (memory_used, memory_total) = service.get_memory_info()?;
    let gpu_metrics = service.get_gpu_metrics()?;
    let network_metrics = service.get_network_metrics()?;

    // スナップショットを作成
    let current_snapshot = SystemMetricsSnapshot::from_metrics(
        cpu_usage,
        memory_used,
        memory_total,
        gpu_metrics.as_ref(),
        &network_metrics,
    );

    // 履歴データ（現在は単一スナップショット）
    let metrics_history = vec![current_snapshot];

    // ビットレート履歴（ダミーデータ - 将来的には実データを使用）
    let bitrate_history: Vec<u64> = vec![request.target_bitrate];

    // 総合分析を実行
    let problems = analyzer.analyze_comprehensive(
        &metrics_history,
        &bitrate_history,
        request.target_bitrate,
        &request.encoder_type,
    );

    // スコアを計算（問題の数と重要度から）
    let overall_score = calculate_overall_score(&problems);

    Ok(AnalyzeProblemsResponse {
        problems,
        overall_score,
    })
}

/// OBS設定を分析して推奨事項を返す
///
/// # Arguments
/// * `request` - 分析リクエスト（プラットフォーム・スタイルをオーバーライド可能）
///
/// # Returns
/// 分析結果（品質スコア、推奨設定、システム情報）
#[tauri::command]
pub async fn analyze_settings(
    request: Option<AnalyzeSettingsRequest>,
) -> Result<AnalysisResult, AppError> {
    // 現在のOBS設定を取得
    let obs_settings = get_obs_settings().await?;

    // システム情報を取得
    let hardware_info = get_hardware_info().await;

    // アプリケーション設定を取得
    let app_config = load_config()?;

    // リクエストパラメータまたは設定ファイルから値を取得
    let platform = request.as_ref()
        .and_then(|r| r.platform)
        .unwrap_or(app_config.streaming_mode.platform);
    let style = request.as_ref()
        .and_then(|r| r.style)
        .unwrap_or(app_config.streaming_mode.style);
    let network_speed = request.as_ref()
        .and_then(|r| r.network_speed_mbps)
        .unwrap_or(app_config.streaming_mode.network_speed_mbps);

    // 推奨設定を計算
    let recommendations = RecommendationEngine::calculate_recommendations(
        &hardware_info,
        &obs_settings,
        platform,
        style,
        network_speed,
    );

    // 推奨事項リストを構築
    let mut recommendation_list = Vec::new();

    // 解像度の推奨
    if obs_settings.video.output_width != recommendations.video.output_width
        || obs_settings.video.output_height != recommendations.video.output_height {
        recommendation_list.push(ObsSetting {
            key: "video.resolution".to_string(),
            display_name: "出力解像度".to_string(),
            current_value: serde_json::json!(format!(
                "{}x{}",
                obs_settings.video.output_width,
                obs_settings.video.output_height
            )),
            recommended_value: serde_json::json!(format!(
                "{}x{}",
                recommendations.video.output_width,
                recommendations.video.output_height
            )),
            reason: "現在の設定はシステム性能に最適化されていません".to_string(),
            priority: "recommended".to_string(),
        });
    }

    // FPSの推奨
    let current_fps = obs_settings.video.fps() as u32;
    if current_fps != recommendations.video.fps {
        recommendation_list.push(ObsSetting {
            key: "video.fps".to_string(),
            display_name: "FPS".to_string(),
            current_value: serde_json::json!(current_fps),
            recommended_value: serde_json::json!(recommendations.video.fps),
            reason: "配信スタイルに適したFPSに変更することを推奨します".to_string(),
            priority: if current_fps > recommendations.video.fps { "recommended" } else { "optional" }.to_string(),
        });
    }

    // ビットレートの推奨
    let bitrate_diff = (obs_settings.output.bitrate_kbps as i32
        - recommendations.output.bitrate_kbps as i32).abs();
    if bitrate_diff > 500 {
        recommendation_list.push(ObsSetting {
            key: "output.bitrate".to_string(),
            display_name: "ビットレート".to_string(),
            current_value: serde_json::json!(obs_settings.output.bitrate_kbps),
            recommended_value: serde_json::json!(recommendations.output.bitrate_kbps),
            reason: format!(
                "ネットワーク速度とプラットフォームに最適化されたビットレートは{}kbpsです",
                recommendations.output.bitrate_kbps
            ),
            priority: if bitrate_diff > 2000 { "critical" } else { "recommended" }.to_string(),
        });
    }

    // エンコーダーの推奨
    if obs_settings.output.encoder != recommendations.output.encoder {
        let priority = if !obs_settings.output.is_hardware_encoder() && hardware_info.gpu.is_some() {
            "critical"
        } else {
            "recommended"
        };

        recommendation_list.push(ObsSetting {
            key: "output.encoder".to_string(),
            display_name: "エンコーダー".to_string(),
            current_value: serde_json::json!(obs_settings.output.encoder),
            recommended_value: serde_json::json!(recommendations.output.encoder),
            reason: "ハードウェアエンコーダーの使用を推奨します（CPU負荷軽減のため）".to_string(),
            priority: priority.to_string(),
        });
    }

    // システム情報を構築
    let (memory_used, memory_total) = get_memory_info().unwrap_or((0, 8_000_000_000));
    let system_info = SystemInfo {
        cpu_model: hardware_info.cpu_name.clone(),
        gpu_model: hardware_info.gpu.as_ref().map(|g| g.name.clone()),
        total_memory_mb: memory_total / 1_048_576,
        available_memory_mb: (memory_total - memory_used) / 1_048_576,
    };

    // 品質スコアを取得
    let quality_score = recommendations.overall_score;

    // 初心者向けサマリーを生成
    let summary = generate_analysis_summary(
        &hardware_info,
        &recommendations,
        quality_score,
    );

    Ok(AnalysisResult {
        quality_score,
        issue_count: recommendation_list.len(),
        recommendations: recommendation_list,
        system_info,
        analyzed_at: chrono::Utc::now().timestamp(),
        summary,
    })
}

/// 問題履歴を取得
///
/// 過去に検出された問題の履歴を取得する
///
/// # Arguments
/// * `limit` - 取得する問題の最大数
///
/// # Returns
/// 問題履歴のリスト
#[tauri::command]
pub async fn get_problem_history(limit: usize) -> Result<Vec<ProblemReport>, AppError> {
    // TODO: 実際の履歴データベースから取得
    // 現在は空のリストを返す
    let _ = limit; // 未使用警告を回避
    Ok(Vec::new())
}

/// スコアを計算
///
/// 問題の数と重要度から総合スコアを算出
fn calculate_overall_score(problems: &[ProblemReport]) -> f64 {
    if problems.is_empty() {
        return 100.0;
    }

    let mut score: f64 = 100.0;

    for problem in problems {
        let penalty = match problem.severity {
            crate::services::alerts::AlertSeverity::Critical => 20.0,
            crate::services::alerts::AlertSeverity::Warning => 10.0,
            crate::services::alerts::AlertSeverity::Info => 5.0,
            crate::services::alerts::AlertSeverity::Tips => 2.0,
        };
        score -= penalty;
    }

    score.clamp(0.0, 100.0)
}

/// エンコーダーIDからユーザー向け表示ラベルを取得
///
/// OBSで使用される様々なエンコーダーIDを判定して、
/// ユーザーフレンドリーな表示名に変換する。
///
/// # Arguments
/// * `encoder_id` - OBSエンコーダーID（例: jim_av1_nvenc, ffmpeg_nvenc, obs_x264）
///
/// # Returns
/// ユーザー向け表示ラベル（例: "NVIDIA NVENC (AV1)"）
///
/// # サポートされるエンコーダー
/// - NVIDIA: jim_av1_nvenc, ffmpeg_nvenc, jim_hevc_nvenc等
/// - AMD: amd_amf_h264, h264_texture_amf, av1_amf等
/// - Intel: obs_qsv11, obs_qsv11_av1, qsv_hevc等
/// - CPU: obs_x264, obs_x265等
fn get_encoder_display_label(encoder_id: &str) -> String {
    // 空文字列チェック
    if encoder_id.is_empty() {
        return "不明なエンコーダー".to_string();
    }

    // 大文字小文字を統一（判定用）
    let encoder_lower = encoder_id.to_lowercase();

    // === AV1エンコーダー ===
    if encoder_lower.contains("av1") {
        // NVIDIA AV1（RTX 40/50シリーズ）
        if encoder_lower.contains("nvenc") || encoder_lower.contains("jim_av1") {
            return "NVIDIA NVENC (AV1)".to_string();
        }
        // AMD AV1（RX 7000シリーズ以降）
        if encoder_lower.contains("amf") || encoder_lower.contains("amd") {
            return "AMD AMF (AV1)".to_string();
        }
        // Intel AV1（Arc/Meteor Lake以降）
        if encoder_lower.contains("qsv") {
            return "Intel QSV (AV1)".to_string();
        }
        // その他のAV1（ソフトウェアエンコード等）
        return "AV1".to_string();
    }

    // === HEVCエンコーダー ===
    if encoder_lower.contains("hevc") || encoder_lower.contains("h265") {
        // NVIDIA HEVC
        if encoder_lower.contains("nvenc") {
            return "NVIDIA NVENC (HEVC)".to_string();
        }
        // AMD HEVC
        if encoder_lower.contains("amf") || encoder_lower.contains("amd") {
            return "AMD AMF (HEVC)".to_string();
        }
        // Intel HEVC
        if encoder_lower.contains("qsv") {
            return "Intel QSV (HEVC)".to_string();
        }
        // CPU x265（ソフトウェアHEVCエンコード）
        if encoder_lower.contains("x265") {
            return "CPU (x265)".to_string();
        }
        return "HEVC".to_string();
    }

    // === H.264エンコーダー ===

    // NVIDIA H.264（すべてのNVENC対応GPU）
    if encoder_lower.contains("nvenc") {
        return "NVIDIA NVENC (H.264)".to_string();
    }

    // AMD H.264
    if encoder_lower.contains("amd") || encoder_lower.contains("amf") {
        return "AMD AMF (H.264)".to_string();
    }

    // Intel QuickSync H.264
    if encoder_lower.contains("qsv") {
        return "Intel QSV (H.264)".to_string();
    }

    // CPU x264（ソフトウェアH.264エンコード）
    if encoder_lower.contains("x264") {
        return "CPU (x264)".to_string();
    }

    // CPU x265（既にHEVCセクションでチェック済みだが念のため）
    if encoder_lower.contains("x265") {
        return "CPU (x265)".to_string();
    }

    // === 不明なエンコーダー ===
    "不明なエンコーダー".to_string()
}

/// 初心者向け分析サマリーを生成
///
/// # Arguments
/// * `hardware` - ハードウェア情報
/// * `recommendations` - 推奨設定
/// * `quality_score` - 品質スコア（0-100）
///
/// # Returns
/// 初心者向けのわかりやすいサマリー
fn generate_analysis_summary(
    hardware: &crate::services::optimizer::HardwareInfo,
    recommendations: &crate::services::optimizer::RecommendedSettings,
    _quality_score: u8,
) -> AnalysisSummary {
    // GPU名を取得（わかりやすく短縮）
    let gpu_name = hardware.gpu.as_ref()
        .map(|g| {
            // NVIDIA GeForce RTX 3060 -> RTX 3060
            let name = &g.name;
            if name.contains("RTX") {
                name.split("RTX").nth(1)
                    .map(|s| format!("RTX{}", s.trim()))
                    .unwrap_or_else(|| name.clone())
            } else if name.contains("GTX") {
                name.split("GTX").nth(1)
                    .map(|s| format!("GTX{}", s.trim()))
                    .unwrap_or_else(|| name.clone())
            } else if name.contains("AMD") || name.contains("Radeon") {
                name.replace("AMD ", "").replace("Radeon ", "")
            } else {
                name.clone()
            }
        })
        .unwrap_or_else(|| "統合GPU".to_string());

    // 推奨プリセットを決定（low/medium/high/ultra）
    let recommended_preset = if hardware.cpu_cores < 4 || hardware.gpu.is_none() {
        "low"
    } else if hardware.cpu_cores < 8 {
        "medium"
    } else if hardware.gpu.is_some() && hardware.cpu_cores >= 8 {
        "high"
    } else {
        "ultra"
    };

    // ヘッドラインを生成
    let headline = format!(
        "あなたのPC（{}）なら、{}p {}fpsで快適に配信できます",
        gpu_name,
        if recommendations.video.output_height >= 1080 { "1080" } else { "720" },
        recommendations.video.fps
    );

    // 主要な推奨項目を抽出
    let mut key_recommendations = Vec::new();

    // 解像度
    key_recommendations.push(KeyRecommendation {
        label: "解像度".to_string(),
        value: format!("{}x{}", recommendations.video.output_width, recommendations.video.output_height),
        reason_simple: if recommendations.video.output_height >= 1080 {
            "お使いのGPUで高画質配信が可能です".to_string()
        } else {
            "安定した配信のため720pを推奨".to_string()
        },
    });

    // FPS
    key_recommendations.push(KeyRecommendation {
        label: "FPS".to_string(),
        value: format!("{}fps", recommendations.video.fps),
        reason_simple: if recommendations.video.fps >= 60 {
            "滑らかな映像で視聴者に快適な体験を".to_string()
        } else {
            "動きの少ない配信なら30fpsで十分".to_string()
        },
    });

    // ビットレート
    key_recommendations.push(KeyRecommendation {
        label: "ビットレート".to_string(),
        value: format!("{}kbps", recommendations.output.bitrate_kbps),
        reason_simple: "ネットワーク速度に最適化".to_string(),
    });

    // エンコーダー
    let encoder_label = get_encoder_display_label(&recommendations.output.encoder);

    key_recommendations.push(KeyRecommendation {
        label: "エンコーダー".to_string(),
        value: encoder_label.clone(),
        reason_simple: if encoder_label.contains("CPU") {
            "CPU負荷が高めです。GPU搭載PCの場合はハードウェアエンコーダー推奨".to_string()
        } else if encoder_label.contains("AV1") {
            "最新AV1コーデック（※OBS 30.0以上 + Enhanced RTMP対応が必要）".to_string()
        } else {
            "GPU使用でCPU負荷を軽減".to_string()
        },
    });

    AnalysisSummary {
        headline,
        recommended_preset: recommended_preset.to_string(),
        key_recommendations,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_overall_score_no_problems() {
        let problems = vec![];
        let score = calculate_overall_score(&problems);
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_calculate_overall_score_with_problems() {
        use crate::services::alerts::{AlertSeverity, MetricType};
        use crate::services::analyzer::ProblemCategory;

        let problems = vec![
            ProblemReport {
                id: "test-1".to_string(),
                category: ProblemCategory::Resource,
                severity: AlertSeverity::Critical,
                title: "Test".to_string(),
                description: "Test".to_string(),
                suggested_actions: vec![],
                affected_metric: MetricType::CpuUsage,
                detected_at: 0,
            },
            ProblemReport {
                id: "test-2".to_string(),
                category: ProblemCategory::Network,
                severity: AlertSeverity::Warning,
                title: "Test".to_string(),
                description: "Test".to_string(),
                suggested_actions: vec![],
                affected_metric: MetricType::NetworkBandwidth,
                detected_at: 0,
            },
        ];

        let score = calculate_overall_score(&problems);
        assert_eq!(score, 70.0); // 100 - 20 - 10
    }

    // === エンコーダー表示ラベルのテスト ===

    #[test]
    fn test_get_encoder_label_nvidia_av1() {
        // NVIDIA AV1エンコーダー（RTX 40/50シリーズ）
        let test_cases = vec![
            ("jim_av1_nvenc", "NVIDIA NVENC (AV1)"),
            ("av1_nvenc", "NVIDIA NVENC (AV1)"),
        ];
        for (encoder_id, expected) in test_cases {
            let label = get_encoder_display_label(encoder_id);
            assert_eq!(label, expected, "Encoder ID: {}", encoder_id);
        }
    }

    #[test]
    fn test_get_encoder_label_nvidia_hevc() {
        // NVIDIA HEVCエンコーダー（GTX 10/RTX 20以降）
        let test_cases = vec![
            ("jim_hevc_nvenc", "NVIDIA NVENC (HEVC)"),
            ("ffmpeg_hevc_nvenc", "NVIDIA NVENC (HEVC)"),
            ("nvenc_hevc", "NVIDIA NVENC (HEVC)"),
        ];
        for (encoder_id, expected) in test_cases {
            let label = get_encoder_display_label(encoder_id);
            assert_eq!(label, expected, "Encoder ID: {}", encoder_id);
        }
    }

    #[test]
    fn test_get_encoder_label_nvidia_h264() {
        // NVIDIA H.264エンコーダー（すべてのNVENC対応GPU）
        let test_cases = vec![
            ("ffmpeg_nvenc", "NVIDIA NVENC (H.264)"),
            ("jim_nvenc", "NVIDIA NVENC (H.264)"),
            ("nvenc_h264", "NVIDIA NVENC (H.264)"),
        ];
        for (encoder_id, expected) in test_cases {
            let label = get_encoder_display_label(encoder_id);
            assert_eq!(label, expected, "Encoder ID: {}", encoder_id);
        }
    }

    #[test]
    fn test_get_encoder_label_amd_av1() {
        // AMD AV1エンコーダー（RX 7000シリーズ以降）
        let test_cases = vec![
            ("av1_amf", "AMD AMF (AV1)"),
            ("amd_av1_amf", "AMD AMF (AV1)"),
        ];
        for (encoder_id, expected) in test_cases {
            let label = get_encoder_display_label(encoder_id);
            assert_eq!(label, expected, "Encoder ID: {}", encoder_id);
        }
    }

    #[test]
    fn test_get_encoder_label_amd_hevc() {
        // AMD HEVCエンコーダー（RX 400シリーズ以降）
        let test_cases = vec![
            ("hevc_amf", "AMD AMF (HEVC)"),
            ("amd_hevc_amf", "AMD AMF (HEVC)"),
        ];
        for (encoder_id, expected) in test_cases {
            let label = get_encoder_display_label(encoder_id);
            assert_eq!(label, expected, "Encoder ID: {}", encoder_id);
        }
    }

    #[test]
    fn test_get_encoder_label_amd_h264() {
        // AMD H.264エンコーダー
        let test_cases = vec![
            ("amd_amf_h264", "AMD AMF (H.264)"),
            ("h264_texture_amf", "AMD AMF (H.264)"),
            ("h264_amf", "AMD AMF (H.264)"),
        ];
        for (encoder_id, expected) in test_cases {
            let label = get_encoder_display_label(encoder_id);
            assert_eq!(label, expected, "Encoder ID: {}", encoder_id);
        }
    }

    #[test]
    fn test_get_encoder_label_intel_av1() {
        // Intel AV1エンコーダー（Arc/Meteor Lake以降）
        let test_cases = vec![
            ("obs_qsv11_av1", "Intel QSV (AV1)"),
            ("qsv_av1", "Intel QSV (AV1)"),
        ];
        for (encoder_id, expected) in test_cases {
            let label = get_encoder_display_label(encoder_id);
            assert_eq!(label, expected, "Encoder ID: {}", encoder_id);
        }
    }

    #[test]
    fn test_get_encoder_label_intel_hevc() {
        // Intel HEVCエンコーダー（Skylake以降）
        let test_cases = vec![
            ("obs_qsv11_hevc", "Intel QSV (HEVC)"),
            ("qsv_hevc", "Intel QSV (HEVC)"),
        ];
        for (encoder_id, expected) in test_cases {
            let label = get_encoder_display_label(encoder_id);
            assert_eq!(label, expected, "Encoder ID: {}", encoder_id);
        }
    }

    #[test]
    fn test_get_encoder_label_intel_h264() {
        // Intel H.264エンコーダー（QuickSync）
        let test_cases = vec![
            ("obs_qsv11", "Intel QSV (H.264)"),
            ("qsv_h264", "Intel QSV (H.264)"),
        ];
        for (encoder_id, expected) in test_cases {
            let label = get_encoder_display_label(encoder_id);
            assert_eq!(label, expected, "Encoder ID: {}", encoder_id);
        }
    }

    #[test]
    fn test_get_encoder_label_cpu_x264() {
        // CPU x264エンコーダー
        let test_cases = vec![
            ("obs_x264", "CPU (x264)"),
            ("x264", "CPU (x264)"),
        ];
        for (encoder_id, expected) in test_cases {
            let label = get_encoder_display_label(encoder_id);
            assert_eq!(label, expected, "Encoder ID: {}", encoder_id);
        }
    }

    #[test]
    fn test_get_encoder_label_cpu_x265() {
        // CPU x265エンコーダー（HEVCソフトウェアエンコード）
        let test_cases = vec![
            ("obs_x265", "CPU (x265)"),
            ("x265", "CPU (x265)"),
        ];
        for (encoder_id, expected) in test_cases {
            let label = get_encoder_display_label(encoder_id);
            assert_eq!(label, expected, "Encoder ID: {}", encoder_id);
        }
    }

    #[test]
    fn test_get_encoder_label_edge_cases() {
        // エッジケース: 空文字列、不明なエンコーダー
        let test_cases = vec![
            ("", "不明なエンコーダー"),
            ("unknown_encoder", "不明なエンコーダー"),
            ("invalid", "不明なエンコーダー"),
        ];
        for (encoder_id, expected) in test_cases {
            let label = get_encoder_display_label(encoder_id);
            assert_eq!(label, expected, "Encoder ID: {}", encoder_id);
        }
    }

    #[test]
    fn test_get_encoder_label_case_insensitive() {
        // 大文字小文字の違いを許容
        let test_cases = vec![
            ("FFMPEG_NVENC", "NVIDIA NVENC (H.264)"),
            ("Jim_AV1_NVENC", "NVIDIA NVENC (AV1)"),
            ("AMD_AMF_H264", "AMD AMF (H.264)"),
            ("OBS_QSV11_AV1", "Intel QSV (AV1)"),
            ("OBS_X264", "CPU (x264)"),
        ];
        for (encoder_id, expected) in test_cases {
            let label = get_encoder_display_label(encoder_id);
            assert_eq!(label, expected, "Encoder ID: {}", encoder_id);
        }
    }

    #[test]
    fn test_get_encoder_label_consistency_with_encoder_selector() {
        // encoder_selector.rsと整合性を確認
        // encoder_selector.rsが返すIDが正しく表示されることを保証
        let selector_encoder_ids = vec![
            "jim_av1_nvenc",      // NVIDIA AV1
            "ffmpeg_nvenc",       // NVIDIA H.264
            "amd_amf_h264",       // AMD H.264
            "obs_qsv11_av1",      // Intel Arc AV1
            "obs_qsv11",          // Intel QuickSync H.264
            "obs_x264",           // CPU x264
        ];

        for encoder_id in selector_encoder_ids {
            let label = get_encoder_display_label(encoder_id);
            // ラベルが空文字列や"不明"でないことを確認
            assert!(!label.is_empty(), "Label should not be empty for: {}", encoder_id);
            assert!(!label.contains("不明"), "Label should be known for encoder_selector ID: {}", encoder_id);
        }
    }

    // === 配信特化UI改善のテスト ===

    #[test]
    fn test_fps_label_in_key_recommendations() {
        // FPSラベルが"FPS"になっていることを確認
        use crate::services::optimizer::{HardwareInfo, RecommendedSettings, GpuInfo, VideoSettings, OutputSettings};

        let hardware = HardwareInfo {
            cpu_name: "Test CPU".to_string(),
            cpu_cores: 8,
            gpu: Some(GpuInfo {
                name: "NVIDIA GeForce RTX 3060".to_string(),
                vendor: "NVIDIA".to_string(),
                tier: 0,
                generation: 3,
            }),
            total_memory_bytes: 16_000_000_000,
        };

        let recommendations = RecommendedSettings {
            video: VideoSettings {
                base_width: 1920,
                base_height: 1080,
                output_width: 1920,
                output_height: 1080,
                fps: 60,
            },
            output: OutputSettings {
                encoder: "jim_av1_nvenc".to_string(),
                bitrate_kbps: 6000,
                preset: "p5".to_string(),
                rate_control: "CBR".to_string(),
            },
            overall_score: 85,
        };

        let summary = generate_analysis_summary(&hardware, &recommendations, 85);

        // FPS項目のラベルをチェック
        let fps_recommendation = summary.key_recommendations.iter()
            .find(|r| r.label == "FPS")
            .expect("FPS recommendation should exist");

        assert_eq!(fps_recommendation.label, "FPS", "FPS label should be 'FPS', not 'フレームレート'");
    }

    #[test]
    fn test_av1_encoder_message_contains_obs_version_warning() {
        // AV1エンコーダーの説明にOBS 30.0要件が含まれることを確認
        use crate::services::optimizer::{HardwareInfo, RecommendedSettings, GpuInfo, VideoSettings, OutputSettings};

        let hardware = HardwareInfo {
            cpu_name: "Test CPU".to_string(),
            cpu_cores: 8,
            gpu: Some(GpuInfo {
                name: "NVIDIA GeForce RTX 4060".to_string(),
                vendor: "NVIDIA".to_string(),
                tier: 0,
                generation: 4,
            }),
            total_memory_bytes: 16_000_000_000,
        };

        let recommendations = RecommendedSettings {
            video: VideoSettings {
                base_width: 1920,
                base_height: 1080,
                output_width: 1920,
                output_height: 1080,
                fps: 60,
            },
            output: OutputSettings {
                encoder: "jim_av1_nvenc".to_string(), // AV1エンコーダー
                bitrate_kbps: 6000,
                preset: "p5".to_string(),
                rate_control: "CBR".to_string(),
            },
            overall_score: 90,
        };

        let summary = generate_analysis_summary(&hardware, &recommendations, 90);

        // エンコーダー項目の説明をチェック
        let encoder_recommendation = summary.key_recommendations.iter()
            .find(|r| r.label == "エンコーダー")
            .expect("Encoder recommendation should exist");

        assert!(
            encoder_recommendation.reason_simple.contains("AV1"),
            "AV1 encoder message should mention AV1"
        );
        assert!(
            encoder_recommendation.reason_simple.contains("OBS 30.0"),
            "AV1 encoder message should warn about OBS 30.0 requirement"
        );
        assert!(
            encoder_recommendation.reason_simple.contains("Enhanced RTMP"),
            "AV1 encoder message should warn about Enhanced RTMP requirement"
        );
    }
}
