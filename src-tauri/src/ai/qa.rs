//! 单篇文献智能问答模块
//!
//! 本模块实现针对单篇 PDF 文献的智能问答功能：
//! - 提取 PDF 全文文本
//! - 构建上下文提示词
//! - 调用 AI 回答用户关于论文的问题
//! - 支持直接使用摘要作为上下文（当 PDF 不可用时）

use crate::ai::models::Message;
use crate::ai::traits::AIProvider;
use crate::db::connection::get_connection;
use crate::db::metadata::get_cached_metadata;
use crate::db::dynamic::{DynamicSqlBuilder, ZoteroTableCandidates};
use crate::pdf::text_extract::extract_text_from_pdf;
use std::path::PathBuf;

/// 最大上下文字符数（避免超出 token 限制）
const MAX_CONTEXT_CHARS: usize = 10000;

/// 单篇文献问答请求
pub struct PaperQARequest {
    /// 文献 ID
    pub item_id: i32,
    /// PDF 文件路径（可选）
    pub pdf_path: Option<String>,
    /// 用户问题
    pub question: String,
}

/// 单篇文献问答结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PaperQAResult {
    /// 回答内容
    pub answer: String,
    /// 使用的上下文类型（pdf 或 abstract）
    pub context_type: String,
    /// 上下文长度（字符数）
    pub context_length: usize,
}

/// 从数据库获取文献摘要
fn get_item_abstract(item_id: i32) -> Result<String, QAError> {
    let guard = get_connection().map_err(|e| QAError::DatabaseError(e.to_string()))?;
    let conn = guard.as_ref().ok_or_else(|| QAError::DatabaseError("数据库连接未初始化".to_string()))?;

    // 获取动态元数据
    let metadata = get_cached_metadata(conn)
        .map_err(|e| QAError::DatabaseError(format!("获取元数据失败: {}", e)))?;
    let dynamic = DynamicSqlBuilder::new(&metadata);

    // 动态获取表名
    let items_table = dynamic.find_table(ZoteroTableCandidates::ITEMS)
        .ok_or_else(|| QAError::DatabaseError("未找到 items 表".to_string()))?;
    let item_data_table = dynamic.find_table(ZoteroTableCandidates::ITEM_DATA)
        .ok_or_else(|| QAError::DatabaseError("未找到 itemData 表".to_string()))?;
    let item_data_values_table = dynamic.find_table(ZoteroTableCandidates::ITEM_DATA_VALUES)
        .ok_or_else(|| QAError::DatabaseError("未找到 itemDataValues 表".to_string()))?;
    let fields_table = dynamic.find_table(ZoteroTableCandidates::FIELDS)
        .ok_or_else(|| QAError::DatabaseError("未找到 fields 表".to_string()))?;

    // 查询文献标题和摘要
    let sql = format!(
        r#"
        SELECT
            fv_title.value as title,
            fv_abstract.value as abstract_text
        FROM {items_table} i
        LEFT JOIN {item_data_table} id_title ON i.itemID = id_title.itemID
            AND id_title.fieldID = (SELECT fieldID FROM {fields_table} WHERE fieldName = 'title')
        LEFT JOIN {item_data_values_table} fv_title ON id_title.valueID = fv_title.valueID
        LEFT JOIN {item_data_table} id_abstract ON i.itemID = id_abstract.itemID
            AND id_abstract.fieldID = (SELECT fieldID FROM {fields_table} WHERE fieldName = 'abstractNote')
        LEFT JOIN {item_data_values_table} fv_abstract ON id_abstract.valueID = fv_abstract.valueID
        WHERE i.itemID = ?
        "#,
        items_table = items_table,
        item_data_table = item_data_table,
        item_data_values_table = item_data_values_table,
        fields_table = fields_table
    );

    let result = conn
        .query_row(&sql, [item_id], |row| {
            Ok((
                row.get::<_, String>(0).unwrap_or_default(),
                row.get::<_, Option<String>>(1).ok().flatten(),
            ))
        })
        .map_err(|e| QAError::DatabaseError(format!("查询摘要失败: {}", e)))?;

    let title = result.0;
    let abstract_text = result.1.unwrap_or_default();

    // 构建摘要上下文
    if abstract_text.is_empty() {
        Err(QAError::ContextNotFound("文献没有摘要信息且无法访问PDF".to_string()))
    } else {
        Ok(format!("标题: {}\n\n摘要: {}", title, abstract_text))
    }
}

