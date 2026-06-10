//! AI Provider 统一接口定义
//!
//! 定义所有 AI 厂商必须实现的统一接口

use super::{AIError, ModelInfo, ModelPrice, Stream};
use crate::ai::models::Message;

/// AI Provider 统一接口
///
/// 所有 AI 厂商必须实现此 trait，以便统一调用
pub trait AIProvider: Send + Sync {
    /// 发送对话补全请求
    ///
    /// # 参数
    /// * `messages` - 对话消息列表
    ///
    /// # 返回值
    /// * `Result<String, AIError>` - 助手的回复内容
    fn chat_completion(&self, messages: Vec<Message>) -> Result<String, AIError>;

    /// 流式对话补全
    ///
    /// # 参数
    /// * `messages` - 对话消息列表
    ///
    /// # 返回值
    /// * `Result<Stream<String>, AIError>` - 流式响应迭代器
    fn stream_chat_completion(&self, messages: Vec<Message>) -> Result<Stream<String>, AIError>;

    /// 获取可用模型列表
    fn get_available_models(&self) -> Vec<ModelInfo>;

    /// 获取模型价格
    ///
    /// # 参数
    /// * `model_id` - 模型 ID
    ///
    /// # 返回值
    /// * `Option<ModelPrice>` - 模型价格信息
    fn get_model_price(&self, model_id: &str) -> Option<ModelPrice>;

    /// 获取厂商类型
    fn provider_type(&self) -> super::AIProviderType;

    /// 获取默认模型 ID
    fn default_model(&self) -> &str;

    /// 测试连接
    fn test_connection(&self) -> Result<bool, AIError> {
        let messages = vec![Message::user("Hello")];
        match self.chat_completion(messages) {
            Ok(_) => Ok(true),
            Err(AIError::ApiKeyInvalid(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}