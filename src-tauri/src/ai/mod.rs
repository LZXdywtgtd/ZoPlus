//! AI 接口模块
//!
//! 本模块负责调用多厂商 AI API 实现以下功能：
//! - 统一 AI Provider 接口抽象
//! - 支持 OpenAI、Anthropic、DeepSeek、豆包、通义千问、智谱GLM、MiniMax、小米MiMo
//! - AI 摘要生成
//! - 翻译功能
//! - 自动标签
//! - 文献问答
//! - 元数据补全
//!
//! # 模块结构
//! - traits.rs: AIProvider trait 定义，所有厂商必须实现此接口
//! - errors.rs: 错误类型定义
//! - models.rs: 模型信息结构体
//! - config.rs: 配置管理
//! - client.rs: 通用 HTTP 客户端
//! - providers/: 各厂商具体实现
//! - commands.rs: Tauri 命令接口

pub mod traits;
pub mod errors;
pub mod models;
pub mod config;
pub mod client;
pub mod providers;
pub mod commands;
pub mod summary;
pub mod note;
pub mod citation;
pub mod citation_commands;

pub use traits::AIProvider;
pub use errors::AIError;
pub use models::{Message, MessageRole, ModelInfo, ModelPrice, AIProviderType, Stream};
pub use config::{AIConfig, AIConfigManager, load_api_key_from_env};
pub use providers::create_provider;
pub use summary::{ArticleSummary, SummaryGenerator, SummaryError};
pub use note::{Note, NoteTemplate, NoteGenerator, NoteError};
pub use citation::{
    CitationFormat, CitationFormatter, CitationMetadata, FormattedCitation, ParsedCitation,
    Author, ItemType, FormatterConfig, FormatterLanguage,
};