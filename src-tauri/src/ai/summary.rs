//! 文献智能摘要生成模块
//!
//! 本模块实现文献摘要的自动生成功能，包括：
//! - 自动整合 Zotero 元数据（作者、期刊、发表年份、引用次数）
//! - 自动提取用户在该文献中的所有高亮和笔记
//! - 生成结构化摘要：核心问题、研究方法、关键结论、创新点、局限性
//! - 摘要结果缓存到 Zotero 数据库的 extra 字段
//! - 支持重新生成摘要
//! - 支持流式输出

use crate::ai::models::{Message, Stream};
use crate::ai::traits::AIProvider;
use crate::db::connection::get_connection;
use crate::db::metadata::get_cached_metadata;
use crate::db::dynamic::{DynamicSqlBuilder, ZoteroTableCandidates};
use crate::pdf::storage::AnnotationStorage;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 摘要缓存前缀（用于存储在 extra 字段）
const SUMMARY_CACHE_PREFIX: &str = "ZoPlus_Summary::";

/// 摘要数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleSummary {
    /// 文献ID
    pub item_id: i32,
    /// 标题
    pub title: String,
    /// 作者
    pub authors: String,
    /// 年份
    pub year: String,
    /// 核心问题
    pub core_problem: String,
    /// 研究方法
    pub research_methods: String,
    /// 关键结论
    pub key_conclusions: String,
    /// 创新点
    pub innovation: String,
    /// 局限性
    pub limitations: String,
    /// 关键词
    pub keywords: Vec<String>,
    /// 生成时间
    pub generated_at: i64,
    /// 来源标注（学术规范引用格式）
    pub citation: String,
    /// 用户标注重点内容
    pub highlighted_content: Vec<String>,
    /// 摘要版本
    pub version: u32,
}

impl ArticleSummary {
    /// 创建新的摘要
    pub fn new(
        item_id: i32,
        title: String,
        authors: String,
        year: String,
        core_problem: String,
        research_methods: String,
        key_conclusions: String,
        innovation: String,
        limitations: String,
        keywords: Vec<String>,
        citation: String,
        highlighted_content: Vec<String>,
    ) -> Self {
        Self {
            item_id,
            title,
            authors,
            year,
            core_problem,
            research_methods,
            key_conclusions,
            innovation,
            limitations,
            keywords,
            generated_at: chrono_timestamp(),
            citation,
            highlighted_content,
            version: 1,
        }
    }

    /// 导出为 Markdown 格式
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", self.title));
        md.push_str(&format!("**作者**: {} | **年份**: {}\n\n", self.authors, self.year));
        md.push_str(&format!("**引用格式**: {}\n\n", self.citation));
        md.push_str("---\n\n");

        if !self.keywords.is_empty() {
            md.push_str(&format!("**关键词**: {}\n\n", self.keywords.join("、")));
        }

        md.push_str("## 核心问题\n\n");
        md.push_str(&self.core_problem);
        md.push_str("\n\n## 研究方法\n\n");
        md.push_str(&self.research_methods);
        md.push_str("\n\n## 关键结论\n\n");
        md.push_str(&self.key_conclusions);
        md.push_str("\n\n## 创新点\n\n");
        md.push_str(&self.innovation);
        md.push_str("\n\n## 局限性\n\n");
        md.push_str(&self.limitations);

        if !self.highlighted_content.is_empty() {
            md.push_str("\n\n## 用户标注重点\n\n");
            for (i, content) in self.highlighted_content.iter().enumerate() {
                md.push_str(&format!("{}. {}\n\n", i + 1, content));
            }
        }

        md.push_str("\n---\n\n");
        md.push_str(&format!("*摘要生成时间: {} | 版本: {}*\n",
            chrono_datetime_string(self.generated_at), self.version));

