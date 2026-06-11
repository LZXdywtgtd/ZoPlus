//! 通义千问 Provider 实现
//!
//! 支持 Qwen 3 系列模型

use futures_util::StreamExt;
use crate::ai::client::HTTPClientFactory;
use crate::ai::{AIProvider, AIProviderType, ModelInfo, ModelPrice, Stream, AIError};
use crate::ai::models::Message;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;

/// 通义千问 Provider
pub struct QwenProvider {
    api_key: String,
    base_url: String,
    client: Client,
}

impl QwenProvider {
    /// 创建新的通义千问 Provider
    pub fn new(api_key: String, base_url: Option<String>) -> Result<Self, AIError> {
        let base_url = base_url.unwrap_or_else(|| "https://dashscope.aliyuncs.com".to_string());
        Ok(Self {
            api_key,
            base_url,
            client: HTTPClientFactory::create(),
        })
    }

    fn get_chat_url(&self) -> String {
        format!("{}/api/v1/chat/completions", self.base_url)
    }
}

impl AIProvider for QwenProvider {
    fn chat_completion(&self, messages: Vec<Message>) -> Result<String, AIError> {
        let request = QwenChatRequest {
            model: self.default_model().to_string(),
            messages: messages
                .into_iter()
                .map(|m| QwenMessage {
                    role: m.role.as_str().to_string(),
                    content: m.content,
                })
                .collect(),
            stream: false,
        };

        let rt = Handle::current();
        rt.block_on(async {
            let response = self
                .client
                .post(self.get_chat_url())
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await;

            match response {
                Ok(response) => {
                    if !response.status().is_success() {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        return Err(AIError::ApiError(format!(
                            "状态码: {}, 响应: {}",
                            status, error_text
                        )));
                    }
                    let chat_response: QwenChatResponse = response.json().await?;
                    Ok(chat_response
                        .choices
                        .first()
                        .map(|c| c.message.content.clone())
                        .unwrap_or_default())
                }
                Err(e) => Err(AIError::from(e))
            }
        })
    }

    fn stream_chat_completion(&self, messages: Vec<Message>) -> Result<Stream<String>, AIError> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let request = QwenChatRequest {
            model: self.default_model().to_string(),
            messages: messages
                .into_iter()
                .map(|m| QwenMessage {
                    role: m.role.as_str().to_string(),
                    content: m.content,
                })
                .collect(),
            stream: true,
        };

        let url = self.get_chat_url();
        let api_key = self.api_key.clone();

        tokio::spawn(async move {
            let response = match HTTPClientFactory::create()
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(Err(AIError::from(e))).await;
                    return;
                }
            };

            if !response.status().is_success() {
                let _ = tx.send(Err(AIError::ApiError(format!("流式请求失败: {}", response.status())))).await;
                return;
            }

            let mut stream = response.bytes_stream();
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        for line in text.lines() {
                            if line.starts_with("data: ") {
                                let data = &line[6..];
                                if data == "[DONE]" {
                                    let _ = tx.send(Err(AIError::Unknown("流结束".to_string()))).await;
                                    return;
                                }
                                if let Ok(event) = serde_json::from_str::<QwenStreamEvent>(data) {
                                    if let Some(choice) = event.choices.first() {
                                        if !choice.delta.content.is_empty() {
                                            let _ = tx.send(Ok(choice.delta.content.clone())).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(AIError::StreamError(e.to_string()))).await;
                        return;
                    }
                }
            }
            let _ = tx.send(Err(AIError::Unknown("流正常结束".to_string()))).await;
        });

        Ok(Stream::new(rx))
    }

    fn get_available_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "qwen-3-max".to_string(),
                name: "Qwen 3 Max".to_string(),
                provider: "qwen".to_string(),
                max_tokens: 32000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "qwen-3-72b".to_string(),
                name: "Qwen 3 72B".to_string(),
                provider: "qwen".to_string(),
                max_tokens: 32000,
                supports_streaming: true,
            },
        ]
    }

    fn get_model_price(&self, model_id: &str) -> Option<ModelPrice> {
        match model_id {
            "qwen-3-max" => Some(ModelPrice {
                input_price: 0.4,
                output_price: 1.6,
                currency: "CNY".to_string(),
            }),
            "qwen-3-72b" => Some(ModelPrice {
                input_price: 0.2,
                output_price: 0.8,
                currency: "CNY".to_string(),
            }),
            _ => None,
        }
    }

    fn provider_type(&self) -> AIProviderType {
        AIProviderType::Qwen
    }

    fn default_model(&self) -> &str {
        "qwen-3-max"
    }
}

// 通义千问 API 请求/响应结构
#[derive(Serialize)]
struct QwenChatRequest {
    model: String,
    messages: Vec<QwenMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct QwenMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct QwenChatResponse {
    choices: Vec<QwenChoice>,
}

#[derive(Deserialize)]
struct QwenChoice {
    message: QwenMessageContent,
}

#[derive(Deserialize)]
struct QwenMessageContent {
    content: String,
}

#[derive(Deserialize)]
struct QwenStreamEvent {
    choices: Vec<QwenStreamChoice>,
}

#[derive(Deserialize)]
struct QwenStreamChoice {
    delta: QwenDelta,
}

#[derive(Deserialize)]
struct QwenDelta {
    content: String,
}