//! AI 通用 HTTP 客户端
//!
//! 基于 reqwest 实现通用的异步 HTTP 客户端

use reqwest::Client;
use std::time::Duration;

/// HTTP 客户端工厂
pub struct HTTPClientFactory;

impl HTTPClientFactory {
    /// 创建默认 HTTP 客户端
    pub fn create() -> Client {
        Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("创建 HTTP 客户端失败")
    }

    /// 创建带自定义超时的 HTTP 客户端
    pub fn create_with_timeout(timeout: Duration) -> Client {
        Client::builder()
            .timeout(timeout)
            .build()
            .expect("创建 HTTP 客户端失败")
    }
}