        md
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// 从 JSON 字符串反序列化
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// 获取当前时间戳（毫秒）
fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// 时间戳转可读日期字符串
fn chrono_datetime_string(timestamp: i64) -> String {
    let secs = timestamp / 1000;
    // 使用 SystemTime 将时间戳转换为日期时间字符串
    std::time::UNIX_EPOCH
        .checked_add(std::time::Duration::from_secs(secs as u64))
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| {
            let days_since_epoch = d.as_secs() / 86400;
            let year = (days_since_epoch / 365) + 1970;
            let remaining = days_since_epoch % 365;
            let month = remaining / 30 + 1;
            let day = remaining % 30 + 1;
            format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(31))
        })
        .unwrap_or_else(|| "未知时间".to_string())
}

/// 摘要生成器
pub struct SummaryGenerator {
    /// AI Provider
    provider: Arc<dyn AIProvider>,
    /// PDF 标注存储（用于获取用户高亮）
    annotation_storage: Option<AnnotationStorage>,
}

impl SummaryGenerator {
    /// 创建新的摘要生成器
    pub fn new(provider: Arc<dyn AIProvider>) -> Self {
        Self {
            provider,
            annotation_storage: None,
        }
    }

    /// 创建带标注存储的摘要生成器
    pub fn with_annotation_storage(provider: Arc<dyn AIProvider>, storage: AnnotationStorage) -> Self {
        Self {
            provider,
            annotation_storage: Some(storage),
        }
    }

    /// 从 Zotero 数据库读取文献元数据
    fn fetch_metadata(&self, item_id: i32) -> Result<ItemMetadata, SummaryError> {
        let guard = get_connection().map_err(|e| SummaryError::DatabaseError(e.to_string()))?;
        let conn = guard.as_ref().ok_or_else(|| SummaryError::DatabaseError("数据库连接未初始化".to_string()))?;

        // 获取动态元数据
        let metadata = get_cached_metadata(conn)
            .map_err(|e| SummaryError::DatabaseError(format!("获取元数据失败: {}", e)))?;
        let dynamic = DynamicSqlBuilder::new(&metadata);

        // 动态获取表名
        let items_table = dynamic.find_table(ZoteroTableCandidates::ITEMS)
            .ok_or_else(|| SummaryError::DatabaseError("未找到 items 表".to_string()))?;
        let item_data_table = dynamic.find_table(ZoteroTableCandidates::ITEM_DATA)
            .ok_or_else(|| SummaryError::DatabaseError("未找到 itemData 表".to_string()))?;
        let item_data_values_table = dynamic.find_table(ZoteroTableCandidates::ITEM_DATA_VALUES)
            .ok_or_else(|| SummaryError::DatabaseError("未找到 itemDataValues 表".to_string()))?;
        let fields_table = dynamic.find_table(ZoteroTableCandidates::FIELDS)
            .ok_or_else(|| SummaryError::DatabaseError("未找到 fields 表".to_string()))?;
        let authors_table = dynamic.find_table(ZoteroTableCandidates::CREATORS)
            .ok_or_else(|| SummaryError::DatabaseError("未找到 itemCreators 表".to_string()))?;
        let creators_table = dynamic.find_table(ZoteroTableCandidates::CREATOR)
            .ok_or_else(|| SummaryError::DatabaseError("未找到 creators 表".to_string()))?;

        // 动态构建 SQL
        let sql = format!(
            r#"
            SELECT
                i.itemID as item_id,
                fv_title.value as title,
                fv_date.value as year,
                (
                    SELECT GROUP_CONCAT(
                        COALESCE(c.lastName, '') || COALESCE(c.firstName, ''),
                        '; '
                    )
                    FROM {authors_table} ia
                    JOIN {creators_table} c ON ia.creatorID = c.creatorID
                    WHERE ia.itemID = i.itemID
                    ORDER BY ia.orderIndex
                ) as authors
            FROM {items_table} i
            LEFT JOIN {item_data_table} id_title ON i.itemID = id_title.itemID
                AND id_title.fieldID = (SELECT fieldID FROM {fields_table} WHERE fieldName = 'title')
            LEFT JOIN {item_data_values_table} fv_title ON id_title.valueID = fv_title.valueID
            LEFT JOIN {item_data_table} id_date ON i.itemID = id_date.itemID
                AND id_date.fieldID = (SELECT fieldID FROM {fields_table} WHERE fieldName = 'date')
            WHERE i.itemID = ?
            "#,
            items_table = items_table,
            item_data_table = item_data_table,
            item_data_values_table = item_data_values_table,
            fields_table = fields_table,
            authors_table = authors_table,
            creators_table = creators_table
        );

        let metadata = conn
            .query_row(&sql, params![item_id], |row| {
                Ok(ItemMetadata {
                    item_id: row.get(0)?,
                    title: row.get::<_, String>(1).unwrap_or_default(),
                    year: row.get::<_, String>(2).unwrap_or_default(),
                    authors: row.get::<_, String>(3).unwrap_or_default(),
                })
            })
            .map_err(|e| SummaryError::DatabaseError(format!("查询元数据失败: {}", e)))?;

        Ok(metadata)
    }

