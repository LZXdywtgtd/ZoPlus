//! RAG Tauri 命令接口
//!
//! 提供跨文献智能问答的前端调用接口

use std::sync::Mutex;
use tauri::State;

use crate::ai::create_provider;
use crate::search::SearchState;

use super::rag::{ChatMessage, DocumentContext, RagConfig, RagEngine, MAX_TOP_K};

/// RAG 状态管理（用于会话和配置存储）
pub struct RagState {
    /// 当前会话 ID
    pub session_id: Mutex<Option<String>>,
    /// RAG 配置
    pub config: Mutex<RagConfig>,
}

impl RagState {
    pub fn new() -> Self {
        Self {
            session_id: Mutex::new(None),
            config: Mutex::new(RagConfig::default()),
        }
    }
}

impl Default for RagState {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建 RAG 引擎（每次调用创建新实例，因为搜索引擎是无状态的）
fn create_engine<'a>(
    ai_state: &State<'_, crate::ai::commands::AIState>,
    search_state: &'a SearchState,
) -> Result<RagEngine<'a>, String> {
    let config = ai_state.config_manager.get_config();

    if config.api_key.is_empty() {
        return Err("API 密钥未配置，请在设置中配置 AI".to_string());
    }

    if !config.enabled {
        return Err("AI 功能已禁用，请在设置中启用".to_string());
    }

    let provider = create_provider(&config).map_err(|e| e.to_string())?;
    Ok(RagEngine::new(provider, search_state))
}

/// Tauri 命令：发送聊天消息
#[tauri::command]
pub async fn ai_chat(
    ai_state: State<'_, crate::ai::commands::AIState>,
    search_state: State<'_, SearchState>,
    _rag_state: State<'_, RagState>,
    message: String,
) -> Result<ChatMessage, String> {
    eprintln!("[命令] ai_chat 被调用: message={}", message);

    let engine = create_engine(&ai_state, &search_state)?;

    // 保存用户消息
    engine.save_user_message(message.clone());

    // 处理对话
    let response = engine.chat(message).await.map_err(|e| e.to_string())?;

    // 保存助手消息
    engine.save_assistant_message(response.content.clone(), response.citations.clone());

    eprintln!("[命令] ai_chat 完成");
    Ok(response)
}

/// Tauri 命令：流式聊天
#[tauri::command]
pub async fn ai_chat_stream(
    ai_state: State<'_, crate::ai::commands::AIState>,
    search_state: State<'_, SearchState>,
    _rag_state: State<'_, RagState>,
    message: String,
) -> Result<Vec<String>, String> {
    eprintln!("[命令] ai_chat_stream 被调用: message={}", message);

    let engine = create_engine(&ai_state, &search_state)?;

    // 保存用户消息
    engine.save_user_message(message.clone());

    // 处理流式对话
    let mut stream = engine.chat_streaming(message).await.map_err(|e| e.to_string())?;

    // 收集所有流式片段
    let mut chunks: Vec<String> = Vec::new();
    while let Some(result) = stream.next().await {
        match result {
            Ok(chunk) => chunks.push(chunk),
            Err(e) => return Err(e.to_string()),
        }
    }

    // 合并所有片段
    let full_response = chunks.join("");

    // 保存助手消息（获取检索到的文献上下文）
    let messages = engine.get_session_messages();
    if let Some(last_msg) = messages.last() {
        engine.save_assistant_message(full_response.clone(), last_msg.citations.clone());
    }

    eprintln!("[命令] ai_chat_stream 完成，共 {} 个片段", chunks.len());
    Ok(chunks)
}

/// Tauri 命令：获取聊天历史
#[tauri::command]
pub fn get_chat_history(
    ai_state: State<'_, crate::ai::commands::AIState>,
    search_state: State<'_, SearchState>,
    _rag_state: State<'_, RagState>,
) -> Result<Vec<ChatMessage>, String> {
    eprintln!("[命令] get_chat_history 被调用");

    let engine = create_engine(&ai_state, &search_state)?;
    Ok(engine.get_session_messages())
}

/// Tauri 命令：清除聊天历史
#[tauri::command]
pub fn clear_chat_history(
    ai_state: State<'_, crate::ai::commands::AIState>,
    search_state: State<'_, SearchState>,
    _rag_state: State<'_, RagState>,
) -> Result<bool, String> {
    eprintln!("[命令] clear_chat_history 被调用");

    let engine = create_engine(&ai_state, &search_state)?;
    engine.clear_session();
    Ok(true)
}

/// Tauri 命令：获取当前引用的文献上下文
#[tauri::command]
pub fn get_chat_context(
    ai_state: State<'_, crate::ai::commands::AIState>,
    search_state: State<'_, SearchState>,
    _rag_state: State<'_, RagState>,
) -> Result<Vec<DocumentContext>, String> {
    eprintln!("[命令] get_chat_context 被调用");

    let engine = create_engine(&ai_state, &search_state)?;
    Ok(engine.get_session_context())
}

/// Tauri 命令：更新 RAG 配置
#[tauri::command]
pub fn update_rag_config(
    _ai_state: State<'_, crate::ai::commands::AIState>,
    _search_state: State<'_, SearchState>,
    rag_state: State<'_, RagState>,
    top_k: Option<usize>,
    streaming: Option<bool>,
    min_score: Option<f32>,
) -> Result<RagConfig, String> {
    eprintln!("[命令] update_rag_config 被调用: top_k={:?}, streaming={:?}, min_score={:?}",
        top_k, streaming, min_score);

    let mut config = rag_state.config.lock().unwrap();

    if let Some(k) = top_k {
        config.top_k = k.min(MAX_TOP_K);
    }
    if let Some(s) = streaming {
        config.streaming = s;
    }
    if let Some(score) = min_score {
        config.min_score = score.clamp(0.0, 1.0);
    }

    let result_config = config.clone();
    Ok(result_config)
}

/// Tauri 命令：获取 RAG 配置
#[tauri::command]
pub fn get_rag_config(
    _ai_state: State<'_, crate::ai::commands::AIState>,
    _search_state: State<'_, SearchState>,
    rag_state: State<'_, RagState>,
) -> Result<RagConfig, String> {
    let config = rag_state.config.lock().unwrap();
    Ok(config.clone())
}