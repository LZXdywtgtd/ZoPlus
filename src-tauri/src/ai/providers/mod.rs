//! AI 厂商实现模块
//!
//!包含各 AI 厂商的具体实现

pub mod openai;
pub mod anthropic;
pub mod deepseek;
pub mod doubao;
pub mod qwen;
pub mod glm;
pub mod minimax;
pub mod mimo;

use std::sync::Arc;
use crate::ai::traits::AIProvider;
use crate::ai::{AIConfig, AIProviderType, AIError, ModelInfo};

/// 创建指定厂商的 AI Provider 实例
pub fn create_provider(config: &AIConfig) -> Result<Arc<dyn AIProvider>, AIError> {
    let api_key = &config.api_key;
    if api_key.is_empty() {
        return Err(AIError::ApiKeyNotConfigured);
    }

    let provider: Arc<dyn AIProvider> = match config.provider {
        AIProviderType::OpenAI => {
            Arc::new(openai::OpenAIProvider::new(api_key.clone(), config.base_url.clone())?) as Arc<dyn AIProvider>
        }
        AIProviderType::Anthropic => {
            Arc::new(anthropic::AnthropicProvider::new(api_key.clone(), config.base_url.clone())?) as Arc<dyn AIProvider>
        }
        AIProviderType::DeepSeek => {
            Arc::new(deepseek::DeepSeekProvider::new(api_key.clone(), config.base_url.clone())?) as Arc<dyn AIProvider>
        }
        AIProviderType::Doubao => {
            Arc::new(doubao::DoubaoProvider::new(api_key.clone(), config.base_url.clone())?) as Arc<dyn AIProvider>
        }
        AIProviderType::Qwen => {
            Arc::new(qwen::QwenProvider::new(api_key.clone(), config.base_url.clone())?) as Arc<dyn AIProvider>
        }
        AIProviderType::Glm => {
            Arc::new(glm::GlmProvider::new(api_key.clone(), config.base_url.clone())?) as Arc<dyn AIProvider>
        }
        AIProviderType::MiniMax => {
            Arc::new(minimax::MiniMaxProvider::new(api_key.clone(), config.base_url.clone())?) as Arc<dyn AIProvider>
        }
        AIProviderType::MiMo => {
            Arc::new(mimo::MiMoProvider::new(api_key.clone(), config.base_url.clone())?) as Arc<dyn AIProvider>
        }
    };
    Ok(provider)
}

/// 获取所有厂商的可用模型
pub fn get_all_models() -> Vec<ModelInfo> {
    let mut models = Vec::new();
    let openai_provider = openai::OpenAIProvider::new("dummy".to_string(), None).unwrap();
    let anthropic_provider = anthropic::AnthropicProvider::new("dummy".to_string(), None).unwrap();
    let deepseek_provider = deepseek::DeepSeekProvider::new("dummy".to_string(), None).unwrap();
    let doubao_provider = doubao::DoubaoProvider::new("dummy".to_string(), None).unwrap();
    let qwen_provider = qwen::QwenProvider::new("dummy".to_string(), None).unwrap();
    let glm_provider = glm::GlmProvider::new("dummy".to_string(), None).unwrap();
    let minimax_provider = minimax::MiniMaxProvider::new("dummy".to_string(), None).unwrap();
    let mimo_provider = mimo::MiMoProvider::new("dummy".to_string(), None).unwrap();

    models.extend(openai_provider.get_available_models());
    models.extend(anthropic_provider.get_available_models());
    models.extend(deepseek_provider.get_available_models());
    models.extend(doubao_provider.get_available_models());
    models.extend(qwen_provider.get_available_models());
    models.extend(glm_provider.get_available_models());
    models.extend(minimax_provider.get_available_models());
    models.extend(mimo_provider.get_available_models());
    models
}