    /// 获取用户在高亮和笔记中的文本内容
    fn fetch_user_highlights(&self, pdf_key: &str) -> Vec<String> {
        if let Some(storage) = &self.annotation_storage {
            if let Ok(annotations) = storage.get_annotations(pdf_key) {
                return annotations
                    .iter()
                    .filter(|a| {
                        use crate::pdf::annotations::AnnotationType;
                        matches!(a.annotation_type, AnnotationType::Highlight) ||
                        matches!(a.annotation_type, AnnotationType::TextNote)
                    })
                    .filter_map(|a| {
                        use crate::pdf::annotations::AnnotationData;
                        match &a.data {
                            AnnotationData::Highlight(h) => Some(h.text.clone()),
                            AnnotationData::TextNote(t) => Some(t.content.clone()),
                            _ => None,
                        }
                    })
                    .collect();
            }
        }
        Vec::new()
    }

    /// 从 extra 字段读取缓存的摘要
    fn read_cached_summary(&self, item_id: i32) -> Option<ArticleSummary> {
        let guard = match get_connection() {
            Ok(g) => g,
            Err(_) => return None,
        };
        let conn = guard.as_ref()?;

        // 获取动态元数据
        let metadata = match get_cached_metadata(conn) {
            Ok(m) => m,
            Err(_) => return None,
        };
        let dynamic = DynamicSqlBuilder::new(&metadata);

        // 动态获取表名
        let item_data_table = dynamic.find_table(ZoteroTableCandidates::ITEM_DATA)?;
        let item_data_values_table = dynamic.find_table(ZoteroTableCandidates::ITEM_DATA_VALUES)?;
        let fields_table = dynamic.find_table(ZoteroTableCandidates::FIELDS)?;

        // 动态构建 SQL
        let sql = format!(
            r#"
            SELECT fv_extra.value
            FROM {item_data_table} id_extra
            JOIN {item_data_values_table} fv_extra ON id_extra.valueID = fv_extra.valueID
            WHERE id_extra.itemID = ?
            AND id_extra.fieldID = (SELECT fieldID FROM {fields_table} WHERE fieldName = 'extra')
            "#,
            item_data_table = item_data_table,
            item_data_values_table = item_data_values_table,
            fields_table = fields_table
        );

        let extra_content: Option<String> = conn
            .query_row(&sql, params![item_id], |row| row.get(0))
            .ok();

        if let Some(extra) = extra_content {
            // 在 extra 中查找 ZoPlus 摘要缓存
            if let Some(start) = extra.find(SUMMARY_CACHE_PREFIX) {
                let json_start = start + SUMMARY_CACHE_PREFIX.len();
                if let Some(end) = extra.find("::EndSummary") {
                    let json_str = &extra[json_start..end];
                    if let Ok(summary) = ArticleSummary::from_json(json_str) {
                        return Some(summary);
                    }
                }
            }
        }

        None
    }

