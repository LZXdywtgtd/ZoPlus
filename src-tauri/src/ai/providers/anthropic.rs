//! Anthropic Provider 实现
//!
//! 支持 Claude 系列模型

use futures_util::StreamExt;
use crate::ai::client::HTTPClientFactory;
use crate::ai::{
    AIProvider, AIProviderType, ModelInfo, ModelPrice, Stream, AIError,
};
use crate::ai::models::Message;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;

/// Anthropic Provider
pub struct AnthropicProvider {
    api_key: String,
    base_url: String,
    client: Client,
}

impl AnthropicProvider {
    /// 创建新的 Anthropic Provider
    pub fn new(api_key: String, base_url: Option<String>) -> Result<Self, AIError> {
        let base_url = base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string());
        Ok(Self {
            api_key,
            base_url,
            client: HTTPClientFactory::create(),
        })
    }

    fn get_chat_url(&self) -> String {
        format!("{}/v1/messages", self.base_url)
    }
}

impl AIProvider for AnthropicProvider {
    fn chat_completion(&self, messages: Vec<Message>) -> Result<String, AIError> {
        // 将 messages 合并为单个 prompt
        let prompt: String = messages
            .into_iter()
            .map(|m| format!("{}: {}", m.role.as_str(), m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let request = AnthropicRequest {
            model: self.default_model().to_string(),
            max_tokens: 4096,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let rt = Handle::current();
        rt.block_on(async {
            let response = self
                .client
                .post(self.get_chat_url())
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
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
                    let chat_response: AnthropicResponse = response.json().await?;
                    Ok(chat_response.content.first().map(|c| c.text.clone()).unwrap_or_default())
                }
                Err(e) => Err(AIError::from(e))
            }
        })
    }

    fn stream_chat_completion(&self, messages: Vec<Message>) -> Result<Stream<String>, AIError> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let prompt: String = messages
            .into_iter()
            .map(|m| format!("{}: {}", m.role.as_str(), m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let request = AnthropicRequest {
            model: self.default_model().to_string(),
            max_tokens: 4096,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let url = self.get_chat_url();
        let api_key = self.api_key.clone();

        tokio::spawn(async move {
            let response = match HTTPClientFactory::create()
                .post(&url)
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
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
                let _ = tx
                    .send(Err(AIError::ApiError(format!(
                        "流式请求失败: {}",
                        response.status()
                    ))))
                    .await;
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
                                if let Ok(event) = serde_json::from_str::<AnthropicStreamEvent>(data) {
                                    if let Some(content) = event.content.first() {
                                        if !content.text.is_empty() {
                                            let _ = tx.send(Ok(content.text.clone())).await;
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
                id: "claude-opus-4.6".to_string(),
                name: "Claude Opus 4.6".to_string(),
                provider: "anthropic".to_string(),
                max_tokens: 200000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "claude-sonnet-4.6".to_string(),
                name: "Claude Sonnet 4.6".to_string(),
                provider: "anthropic".to_string(),
                max_tokens: 200000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "claude-haiku-4.5".to_string(),
                name: "Claude Haiku 4.5".to_string(),
                provider: "anthropic".to_string(),
                max_tokens: 200000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "claude-opus-4.5".to_string(),
                name: "Claude Opus 4.5".to_string(),
                provider: "anthropic".to_string(),
                max_tokens: 200000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "claude-3.7-sonnet".to_string(),
                name: "Claude 3.7 Sonnet".to_string(),
                provider: "anthropic".to_string(),
                max_tokens: 200000,
                supports_streaming: true,
            },
        ]
    }

    fn get_model_price(&self, model_id: &str) -> Option<ModelPrice> {
        match model_id {
            "claude-opus-4.6" => Some(ModelPrice {
                input_price: 15.0,
                output_price: 75.0,
                currency: "USD".to_string(),
            }),
            "claude-sonnet-4.6" => Some(ModelPrice {
                input_price: 3.0,
                output_price: 15.0,
                currency: "USD".to_string(),
            }),
            "claude-haiku-4.5" => Some(ModelPrice {
                input_price: 0.8,
                output_price: 4.0,
                currency: "USD".to_string(),
            }),
            "claude-opus-4.5" => Some(ModelPrice {
                input_price: 15.0,
                output_price: 75.0,
                currency: "USD".to_string(),
            }),
            "claude-3.7-sonnet" => Some(ModelPrice {
                input_price: 3.0,
                output_price: 15.0,
                currency: "USD".to_string(),
            }),
            _ => None,
        }
    }

    fn provider_type(&self) -> AIProviderType {
        AIProviderType::Anthropic
    }

    fn default_model(&self) -> &str {
        "claude-sonnet-4.6"
    }
}

// Anthropic API 请求/响应结构
#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
}

#[derive(Deserialize)]
struct AnthropicContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct AnthropicStreamEvent {
    content: Vec<AnthropicContentBlock>,
}