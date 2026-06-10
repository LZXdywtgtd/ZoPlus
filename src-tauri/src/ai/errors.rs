//! AI 模块错误类型定义
//!
//! 定义 AI 调用过程中可能出现的错误类型

use thiserror::Error;

/// AI 模块错误类型
#[derive(Error, Debug, Clone)]
pub enum AIError {
    /// API密钥未配置
    #[error("API 密钥未配置")]
    ApiKeyNotConfigured,

    /// API 密钥无效
    #[error("API 密钥无效: {0}")]
    ApiKeyInvalid(String),

    /// 网络请求失败
    #[error("网络请求失败: {0}")]
    NetworkError(String),

    /// API 调用失败
    #[error("API 调用失败: {0}")]
    ApiError(String),

    /// 模型不支持
    #[error("模型不支持: {0}")]
    ModelNotSupported(String),

    /// 流式响应失败
    #[error("流式响应失败: {0}")]
    StreamError(String),

    /// 响应解析失败
    #[error("响应解析失败: {0}")]
    ParseError(String),

    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// 速率限制
    #[error("速率限制: {0}")]
    RateLimit(String),

    /// 未知错误
    #[error("未知错误: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for AIError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_connect() {
            AIError::NetworkError(format!("连接失败: {}", err))
        } else if err.is_timeout() {
            AIError::NetworkError(format!("请求超时: {}", err))
        } else {
            AIError::NetworkError(format!("网络错误: {}", err))
        }
    }
}