    /// 保存摘要到 extra 字段
    fn save_summary_to_extra(&self, item_id: i32, summary: &ArticleSummary) -> Result<(), SummaryError> {
        let guard = get_connection().map_err(|e| SummaryError::DatabaseError(e.to_string()))?;
        let conn = guard.as_ref().ok_or_else(|| SummaryError::DatabaseError("数据库连接未初始化".to_string()))?;

        // 获取动态元数据
        let metadata = get_cached_metadata(conn)
            .map_err(|e| SummaryError::DatabaseError(format!("获取元数据失败: {}", e)))?;
        let dynamic = DynamicSqlBuilder::new(&metadata);

        // 动态获取表名
        let item_data_table = dynamic.find_table(ZoteroTableCandidates::ITEM_DATA)
            .ok_or_else(|| SummaryError::DatabaseError("未找到 itemData 表".to_string()))?;
        let item_data_values_table = dynamic.find_table(ZoteroTableCandidates::ITEM_DATA_VALUES)
            .ok_or_else(|| SummaryError::DatabaseError("未找到 itemDataValues 表".to_string()))?;
        let fields_table = dynamic.find_table(ZoteroTableCandidates::FIELDS)
            .ok_or_else(|| SummaryError::DatabaseError("未找到 fields 表".to_string()))?;

        // 序列化摘要
        let json = summary.to_json().map_err(|e| SummaryError::SerializeError(e.to_string()))?;
        let cache_entry = format!("{}{}::EndSummary", SUMMARY_CACHE_PREFIX, json);

        // 获取 extra 字段的当前值
        let select_sql = format!(
            r#"
            SELECT fv_extra.value
            FROM {item_data_table} id_extra
            JOIN {item_data_values_table} fv_extra ON id_extra.valueID = fv_extra.valueID
            WHERE id_extra.itemID = ?
            AND id_extra.fieldID = (SELECT fieldID FROM {fields_table} WHERE fieldName = 'extra')
            "#,
            item_data_table = item_data_table,
            item_data_values_table = item_data_values_table,
            fields_table = fields_table
        );
        let current_extra: Option<String> = conn
            .query_row(&select_sql, params![item_id], |row| row.get(0))
            .ok();

        // 构建新的 extra 值（保留原有内容，但移除旧的摘要缓存）
        let new_extra = if let Some(existing) = current_extra {
            // 移除旧的摘要缓存
            let cleaned = if let Some(start) = existing.find(SUMMARY_CACHE_PREFIX) {
                let before = &existing[..start];
                let after = existing[start..].find("::EndSummary")
                    .map(|end| &existing[start + end + 12..])
                    .unwrap_or("");
                format!("{}{}", before.trim(), after.trim())
            } else {
                existing
            };
            format!("{}\n{}", cleaned.trim(), cache_entry)
        } else {
            cache_entry
        };

        // 更新 extra 字段（由于是只读模式，我们需要使用事务）
        // 注意：这里我们假设 extra 字段已经存在记录
        let update_sql = format!(
            r#"
            UPDATE {item_data_table}
            SET valueID = (
                SELECT v.valueID
                FROM {item_data_values_table} v
                WHERE v.value = ?
                LIMIT 1
            )
            WHERE itemID = ?
            AND fieldID = (SELECT fieldID FROM {fields_table} WHERE fieldName = 'extra')
            "#,
            item_data_table = item_data_table,
            item_data_values_table = item_data_values_table,
            fields_table = fields_table
        );

        // 如果 extra 字段不存在记录，尝试插入
        let result = conn.execute(&update_sql, params![new_extra, item_id]);
        if result.is_err() {
            // 尝试插入新记录
            let insert_sql = format!(
                r#"
                INSERT INTO {item_data_table} (itemID, fieldID, valueID)
                SELECT ?, fieldID, v.valueID
                FROM {item_data_values_table} v
                WHERE v.value = ?
                LIMIT 1
                "#,
                item_data_table = item_data_table,
                item_data_values_table = item_data_values_table
            );
            conn.execute(&insert_sql, params![item_id, new_extra])
                .map_err(|e| SummaryError::DatabaseError(format!("保存摘要失败: {}", e)))?;
        }

        eprintln!("[摘要] 摘要已保存到 extra 字段: item_id={}", item_id);
        Ok(())
    }

