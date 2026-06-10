//! 文献对比 Tauri 命令接口
//!
//! 提供文献对比功能的前端调用接口

use tauri::State;
use crate::ai::comparison::{
    ArticleComparison, ComparisonGenerator,
};
use crate::ai::commands::AIState;

/// Tauri 命令：对比多篇文献
#[tauri::command]
pub async fn compare_articles(
    state: State<'_, AIState>,
    item_ids: Vec<i32>,
) -> Result<ArticleComparison, String> {
    eprintln!("[命令] compare_articles 被调用: item_ids={:?}", item_ids);

    let config = state.config_manager.get_config();

    if config.api_key.is_empty() {
        return Err("API 密钥未配置，请在设置中配置 AI".to_string());
    }

    if !config.enabled {
        return Err("AI 功能已禁用，请在设置中启用".to_string());
    }

    if item_ids.len() < 2 {
        return Err("对比需要至少2篇文献".to_string());
    }

    if item_ids.len() > 5 {
        return Err("对比最多支持5篇文献".to_string());
    }

    let provider = crate::ai::create_provider(&config).map_err(|e| e.to_string())?;
    let generator = ComparisonGenerator::new(provider);

    generator
        .generate_comparison(item_ids)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri 命令：获取对比结果（从缓存）
#[tauri::command]
pub fn get_comparison_result(
    state: State<AIState>,
    item_ids: Vec<i32>,
) -> Option<ArticleComparison> {
    let config = state.config_manager.get_config();
    let provider = match crate::ai::create_provider(&config) {
        Ok(p) => p,
        Err(_) => return None,
    };
    let generator = ComparisonGenerator::new(provider);
    generator.get_cached_comparison(&item_ids)
}

/// Tauri 命令：检查是否有缓存的对比结果
#[tauri::command]
pub fn has_comparison_result(
    state: State<AIState>,
    item_ids: Vec<i32>,
) -> bool {
    let config = state.config_manager.get_config();
    let provider = match crate::ai::create_provider(&config) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let generator = ComparisonGenerator::new(provider);
    generator.has_cached_comparison(&item_ids)
}

/// 导出格式枚举
#[derive(Debug, Clone, serde::Deserialize)]
pub enum ExportFormat {
    Markdown,
    Csv,
}

/// Tauri 命令：导出对比结果
#[tauri::command]
pub fn export_comparison(
    comparison: ArticleComparison,
    format: String,
) -> Result<String, String> {
    eprintln!(
        "[命令] export_comparison 被调用: comparison_id={}, format={}",
        comparison.comparison_id, format
    );

    match format.as_str() {
        "markdown" | "md" => Ok(comparison.to_markdown()),
        "csv" | "excel" => Ok(comparison.to_csv()),
        _ => Err(format!("不支持的导出格式: {}，支持 markdown 和 csv", format)),
    }
}

/// Tauri 命令：获取对比的 Markdown 格式（便捷方法）
#[tauri::command]
pub fn get_comparison_as_markdown(
    state: State<AIState>,
    item_ids: Vec<i32>,
) -> Result<String, String> {
    let config = state.config_manager.get_config();
    let provider = crate::ai::create_provider(&config).map_err(|e| e.to_string())?;
    let generator = ComparisonGenerator::new(provider);

    generator
        .get_cached_comparison(&item_ids)
        .map(|c| c.to_markdown())
        .ok_or_else(|| "没有缓存的对比结果".to_string())
}

/// Tauri 命令：获取对比的 CSV 格式（便捷方法）
#[tauri::command]
pub fn get_comparison_as_csv(
    state: State<AIState>,
    item_ids: Vec<i32>,
) -> Result<String, String> {
    let config = state.config_manager.get_config();
    let provider = crate::ai::create_provider(&config).map_err(|e| e.to_string())?;
    let generator = ComparisonGenerator::new(provider);

    generator
        .get_cached_comparison(&item_ids)
        .map(|c| c.to_csv())
        .ok_or_else(|| "没有缓存的对比结果".to_string())
}