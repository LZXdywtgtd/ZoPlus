//! AI Tauri 命令接口
//!
//! 提供 AI 功能的前端调用接口

use std::sync::Arc;
use tauri::State;
use rusqlite::params;

use crate::ai::traits::AIProvider;
use crate::ai::{
    AIConfig, AIConfigManager, AIProviderType, ModelInfo, ModelPrice, Message,
    create_provider,
};

/// AI 配置状态管理
pub struct AIState {
    pub config_manager: AIConfigManager,
}

impl AIState {
    pub fn new() -> Self {
        Self {
            config_manager: AIConfigManager::new(),
        }
    }
}

impl Default for AIState {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri 命令：获取 AI 配置
#[tauri::command]
pub fn get_ai_config(state: State<AIState>) -> AIConfig {
    state.config_manager.get_config()
}

/// Tauri 命令：更新 AI 配置
#[tauri::command]
pub fn update_ai_config(state: State<AIState>, config: AIConfig) {
    state.config_manager.update_config(config);
}

/// Tauri 命令：更新 API 密钥
#[tauri::command]
pub fn update_ai_api_key(state: State<AIState>, api_key: String) {
    state.config_manager.update_api_key(api_key);
}

/// Tauri 命令：更新选中的模型
#[tauri::command]
pub fn update_ai_model(state: State<AIState>, model_id: String) {
    state.config_manager.update_model(model_id);
}

/// Tauri 命令：更新 AI 厂商
#[tauri::command]
pub fn update_ai_provider(state: State<AIState>, provider: AIProviderType) {
    state.config_manager.update_provider(provider);
}

/// Tauri 命令：启用/禁用 AI 功能
#[tauri::command]
pub fn set_ai_enabled(state: State<AIState>, enabled: bool) {
    state.config_manager.set_enabled(enabled);
}

/// Tauri 命令：检查 AI 是否已配置
#[tauri::command]
pub fn is_ai_configured(state: State<AIState>) -> bool {
    state.config_manager.is_configured()
}

/// Tauri 命令：发送对话补全请求
#[tauri::command]
pub async fn chat_completion(
    state: State<'_, AIState>,
    messages: Vec<Message>,
) -> Result<String, String> {
    let config = state.config_manager.get_config();

    if config.api_key.is_empty() {
        return Err("API 密钥未配置".to_string());
    }

    let provider = create_provider(&config).map_err(|e| e.to_string())?;
    provider.chat_completion(messages).map_err(|e| e.to_string())
}

/// Tauri 命令：测试 AI 连接
#[tauri::command]
pub async fn test_ai_connection(state: State<'_, AIState>) -> Result<bool, String> {
    let config = state.config_manager.get_config();

    if config.api_key.is_empty() {
        return Err("API 密钥未配置".to_string());
    }

    let provider = create_provider(&config).map_err(|e| e.to_string())?;
    provider.test_connection().map_err(|e| e.to_string())
}

/// Tauri 命令：获取所有可用模型
#[tauri::command]
pub fn get_all_ai_models() -> Vec<ModelInfo> {
    crate::ai::providers::get_all_models()
}

/// Tauri 命令：获取指定厂商的可用模型
#[tauri::command]
pub fn get_ai_models_by_provider(provider: AIProviderType) -> Vec<ModelInfo> {
    use crate::ai::providers;
    let dummy_key = "dummy".to_string();
    match provider {
        AIProviderType::OpenAI => {
            if let Ok(p) = providers::openai::OpenAIProvider::new(dummy_key.clone(), None) {
                p.get_available_models()
            } else {
                vec![]
            }
        }
        AIProviderType::Anthropic => {
            if let Ok(p) = providers::anthropic::AnthropicProvider::new(dummy_key.clone(), None) {
                p.get_available_models()
            } else {
                vec![]
            }
        }
        AIProviderType::DeepSeek => {
            if let Ok(p) = providers::deepseek::DeepSeekProvider::new(dummy_key.clone(), None) {
                p.get_available_models()
            } else {
                vec![]
            }
        }
        AIProviderType::Doubao => {
            if let Ok(p) = providers::doubao::DoubaoProvider::new(dummy_key.clone(), None) {
                p.get_available_models()
            } else {
                vec![]
            }
        }
        AIProviderType::Qwen => {
            if let Ok(p) = providers::qwen::QwenProvider::new(dummy_key.clone(), None) {
                p.get_available_models()
            } else {
                vec![]
            }
        }
        AIProviderType::Glm => {
            if let Ok(p) = providers::glm::GlmProvider::new(dummy_key.clone(), None) {
                p.get_available_models()
            } else {
                vec![]
            }
        }
        AIProviderType::MiniMax => {
            if let Ok(p) = providers::minimax::MiniMaxProvider::new(dummy_key.clone(), None) {
                p.get_available_models()
            } else {
                vec![]
            }
        }
        AIProviderType::MiMo => {
            if let Ok(p) = providers::mimo::MiMoProvider::new(dummy_key.clone(), None) {
                p.get_available_models()
            } else {
                vec![]
            }
        }
    }
}

/// Tauri 命令：获取模型价格
#[tauri::command]
pub fn get_model_price(state: State<AIState>, model_id: String) -> Option<ModelPrice> {
    let config = state.config_manager.get_config();
    let provider = create_provider(&config).ok()?;
    provider.get_model_price(&model_id)
}

// ============== 文献摘要相关命令 ==============

use crate::ai::summary::{ArticleSummary, SummaryGenerator};

/// Tauri 命令：生成文献摘要
#[tauri::command]
pub async fn get_article_summary(
    state: State<'_, AIState>,
    item_id: i32,
    pdf_key: Option<String>,
) -> Result<ArticleSummary, String> {
    eprintln!("[命令] get_article_summary 被调用: item_id={}, pdf_key={:?}", item_id, pdf_key);

    let config = state.config_manager.get_config();

    if config.api_key.is_empty() {
        return Err("API 密钥未配置，请在设置中配置 AI".to_string());
    }

    if !config.enabled {
        return Err("AI 功能已禁用，请在设置中启用".to_string());
    }

    let provider = create_provider(&config).map_err(|e| e.to_string())?;
    let generator = SummaryGenerator::new(provider);

    generator
        .generate_summary(item_id, pdf_key.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Tauri 命令：检查是否有缓存的摘要
#[tauri::command]
pub fn has_cached_summary(state: State<AIState>, item_id: i32) -> bool {
    let config = state.config_manager.get_config();
    let provider = match create_provider(&config) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let generator = SummaryGenerator::new(provider);
    generator.has_cached_summary(item_id)
}

/// Tauri 命令：获取缓存的摘要
#[tauri::command]
pub fn get_cached_summary(state: State<AIState>, item_id: i32) -> Option<ArticleSummary> {
    let config = state.config_manager.get_config();
    let provider = match create_provider(&config) {
        Ok(p) => p,
        Err(_) => return None,
    };
    let generator = SummaryGenerator::new(provider);
    generator.get_cached_summary(item_id)
}

/// Tauri 命令：导出摘要为 Markdown
#[tauri::command]
pub fn export_summary_as_markdown(state: State<AIState>, item_id: i32) -> Result<String, String> {
    let config = state.config_manager.get_config();
    let provider = create_provider(&config).map_err(|e| e.to_string())?;
    let generator = SummaryGenerator::new(provider);

    generator
        .get_cached_summary(item_id)
        .map(|s| s.to_markdown())
        .ok_or_else(|| "没有缓存的摘要".to_string())
}

// ============== 智能笔记相关命令 ==============

use crate::ai::note::{Note, NoteTemplate, NoteGenerator as NoteGen};

/// Tauri 命令：生成单条笔记
#[tauri::command]
pub async fn generate_note(
    state: State<'_, AIState>,
    item_id: i32,
    source_text: Option<String>,
    page: Option<u32>,
    template: String,
) -> Result<Note, String> {
    eprintln!("[命令] generate_note 被调用: item_id={}, template={}", item_id, template);

    let config = state.config_manager.get_config();

    if config.api_key.is_empty() {
        return Err("API 密钥未配置，请在设置中配置 AI".to_string());
    }

    if !config.enabled {
        return Err("AI 功能已禁用，请在设置中启用".to_string());
    }

    // 解析模板类型
    let template_type = match template.as_str() {
        "key_points" => NoteTemplate::KeyPoints,
        "methods" => NoteTemplate::Methods,
        "conclusions" => NoteTemplate::Conclusions,
        "critical" => NoteTemplate::Critical,
        _ => NoteTemplate::General,
    };

    let provider = create_provider(&config).map_err(|e| e.to_string())?;
    let generator = NoteGen::new(provider);

    generator
        .generate_note(item_id, source_text, page, template_type)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri 命令：批量生成笔记（基于多个高亮）
#[tauri::command]
pub async fn generate_notes_batch(
    state: State<'_, AIState>,
    item_id: i32,
    pdf_key: String,
    template: String,
) -> Result<Vec<Note>, String> {
    eprintln!("[命令] generate_notes_batch 被调用: item_id={}, pdf_key={}", item_id, pdf_key);

    let config = state.config_manager.get_config();

    if config.api_key.is_empty() {
        return Err("API 密钥未配置，请在设置中配置 AI".to_string());
    }

    if !config.enabled {
        return Err("AI 功能已禁用，请在设置中启用".to_string());
    }

    // 解析模板类型
    let template_type = match template.as_str() {
        "key_points" => NoteTemplate::KeyPoints,
        "methods" => NoteTemplate::Methods,
        "conclusions" => NoteTemplate::Conclusions,
        "critical" => NoteTemplate::Critical,
        _ => NoteTemplate::General,
    };

    let provider = create_provider(&config).map_err(|e| e.to_string())?;
    let generator = NoteGen::new(provider);

    generator
        .generate_notes_batch(item_id, &pdf_key, template_type)
        .await
        .map_err(|e| e.to_string())
}

/// Tauri 命令：保存笔记到 Zotero itemNotes 表
#[tauri::command]
pub fn save_note_to_item(
    item_id: i32,
    note: Note,
) -> Result<bool, String> {
    eprintln!("[命令] save_note_to_item 被调用: item_id={}", item_id);

    let guard = crate::db::connection::get_connection()
        .map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let conn = guard.as_ref().ok_or_else(|| "数据库连接未初始化".to_string())?;

    // 将笔记序列化为 JSON
    let note_json = note.to_json().map_err(|e| format!("序列化笔记失败: {}", e))?;

    // 构建 itemNotes 表的 note 内容（Zotero 原生格式 + ZoPlus 元数据）
    let note_content = format!(
        "{{\"zoplus_note\":true,\"version\":1}}\n{}",
        note_json
    );

    // 插入到 itemNotes 表
    let sql = "INSERT INTO itemNotes (itemID, note, clientDate, source) VALUES (?, ?, datetime('now'), 'ZoPlus')";
    conn.execute(sql, params![item_id, note_content])
        .map_err(|e| format!("保存笔记失败: {}", e))?;

    eprintln!("[命令] 笔记已保存到 itemNotes: item_id={}", item_id);
    Ok(true)
}

/// Tauri 命令：获取指定文献的所有笔记
#[tauri::command]
pub fn get_notes_for_item(item_id: i32) -> Result<Vec<Note>, String> {
    eprintln!("[命令] get_notes_for_item 被调用: item_id={}", item_id);

    let guard = crate::db::connection::get_connection()
        .map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let conn = guard.as_ref().ok_or_else(|| "数据库连接未初始化".to_string())?;

    // 从 itemNotes 表查询笔记
    let sql = "SELECT noteID, itemID, note, clientDate FROM itemNotes WHERE itemID = ? ORDER BY clientDate DESC";
    let mut stmt = conn.prepare(sql)
        .map_err(|e| format!("准备查询失败: {}", e))?;

    let notes: Vec<Note> = stmt
        .query_map(params![item_id], |row| {
            let note_content: String = row.get(2)?;
            Ok(note_content)
        })
        .map_err(|e| format!("查询笔记失败: {}", e))?
        .filter_map(|result| {
            result.ok().and_then(|content| {
                // 尝试解析 ZoPlus 格式的笔记
                parse_zoplus_note(&content)
            })
        })
        .collect();

    eprintln!("[命令] 获取到 {} 条笔记", notes.len());
    Ok(notes)
}

/// Tauri 命令：删除笔记
#[tauri::command]
pub fn delete_note(note_id: String) -> Result<bool, String> {
    eprintln!("[命令] delete_note 被调用: note_id={}", note_id);

    let guard = crate::db::connection::get_connection()
        .map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let conn = guard.as_ref().ok_or_else(|| "数据库连接未初始化".to_string())?;

    // 从 itemNotes 表删除笔记
    let sql = "DELETE FROM itemNotes WHERE noteID = ?";
    conn.execute(sql, params![note_id])
        .map_err(|e| format!("删除笔记失败: {}", e))?;

    eprintln!("[命令] 笔记已删除: note_id={}", note_id);
    Ok(true)
}

/// Tauri 命令：更新笔记
#[tauri::command]
pub fn update_note(note: Note) -> Result<bool, String> {
    eprintln!("[命令] update_note 被调用: note_id={}", note.note_id);

    let guard = crate::db::connection::get_connection()
        .map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let conn = guard.as_ref().ok_or_else(|| "数据库连接未初始化".to_string())?;

    // 将笔记序列化为 JSON
    let note_json = note.to_json().map_err(|e| format!("序列化笔记失败: {}", e))?;

    // 构建更新后的 note 内容
    let note_content = format!(
        "{{\"zoplus_note\":true,\"version\":1}}\n{}",
        note_json
    );

    // 更新 itemNotes 表
    let sql = "UPDATE itemNotes SET note = ?, clientDate = datetime('now') WHERE noteID = ?";
    conn.execute(sql, params![note_content, note.note_id])
        .map_err(|e| format!("更新笔记失败: {}", e))?;

    eprintln!("[命令] 笔记已更新: note_id={}", note.note_id);
    Ok(true)
}

/// Tauri 命令：导出笔记为 Markdown
#[tauri::command]
pub fn export_note_as_markdown(note: Note) -> String {
    note.to_markdown()
}

/// Tauri 命令：批量导出指定文献的所有笔记为 Markdown
#[tauri::command]
pub fn export_all_notes_as_markdown(notes: Vec<Note>, item_title: String) -> String {
    let mut md = String::new();

    md.push_str(&format!("# {} - 笔记汇总\n\n", item_title));
    md.push_str("---\n\n");

    for note in notes {
        md.push_str(&note.to_markdown());
        md.push_str("\n\n---\n\n");
    }

    md
}

/// 解析 ZoPlus 格式的笔记
fn parse_zoplus_note(content: &str) -> Option<Note> {
    // 检查是否是 ZoPlus 格式的笔记
    if !content.contains("\"zoplus_note\":true") {
        return None;
    }

    // 查找 JSON 内容的开始位置（跳过元数据头）
    if let Some(start) = content.find("}}\n") {
        let json_str = &content[start + 2..];
        Note::from_json(json_str).ok()
    } else {
        // 尝试直接解析（可能是旧格式）
        Note::from_json(content).ok()
    }
}