    /// 生成摘要
    pub async fn generate_summary(
        &self,
        item_id: i32,
        pdf_key: Option<&str>,
    ) -> Result<ArticleSummary, SummaryError> {
        eprintln!("[摘要] 开始生成摘要: item_id={}", item_id);

        // 获取元数据
        let metadata = self.fetch_metadata(item_id)?;

        // 获取用户高亮内容
        let highlights = if let Some(key) = pdf_key {
            self.fetch_user_highlights(key)
        } else {
            Vec::new()
        };

        // 构建提示词
        let prompt = self.build_summary_prompt(&metadata, &highlights);

        // 调用 AI 生成摘要
        let messages = vec![
            Message::system("你是一个专业的学术论文摘要生成助手。请根据提供的论文信息和用户标注内容，生成结构化的学术摘要。"),
            Message::user(prompt),
        ];

        let response = self.provider.chat_completion(messages)
            .map_err(|e| SummaryError::AIError(e.to_string()))?;

        // 解析 AI 响应
        let summary = self.parse_summary_response(&metadata, &response, &highlights)?;

        // 保存到 extra 字段
        self.save_summary_to_extra(item_id, &summary)?;

        eprintln!("[摘要] 摘要生成完成: item_id={}", item_id);
        Ok(summary)
    }

    /// 流式生成摘要
    pub async fn generate_summary_streaming(
        &self,
        item_id: i32,
        pdf_key: Option<&str>,
    ) -> Result<Stream<String>, SummaryError> {
        eprintln!("[摘要] 开始流式生成摘要: item_id={}", item_id);

        // 获取元数据
        let metadata = self.fetch_metadata(item_id)?;

        // 获取用户高亮内容
        let highlights = if let Some(key) = pdf_key {
            self.fetch_user_highlights(key)
        } else {
            Vec::new()
        };

        // 构建提示词
        let prompt = self.build_summary_prompt(&metadata, &highlights);

        // 调用 AI 流式生成
        let messages = vec![
            Message::system("你是一个专业的学术论文摘要生成助手。请根据提供的论文信息和用户标注内容，生成结构化的学术摘要。"),
            Message::user(prompt),
        ];

        let stream = self.provider.stream_chat_completion(messages)
            .map_err(|e| SummaryError::AIError(e.to_string()))?;

        Ok(stream)
    }

    /// 构建摘要生成提示词
    fn build_summary_prompt(&self, metadata: &ItemMetadata, highlights: &[String]) -> String {
        let mut prompt = format!(
            "请为以下学术论文生成结构化摘要：\n\n\
            标题：{}\n\
            作者：{}\n\
            年份：{}\n\n",
            metadata.title, metadata.authors, metadata.year
        );

        if !highlights.is_empty() {
            prompt.push_str("用户在该论文中的标注内容：\n");
            for (i, h) in highlights.iter().enumerate().take(10) {
                prompt.push_str(&format!("{}. {}\n", i + 1, h));
            }
            prompt.push_str("\n");
        }

        prompt.push_str(
            "请按以下结构生成摘要（使用 Markdown 格式）：\n\n\
            ## 核心问题\n\
            （简明扼要地描述论文研究的核心问题）\n\n\
            ## 研究方法\n\
            （描述论文采用的研究方法和技术路线）\n\n\
            ## 关键结论\n\
            （总结论文的主要发现和结论）\n\n\
            ## 创新点\n\
            （指出论文的创新之处）\n\n\
            ## 局限性\n\
            （客观分析论文的不足和局限性）\n\n\
            ## 关键词\n\
            （列出3-5个关键词，用逗号分隔）\n\n\
            ## 学术引用格式\n\
            （提供符合学术规范的引用格式，如：作者 (年份). 标题. 期刊名, 卷(期), 页码）"
        );

        prompt
    }

