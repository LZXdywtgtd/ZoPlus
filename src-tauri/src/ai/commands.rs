//! AI Tauri 命令接口
//!
//! 提供 AI 功能的前端调用接口

use std::sync::Arc;
use tauri::State;

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