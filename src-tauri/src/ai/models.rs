//! AI 模型信息结构体定义
//!
//! 定义各厂商模型的结构化表示

use serde::{Deserialize, Serialize};
use crate::ai::errors::AIError;

/// 消息角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// 系统消息
    System,
    /// 用户消息
    User,
    /// 助手消息
    Assistant,
}

impl MessageRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        }
    }
}

/// 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 角色
    pub role: MessageRole,
    /// 内容
    pub content: String,
}

impl Message {
    /// 创建用户消息
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    /// 创建助手消息
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }

    /// 创建系统消息
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }
}

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// 模型 ID
    pub id: String,
    /// 模型名称
    pub name: String,
    /// 所属厂商
    pub provider: String,
    /// 最大 token 数
    pub max_tokens: u32,
    /// 是否支持流式输出
    pub supports_streaming: bool,
}

/// 模型价格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPrice {
    /// 输入价格（每百万 token）
    pub input_price: f64,
    /// 输出价格（每百万 token）
    pub output_price: f64,
    /// 货币单位
    pub currency: String,
}

/// AI 厂商枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AIProviderType {
    /// OpenAI
    OpenAI,
    /// Anthropic
    Anthropic,
    /// DeepSeek
    DeepSeek,
    /// 豆包
    Doubao,
    /// 通义千问
    Qwen,
    /// 智谱 GLM
    Glm,
    /// MiniMax
    MiniMax,
    /// 小米 MiMo
    MiMo,
}

impl AIProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AIProviderType::OpenAI => "openai",
            AIProviderType::Anthropic => "anthropic",
            AIProviderType::DeepSeek => "deepseek",
            AIProviderType::Doubao => "doubao",
            AIProviderType::Qwen => "qwen",
            AIProviderType::Glm => "glm",
            AIProviderType::MiniMax => "minimax",
            AIProviderType::MiMo => "mimo",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AIProviderType::OpenAI => "OpenAI",
            AIProviderType::Anthropic => "Anthropic",
            AIProviderType::DeepSeek => "DeepSeek",
            AIProviderType::Doubao => "豆包",
            AIProviderType::Qwen => "通义千问",
            AIProviderType::Glm => "智谱GLM",
            AIProviderType::MiniMax => "MiniMax",
            AIProviderType::MiMo => "小米MiMo",
        }
    }
}

/// 流式响应迭代器
pub struct Stream<T> {
    inner: tokio::sync::mpsc::Receiver<Result<T, AIError>>,
}

impl<T> Stream<T> {
    /// 创建新的流
    pub fn new(rx: tokio::sync::mpsc::Receiver<Result<T, AIError>>) -> Self {
        Self { inner: rx }
    }
}

impl<T: Send + 'static> Stream<T> {
    /// 异步迭代流内容
    pub async fn next(&mut self) -> Option<Result<T, AIError>> {
        self.inner.recv().await
    }
}