    /// 解析 AI 响应，生成摘要结构体
    fn parse_summary_response(
        &self,
        metadata: &ItemMetadata,
        response: &str,
        highlights: &[String],
    ) -> Result<ArticleSummary, SummaryError> {
        // 简单解析：从 Markdown 中提取各部分内容
        // 注意：实际项目中可以使用更复杂的解析逻辑
        let sections = parse_markdown_sections(response);

        let core_problem = sections.get("核心问题").cloned().unwrap_or_default();
        let research_methods = sections.get("研究方法").cloned().unwrap_or_default();
        let key_conclusions = sections.get("关键结论").cloned().unwrap_or_default();
        let innovation = sections.get("创新点").cloned().unwrap_or_default();
        let limitations = sections.get("局限性").cloned().unwrap_or_default();
        let keywords_str = sections.get("关键词").cloned().unwrap_or_default();
        let citation = sections.get("学术引用格式").cloned().unwrap_or_else(|| {
            format!("{}. ({}). {}.", metadata.authors, metadata.year, metadata.title)
        });

        let keywords: Vec<String> = keywords_str
            .split(|c| c == ',' || c == '、' || c == '；')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .take(5)
            .collect();

        Ok(ArticleSummary::new(
            metadata.item_id,
            metadata.title.clone(),
            metadata.authors.clone(),
            metadata.year.clone(),
            core_problem,
            research_methods,
            key_conclusions,
            innovation,
            limitations,
            keywords,
            citation,
            highlights.iter().take(5).cloned().collect(),
        ))
    }

    /// 检查是否有缓存的摘要
    pub fn has_cached_summary(&self, item_id: i32) -> bool {
        self.read_cached_summary(item_id).is_some()
    }

    /// 获取缓存的摘要
    pub fn get_cached_summary(&self, item_id: i32) -> Option<ArticleSummary> {
        self.read_cached_summary(item_id)
    }
}

/// 文献元数据
struct ItemMetadata {
    item_id: i32,
    title: String,
    authors: String,
    year: String,
}

/// 摘要错误类型
#[derive(Debug)]
pub enum SummaryError {
    /// AI 调用错误
    AIError(String),
    /// 数据库错误
    DatabaseError(String),
    /// 解析错误
    ParseError(String),
    /// 序列化错误
    SerializeError(String),
    /// 缓存不存在
    CacheNotFound,
}

impl std::fmt::Display for SummaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SummaryError::AIError(msg) => write!(f, "AI 调用失败: {}", msg),
            SummaryError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            SummaryError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            SummaryError::SerializeError(msg) => write!(f, "序列化错误: {}", msg),
            SummaryError::CacheNotFound => write!(f, "缓存不存在"),
        }
    }
}

impl std::error::Error for SummaryError {}

/// 解析 Markdown 内容，提取各章节
fn parse_markdown_sections(content: &str) -> std::collections::HashMap<String, String> {
    let mut sections = std::collections::HashMap::new();
    let mut current_section = String::new();
    let mut current_title = String::new();
    let mut in_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // 检查是否是二级标题
        if trimmed.starts_with("## ") {
            // 保存上一个 section
            if in_section && !current_title.is_empty() {
                sections.insert(current_title.clone(), current_section.trim().to_string());
            }

            // 开始新的 section
            current_title = trimmed.trim_start_matches("## ").to_string();
            current_section.clear();
            in_section = true;
        } else if in_section {
            current_section.push_str(trimmed);
            current_section.push('\n');
        }
    }

    // 保存最后一个 section
    if in_section && !current_title.is_empty() {
        sections.insert(current_title, current_section.trim().to_string());
    }

    sections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_sections() {
        let content = r#"
## 核心问题
本文研究了机器学习在自然语言处理中的应用。

## 研究方法
采用了深度学习的方法。

## 关键结论
取得了显著的效果提升。
"#;

        let sections = parse_markdown_sections(content);
        assert!(sections.contains_key("核心问题"));
        assert!(sections.contains_key("研究方法"));
        assert!(sections.contains_key("关键结论"));
    }
}