/// 单篇文献问答
///
/// # 参数
/// * `provider` - AI Provider
/// * `request` - 问答请求
///
/// # 返回值
/// * `Result<PaperQAResult, QAError>` - 问答结果
pub fn answer_paper_question(
    provider: &dyn AIProvider,
    request: &PaperQARequest,
) -> Result<PaperQAResult, QAError> {
    eprintln!("[问答] 开始处理问答请求: item_id={}", request.item_id);

    // 1. 提取上下文（PDF 全文或摘要）
    let (context, context_type) = if let Some(path) = &request.pdf_path {
        let pdf_path = PathBuf::from(path);
        match extract_text_from_pdf(&pdf_path) {
            Ok(text) => (text, "pdf".to_string()),
            Err(e) => {
                eprintln!("[问答] PDF 提取失败，尝试使用摘要: {}", e);
                (get_item_abstract(request.item_id)?, "abstract".to_string())
            }
        }
    } else {
        (get_item_abstract(request.item_id)?, "abstract".to_string())
    };

    let context_length = context.len();

    // 2. 截取前 MAX_CONTEXT_CHARS 字符（避免超出 token 限制）
    let truncated_context = if context.len() > MAX_CONTEXT_CHARS {
        // 找到最后一个完整句子的位置
        let end_idx = &context[..MAX_CONTEXT_CHARS]
            .rfind(|c| c == '。' || c == '.' || c == '!' || c == '?')
            .map(|i| i + 1)
            .unwrap_or(MAX_CONTEXT_CHARS);
        format!("{}...(内容已截断)", &context[..*end_idx])
    } else {
        context
    };

    // 3. 构建提示词
    let system_msg = "你是一个专业的学术论文阅读助手。用户会给你一篇论文的全文或摘要，以及一个关于这篇论文的问题。请仔细阅读论文内容，准确回答用户的问题。如果论文中没有涉及相关内容，请如实说明。回答应该简洁、有条理，最好引用论文中的具体内容来支持你的回答。";

    let user_msg = format!(
        "论文内容：\n{}\n\n用户问题：{}\n\n请根据论文内容回答问题。",
        truncated_context, request.question
    );

    // 4. 调用 AI
    let messages = vec![
        Message {
            role: crate::ai::models::MessageRole::System,
            content: system_msg.to_string(),
        },
        Message {
            role: crate::ai::models::MessageRole::User,
            content: user_msg,
        },
    ];

    let answer = provider
        .chat_completion(messages)
        .map_err(|e| QAError::AIError(e.to_string()))?;

    eprintln!(
        "[问答] 问答完成: item_id={}, context_type={}, context_length={}",
        request.item_id, context_type, context_length
    );

    Ok(PaperQAResult {
        answer,
        context_type,
        context_length,
    })
}

/// 问答错误类型
#[derive(Debug)]
pub enum QAError {
    /// AI 调用错误
    AIError(String),
    /// 数据库错误
    DatabaseError(String),
    /// PDF 提取错误
    PdfError(String),
    /// 上下文不存在
    ContextNotFound(String),
}

impl std::fmt::Display for QAError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QAError::AIError(msg) => write!(f, "AI 调用失败: {}", msg),
            QAError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            QAError::PdfError(msg) => write!(f, "PDF 提取失败: {}", msg),
            QAError::ContextNotFound(msg) => write!(f, "上下文不存在: {}", msg),
        }
    }
}

impl std::error::Error for QAError {}

impl From<crate::pdf::text_extract::PdfTextError> for QAError {
    fn from(err: crate::pdf::text_extract::PdfTextError) -> Self {
        QAError::PdfError(err.to_string())
    }
}