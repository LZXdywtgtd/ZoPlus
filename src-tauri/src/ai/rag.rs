//!跨文献智能问答模块（RAG - Retrieval Augmented Generation）
//!
//! 本模块实现基于检索增强生成的跨文献问答功能：
//! - 自动检索最相关的 N 篇文献（结合 Tantivy 搜索）
//! - 提取上下文回答问题
//! - 回答中自动标注引用来源（文献标题+作者+年份）
//! - 支持追问功能，保持对话上下文
//! - 支持流式输出
//!
//! # 约束条件
//! - 所有检索和处理在本地完成，仅发送问题和检索到的文本片段给 AI
//! - 不发送完整 PDF 文件
//! - 聊天历史不持久化，关闭窗口自动清除
//! - 支持取消正在进行的生成

use crate::ai::models::{Message, Stream};
use crate::ai::traits::AIProvider;
use crate::search::SearchState;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// 默认检索文献数量
const DEFAULT_TOP_K: usize = 5;

/// 最大检索文献数量
pub const MAX_TOP_K: usize = 20;

/// RAG 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    /// 检索文献数量
    pub top_k: usize,
    /// 是否启用流式输出
    pub streaming: bool,
    /// 最小相关度阈值（0.0-1.0）
    pub min_score: f32,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            top_k: DEFAULT_TOP_K,
            streaming: true,
            min_score: 0.0,
        }
    }
}

/// 文献上下文片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContext {
    /// 文献ID
    pub item_id: i32,
    /// 标题
    pub title: String,
    /// 作者
    pub authors: String,
    /// 年份
    pub year: String,
    /// 摘要
    pub abstract_text: String,
    /// 关键词
    pub keywords: String,
    /// 相关度得分
    pub score: f32,
    /// 引用标识（用于回答中标注）
    pub citation_key: String,
}

impl DocumentContext {
    /// 创建引用标识
    fn create_citation_key(&self) -> String {
        let first_author = self.authors
            .split(|c| c == ';' || c == ',')
            .next()
            .unwrap_or("Unknown")
            .trim();
        format!("[{} {}]", first_author, self.year)
    }
}

/// 聊天消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 消息 ID
    pub id: String,
    /// 角色（user/assistant）
    pub role: String,
    /// 消息内容
    pub content: String,
    /// 关联的文献上下文
    pub citations: Vec<DocumentContext>,
    /// 时间戳
    pub timestamp: i64,
}

impl ChatMessage {
    /// 创建用户消息
    pub fn user(content: String) -> Self {
        Self {
            id: uuid_v4(),
            role: "user".to_string(),
            content,
            citations: Vec::new(),
            timestamp: chrono_timestamp(),
        }
    }

    /// 创建助手消息
    pub fn assistant(content: String, citations: Vec<DocumentContext>) -> Self {
        Self {
            id: uuid_v4(),
            role: "assistant".to_string(),
            content,
            citations,
            timestamp: chrono_timestamp(),
        }
    }
}

/// 对话会话（不持久化，内存存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    /// 会话 ID
    pub id: String,
    /// 对话历史
    pub messages: Vec<ChatMessage>,
    /// 关联的文献上下文（用于追问）
    pub context: Vec<DocumentContext>,
}

impl ChatSession {
    /// 创建新的对话会话
    pub fn new() -> Self {
        Self {
            id: uuid_v4(),
            messages: Vec::new(),
            context: Vec::new(),
        }
    }

    /// 添加用户消息
    pub fn add_user_message(&mut self, content: String) -> &ChatMessage {
        let msg = ChatMessage::user(content);
        self.messages.push(msg);
        self.messages.last().unwrap()
    }

    /// 添加助手消息
    pub fn add_assistant_message(&mut self, content: String, citations: Vec<DocumentContext>) -> &ChatMessage {
        let msg = ChatMessage::assistant(content, citations.clone());
        self.messages.push(msg);
        // 更新上下文（保留所有引用的文献）
        for citation in citations {
            if !self.context.iter().any(|c| c.item_id == citation.item_id) {
                self.context.push(citation);
            }
        }
        self.messages.last().unwrap()
    }

    /// 获取对话历史（用于 AI 上下文）
    pub fn get_history_for_ai(&self) -> Vec<Message> {
        let mut msgs = Vec::new();
        for msg in &self.messages {
            let role = match msg.role.as_str() {
                "user" => crate::ai::models::MessageRole::User,
                "assistant" => crate::ai::models::MessageRole::Assistant,
                _ => crate::ai::models::MessageRole::User,
            };
            msgs.push(Message {
                role,
                content: msg.content.clone(),
            });
        }
        msgs
    }
}

