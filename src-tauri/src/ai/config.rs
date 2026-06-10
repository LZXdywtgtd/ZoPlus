//! AI 配置管理模块
//!
//! 负责 AI 配置的加载、存储和加密

use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use tauri::AppHandle;

use super::{AIProviderType, ModelInfo};

/// AI 全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// 是否启用 AI 功能
    pub enabled: bool,
    /// 当前选中的厂商
    pub provider: AIProviderType,
    /// 当前选中的模型
    pub model_id: String,
    /// API 密钥（加密存储）
    pub api_key: String,
    /// API 基础 URL（可选，用于代理）
    pub base_url: Option<String>,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: AIProviderType::MiniMax,
            model_id: "MiniMax-M2.7".to_string(),
            api_key: String::new(),
            base_url: None,
        }
    }
}

/// AI 配置管理器
pub struct AIConfigManager {
    config: RwLock<AIConfig>,
}

impl AIConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Self {
        Self {
            config: RwLock::new(AIConfig::default()),
        }
    }

    /// 获取当前配置
    pub fn get_config(&self) -> AIConfig {
        self.config.read().unwrap().clone()
    }

    /// 更新配置
    pub fn update_config(&self, config: AIConfig) {
        *self.config.write().unwrap() = config;
    }

    /// 更新 API 密钥
    pub fn update_api_key(&self, api_key: String) {
        self.config.write().unwrap().api_key = api_key;
    }

    /// 更新选中的模型
    pub fn update_model(&self, model_id: String) {
        self.config.write().unwrap().model_id = model_id;
    }

    /// 更新厂商
    pub fn update_provider(&self, provider: AIProviderType) {
        self.config.write().unwrap().provider = provider;
    }

    /// 启用/禁用 AI
    pub fn set_enabled(&self, enabled: bool) {
        self.config.write().unwrap().enabled = enabled;
    }

    /// 检查是否已配置
    pub fn is_configured(&self) -> bool {
        let config = self.config.read().unwrap();
        !config.api_key.is_empty()
    }
}

impl Default for AIConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 从环境变量加载 API密钥
pub fn load_api_key_from_env(provider: AIProviderType) -> Option<String> {
    let env_key = match provider {
        AIProviderType::OpenAI => "OPENAI_API_KEY",
        AIProviderType::Anthropic => "ANTHROPIC_API_KEY",
        AIProviderType::DeepSeek => "DEEPSEEK_API_KEY",
        AIProviderType::Doubao => "DOUBAO_API_KEY",
        AIProviderType::Qwen => "QWEN_API_KEY",
        AIProviderType::Glm => "GLM_API_KEY",
        AIProviderType::MiniMax => "MINIMAX_API_KEY",
        AIProviderType::MiMo => "MIMO_API_KEY",
    };

    std::env::var(env_key).ok()
}