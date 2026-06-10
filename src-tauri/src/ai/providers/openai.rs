//! OpenAI Provider 实现
//!
//! 支持 GPT 系列模型

use bytes::Bytes;
use futures_util::StreamExt;
use crate::ai::client::HTTPClientFactory;
use crate::ai::{
    AIProvider, AIProviderType, ModelInfo, ModelPrice, Stream, AIError,
};
use crate::ai::models::Message;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;

/// OpenAI Provider
pub struct OpenAIProvider {
    api_key: String,
    base_url: String,
    client: Client,
}

impl OpenAIProvider {
    /// 创建新的 OpenAI Provider
    pub fn new(api_key: String, base_url: Option<String>) -> Result<Self, AIError> {
        let base_url = base_url.unwrap_or_else(|| "https://api.openai.com".to_string());
        Ok(Self {
            api_key,
            base_url,
            client: HTTPClientFactory::create(),
        })
    }

    fn get_models_url(&self) -> String {
        format!("{}/v1/models", self.base_url)
    }

    fn get_chat_url(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url)
    }
}

impl AIProvider for OpenAIProvider {
    fn chat_completion(&self, messages: Vec<Message>) -> Result<String, AIError> {
        let request = OpenAIChatRequest {
            model: self.default_model().to_string(),
            messages: messages
                .into_iter()
                .map(|m| OpenAIMessage {
                    role: m.role.as_str().to_string(),
                    content: m.content,
                })
                .collect(),
            stream: false,
        };

        let rt = Handle::current();
        let result = rt.block_on(async {
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
                            "状态码: {},响应: {}",
                            status, error_text
                        )));
                    }
                    let chat_response: OpenAIChatResponse = response.json().await?;
                    Ok(chat_response
                        .choices
                        .first()
                        .map(|c| c.message.content.clone())
                        .unwrap_or_default())
                }
                Err(e) => Err(AIError::from(e))
            }
        });
        result
    }

    fn stream_chat_completion(&self, messages: Vec<Message>) -> Result<Stream<String>, AIError> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let request = OpenAIChatRequest {
            model: self.default_model().to_string(),
            messages: messages
                .into_iter()
                .map(|m| OpenAIMessage {
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
                                if let Ok(event) = serde_json::from_str::<OpenAIStreamEvent>(data) {
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
                id: "gpt-5.5-pro".to_string(),
                name: "GPT-5.5 Pro".to_string(),
                provider: "openai".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "gpt-5.5".to_string(),
                name: "GPT-5.5".to_string(),
                provider: "openai".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "gpt-5.4-pro".to_string(),
                name: "GPT-5.4 Pro".to_string(),
                provider: "openai".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "gpt-5.4".to_string(),
                name: "GPT-5.4".to_string(),
                provider: "openai".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "gpt-5.4-mini".to_string(),
                name: "GPT-5.4 mini".to_string(),
                provider: "openai".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "gpt-5.4-nano".to_string(),
                name: "GPT-5.4 nano".to_string(),
                provider: "openai".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
            },
            ModelInfo {
                id: "gpt-5".to_string(),
                name: "GPT-5".to_string(),
                provider: "openai".to_string(),
                max_tokens: 128000,
                supports_streaming: true,
            },
        ]
    }

    fn get_model_price(&self, model_id: &str) -> Option<ModelPrice> {
        match model_id {
            "gpt-5.5-pro" => Some(ModelPrice {
                input_price: 7.5,
                output_price: 30.0,
                currency: "USD".to_string(),
            }),
            "gpt-5.5" => Some(ModelPrice {
                input_price: 5.0,
                output_price: 20.0,
                currency: "USD".to_string(),
            }),
            "gpt-5.4-pro" => Some(ModelPrice {
                input_price: 5.0,
                output_price: 20.0,
                currency: "USD".to_string(),
            }),
            "gpt-5.4" => Some(ModelPrice {
                input_price: 2.5,
                output_price: 10.0,
                currency: "USD".to_string(),
            }),
            "gpt-5.4-mini" => Some(ModelPrice {
                input_price: 0.15,
                output_price: 0.6,
                currency: "USD".to_string(),
            }),
            "gpt-5.4-nano" => Some(ModelPrice {
                input_price: 0.05,
                output_price: 0.2,
                currency: "USD".to_string(),
            }),
            "gpt-5" => Some(ModelPrice {
                input_price: 15.0,
                output_price: 60.0,
                currency: "USD".to_string(),
            }),
            _ => None,
        }
    }

    fn provider_type(&self) -> AIProviderType {
        AIProviderType::OpenAI
    }

    fn default_model(&self) -> &str {
        "gpt-5.4"
    }
}

// OpenAI API 请求/响应结构
#[derive(Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageContent,
}

#[derive(Deserialize)]
struct OpenAIMessageContent {
    content: String,
}

#[derive(Deserialize)]
struct OpenAIStreamEvent {
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIDelta,
}

#[derive(Deserialize)]
struct OpenAIDelta {
    content: String,
}