/// RAG 引擎
pub struct RagEngine<'a> {
    /// AI Provider
    provider: Arc<dyn AIProvider>,
    /// 搜索状态引用
    search_state: &'a SearchState,
    /// 当前会话
    session: Mutex<ChatSession>,
    /// 配置
    config: Mutex<RagConfig>,
}

impl<'a> RagEngine<'a> {
    /// 创建新的 RAG 引擎
    pub fn new(provider: Arc<dyn AIProvider>, search_state: &'a SearchState) -> Self {
        Self {
            provider,
            search_state,
            session: Mutex::new(ChatSession::new()),
            config: Mutex::new(RagConfig::default()),
        }
    }

    /// 创建带配置的 RAG 引擎
    pub fn with_config(provider: Arc<dyn AIProvider>, search_state: &'a SearchState, config: RagConfig) -> Self {
        Self {
            provider,
            search_state,
            session: Mutex::new(ChatSession::new()),
            config: Mutex::new(config),
        }
    }

    /// 检索相关文献
    fn retrieve_documents(&self, query: &str) -> Result<Vec<DocumentContext>, RagError> {
        let config = self.config.lock().unwrap();
        let top_k = config.top_k;

        eprintln!("[RAG] 开始检索文献: query={}, top_k={}", query, top_k);

        // 使用搜索引擎检索相关文献
        let engine_guard = self.search_state.get_engine()
            .map_err(|e| RagError::SearchError(e.to_string()))?;
        let engine = engine_guard.as_ref()
            .ok_or_else(|| RagError::SearchError("搜索引擎未初始化".to_string()))?;

        // 执行搜索
        let params = crate::search::SearchParams {
            query: query.to_string(),
            offset: 0,
            limit: top_k,
            fuzzy: true,
            fuzzy_distance: 2,
        };

        let results = engine.search(params)
            .map_err(|e| RagError::SearchError(e.to_string()))?;

        // 转换为文档上下文
        let documents: Vec<DocumentContext> = results
            .into_iter()
            .filter(|r| r.score >= config.min_score)
            .map(|r| {
                let mut ctx = DocumentContext {
                    item_id: r.item_id,
                    title: r.title,
                    authors: r.authors,
                    year: r.year,
                    abstract_text: r.abstract_text,
                    keywords: r.keywords,
                    score: r.score,
                    citation_key: String::new(),
                };
                ctx.citation_key = ctx.create_citation_key();
                ctx
            })
            .collect();

        eprintln!("[RAG] 检索到 {} 篇相关文献", documents.len());
        Ok(documents)
    }

    /// 构建检索增强的提示词
    fn build_rag_prompt(&self, query: &str, documents: &[DocumentContext]) -> String {
        let mut prompt = String::new();

        prompt.push_str("你是一个专业的学术论文阅读助手。请根据提供的文献上下文回答用户的问题。\n\n");

        if !documents.is_empty() {
            prompt.push_str("## 参考文献\n\n");
            for (i, doc) in documents.iter().enumerate() {
                let abstract_short = if doc.abstract_text.len() > 300 {
                    format!("{}...", &doc.abstract_text[..300])
                } else {
                    doc.abstract_text.clone()
                };
                prompt.push_str(&format!(
                    "{}. **{}** ({})\n   作者: {}\n   摘要: {}\n\n",
                    i + 1, doc.title, doc.year, doc.authors, abstract_short
                ));
            }
            prompt.push_str("\n---\n\n");
        }

        prompt.push_str(&format!("## 用户问题\n\n{}\n\n", query));

        prompt.push_str(
            "## 回答要求\n\n\
            1. 请基于上述参考文献回答问题，结合每篇文献的观点进行对比分析\n\
            2. 在回答中引用文献时，请使用引用标注，格式为 [作者 年份]，例如：[张三 2023]\n\
            3. 如果多篇文献涉及同一问题，请对比分析它们的不同观点和方法\n\
            4. 如果无法从提供的文献中找到答案，请明确说明\n\
            5. 回答应该结构清晰，使用 Markdown 格式\n\
            6. 回答完成后，列出你参考的所有文献\n\n\
            ## 参考文献列表\n\n"
        );

        for doc in documents {
            prompt.push_str(&format!("- {} ({}), {}\n", doc.title, doc.authors, doc.year));
        }

        prompt
    }

    /// 构建 AI 消息列表（公共逻辑，消除重复）
    fn build_ai_messages(&self, prompt: String) -> Vec<Message> {
        let system_msg = Message::system("你是一个专业的学术论文阅读助手，擅长对比分析多篇文献的异同。");
        let user_msg = Message::user(prompt);
        let session = self.session.lock().unwrap();
        let history = session.get_history_for_ai();

        let mut messages = vec![system_msg];
        messages.extend(history);
        messages.push(user_msg);
        messages
    }

