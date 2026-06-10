//! 智谱 GLM Provider 实现
//!
//! 支持 GLM-5 系列模型

use bytes::Bytes;
use futures_util::StreamExt;
use crate::ai::client::HTTPClientFactory;
use crate::ai::{AIProvider, AIProviderType, ModelInfo, ModelPrice, Stream, AIError};
use crate::ai::models::Message;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;

/// 智谱 GLM Provider
pub struct GlmProvider {
    api_key: String,
    base_url: String,
    client: Client,
}

impl GlmProvider {
    /// 创建新的智谱 GLM Provider
    pub fn new(api_key: String, base_url: Option<String>) -> Result<Self, AIError> {
        let base_url = base_url.unwrap_or_else(|| "https://open.bigmodel.cn".to_string());
        Ok(Self {
            api_key,
            base_url,
            client: HTTPClientFactory::create(),
        })
    }

    fn get_chat_url(&self) -> String {
        format!("{}/api/llm-pro/chat/completions", self.base_url)
    }
}

impl AIProvider for GlmProvider {
    fn chat_completion(&self, messages: Vec<Message>) -> Result<String, AIError> {
        let request = GlmChatRequest {
            model: self.default_model().to_string(),
            messages: messages
                .into_iter()
                .map(|m| GlmMessage {
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
                    let chat_response: GlmChatResponse = response.json().await?;
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

        let request = GlmChatRequest {
            model: self.default_model().to_string(),
            messages: messages
                .into_iter()
                .map(|m| GlmMessage {
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
                                if let Ok(event) = serde_json::from_str::<GlmStreamEvent>(data) {
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
                id: "glm-5.1".to_string(),
                name: "GLM-5.1".to_string(),
                provider: "glm".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "glm-5".to_string(),
                name: "GLM-5".to_string(),
                provider: "glm".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "glm-5-turbo".to_string(),
                name: "GLM-5-Turbo".to_string(),
                provider: "glm".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "glm-5v-turbo".to_string(),
                name: "GLM-5V-Turbo".to_string(),
                provider: "glm".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "glm-4.7".to_string(),
                name: "GLM-4.7".to_string(),
                provider: "glm".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "glm-4.7-flash".to_string(),
                name: "GLM-4.7-Flash".to_string(),
                provider: "glm".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
            },
        ]
    }

    fn get_model_price(&self, model_id: &str) -> Option<ModelPrice> {
        match model_id {
            "glm-5.1" => Some(ModelPrice {
                input_price: 0.5,
                output_price: 2.0,
                currency: "CNY".to_string(),
            }),
            "glm-5" => Some(ModelPrice {
                input_price: 0.4,
                output_price: 1.6,
                currency: "CNY".to_string(),
            }),
            "glm-5-turbo" => Some(ModelPrice {
                input_price: 0.1,
                output_price: 0.4,
                currency: "CNY".to_string(),
            }),
            "glm-5v-turbo" => Some(ModelPrice {
                input_price: 0.5,
                output_price: 2.0,
                currency: "CNY".to_string(),
            }),
            "glm-4.7" => Some(ModelPrice {
                input_price: 0.2,
                output_price: 0.8,
                currency: "CNY".to_string(),
            }),
            "glm-4.7-flash" => Some(ModelPrice {
                input_price: 0.05,
                output_price: 0.2,
                currency: "CNY".to_string(),
            }),
            _ => None,
        }
    }

    fn provider_type(&self) -> AIProviderType {
        AIProviderType::Glm
    }

    fn default_model(&self) -> &str {
        "glm-5"
    }
}

//智谱 GLM API 请求/响应结构
#[derive(Serialize)]
struct GlmChatRequest {
    model: String,
    messages: Vec<GlmMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct GlmMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct GlmChatResponse {
    choices: Vec<GlmChoice>,
}

#[derive(Deserialize)]
struct GlmChoice {
    message: GlmMessageContent,
}

#[derive(Deserialize)]
struct GlmMessageContent {
    content: String,
}

#[derive(Deserialize)]
struct GlmStreamEvent {
    choices: Vec<GlmStreamChoice>,
}

#[derive(Deserialize)]
struct GlmStreamChoice {
    delta: GlmDelta,
}

#[derive(Deserialize)]
struct GlmDelta {
    content: String,
}