    /// 处理对话（单次，非流式）
    pub async fn chat(&self, query: String) -> Result<ChatMessage, RagError> {
        eprintln!("[RAG] 处理对话: {}", query);

        // 1. 检索相关文献
        let documents = self.retrieve_documents(&query)?;
        // 2. 构建提示词并生成消息
        let prompt = self.build_rag_prompt(&query, &documents);
        let messages = self.build_ai_messages(prompt);
        // 3. 调用 AI
        let response = self.provider.chat_completion(messages)
            .map_err(|e| RagError::AIError(e.to_string()))?;
        // 4. 创建助手消息
        Ok(ChatMessage::assistant(response, documents))
    }

    /// 处理对话（流式）
    pub async fn chat_streaming(&self, query: String) -> Result<Stream<String>, RagError> {
        eprintln!("[RAG] 处理流式对话: {}", query);

        // 1. 检索相关文献
        let documents = self.retrieve_documents(&query)?;
        // 2. 构建提示词并生成消息
        let prompt = self.build_rag_prompt(&query, &documents);
        let messages = self.build_ai_messages(prompt);
        // 3. 调用 AI 流式输出
        self.provider.stream_chat_completion(messages)
            .map_err(|e| RagError::AIError(e.to_string()))
    }

    /// 保存用户消息到会话
    pub fn save_user_message(&self, content: String) -> String {
        let mut session = self.session.lock().unwrap();
        let msg = session.add_user_message(content);
        msg.id.clone()
    }

    /// 保存助手消息到会话
    pub fn save_assistant_message(&self, content: String, citations: Vec<DocumentContext>) -> String {
        let mut session = self.session.lock().unwrap();
        let msg = session.add_assistant_message(content, citations);
        msg.id.clone()
    }

    /// 获取当前会话的所有消息
    pub fn get_session_messages(&self) -> Vec<ChatMessage> {
        let session = self.session.lock().unwrap();
        session.messages.clone()
    }

    /// 获取当前会话的上下文文献
    pub fn get_session_context(&self) -> Vec<DocumentContext> {
        let session = self.session.lock().unwrap();
        session.context.clone()
    }

    /// 清除会话
    pub fn clear_session(&self) {
        let mut session = self.session.lock().unwrap();
        *session = ChatSession::new();
        eprintln!("[RAG] 会话已清除");
    }

    /// 更新配置
    pub fn update_config(&self, config: RagConfig) {
        let mut cfg = self.config.lock().unwrap();
        *cfg = config;
        eprintln!("[RAG] 配置已更新: top_k={}", cfg.top_k);
    }

    /// 获取当前配置
    pub fn get_config(&self) -> RagConfig {
        self.config.lock().unwrap().clone()
    }
}

/// RAG 错误类型
#[derive(Debug)]
pub enum RagError {
    /// AI 调用错误
    AIError(String),
    /// 搜索错误
    SearchError(String),
    /// 数据库错误
    DatabaseError(String),
    /// 配置错误
    ConfigError(String),
    /// 会话不存在
    SessionNotFound,
}

impl std::fmt::Display for RagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RagError::AIError(msg) => write!(f, "AI 调用失败: {}", msg),
            RagError::SearchError(msg) => write!(f, "搜索失败: {}", msg),
            RagError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            RagError::ConfigError(msg) => write!(f, "配置错误: {}", msg),
            RagError::SessionNotFound => write!(f, "会话不存在"),
        }
    }
}

impl std::error::Error for RagError {}

/// 获取当前时间戳（毫秒）
fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// 生成简单的 UUID v4（简化实现）
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let random: u64 = (now as u64) ^ (std::process::id() as u64 * 0x5DEADC0DE);
    format!("{:016x}-{:04x}-{:04x}-{:04x}-{:012x}",
        now as u64,
        (random >> 48) as u16 & 0xFFFF,
        (random >> 32) as u16 & 0xFFFF,
        0x4000 | (random >> 16) as u16 & 0x3FFF,
        random & 0xFFFFFFFFFFFF
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_key() {
        let ctx = DocumentContext {
            item_id: 1,
            title: "Test Paper".to_string(),
            authors: "张三;李四".to_string(),
            year: "2023".to_string(),
            abstract_text: "Test abstract".to_string(),
            keywords: "test".to_string(),
            score: 0.9,
            citation_key: String::new(),
        };
        let key = ctx.create_citation_key();
        assert!(key.contains("张三"));
        assert!(key.contains("2023"));
    }
}