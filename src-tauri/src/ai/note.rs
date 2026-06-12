//! 智能笔记生成模块
//!
//! 本模块实现基于 AI 的智能笔记生成功能，包括：
//! - 支持多种笔记模板：要点笔记、方法笔记、结论笔记、批判性笔记
//! - 笔记自动关联到原文段落和页码
//! - 笔记存储到 Zotero itemNotes 表（完全兼容 Zotero 原生格式）
//! - 支持笔记的二次编辑和标签分类
//! - 支持批量生成多个高亮段落的笔记
//! - 笔记自动添加引用信息（文献标题+页码）
//! - 支持一键导出所有笔记为 Markdown 格式

use crate::ai::models::Message;
use crate::ai::traits::AIProvider;
use crate::db::connection::get_connection;
use crate::db::metadata::get_cached_metadata;
use crate::db::dynamic::{DynamicSqlBuilder, ZoteroTableCandidates};
use crate::db::items::ItemInfo;
use crate::pdf::storage::AnnotationStorage;
use crate::pdf::annotations::{AnnotationType, AnnotationData};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 笔记模板类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NoteTemplate {
    /// 要点笔记 - 提取关键信息点
    KeyPoints,
    /// 方法笔记 - 记录研究方法和技术路线
    Methods,
    /// 结论笔记 - 总结主要发现和结论
    Conclusions,
    /// 批判性笔记 - 分析论文的优点、局限性和改进方向
    Critical,
    /// 通用笔记 - 自由格式笔记
    General,
}

impl NoteTemplate {
    /// 获取模板显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            NoteTemplate::KeyPoints => "要点笔记",
            NoteTemplate::Methods => "方法笔记",
            NoteTemplate::Conclusions => "结论笔记",
            NoteTemplate::Critical => "批判性笔记",
            NoteTemplate::General => "通用笔记",
        }
    }

    /// 获取模板描述
    pub fn description(&self) -> &'static str {
        match self {
            NoteTemplate::KeyPoints => "提取文本中的关键信息点，适合快速回顾",
            NoteTemplate::Methods => "详细记录研究方法、技术路线和实验设计",
            NoteTemplate::Conclusions => "总结主要发现、结论和研究贡献",
            NoteTemplate::Critical => "批判性分析：优点、局限性和潜在改进方向",
            NoteTemplate::General => "自由格式笔记，可根据内容灵活记录",
        }
    }
}

/// 笔记数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// 笔记ID（本地生成的 UUID）
    pub note_id: String,
    /// 关联的文献ID
    pub item_id: i32,
    /// 文献标题（冗余存储便于显示）
    pub item_title: String,
    /// 笔记标题
    pub title: String,
    /// 笔记内容（支持 Markdown）
    pub content: String,
    /// 模板类型
    pub template: NoteTemplate,
    /// 关联的原文内容（选自高亮）
    pub source_text: Option<String>,
    /// 关联的页码
    pub page: Option<u32>,
    /// 标签列表
    pub tags: Vec<String>,
    /// 创建时间（Unix 时间戳，毫秒）
    pub created_at: i64,
    /// 更新时间（Unix 时间戳，毫秒）
    pub updated_at: i64,
    /// 笔记版本
    pub version: u32,
}

impl Note {
    /// 创建新的笔记
    pub fn new(
        item_id: i32,
        item_title: String,
        title: String,
        content: String,
        template: NoteTemplate,
        source_text: Option<String>,
        page: Option<u32>,
        tags: Vec<String>,
    ) -> Self {
        let now = chrono_timestamp();
        Self {
            note_id: generate_uuid(),
            item_id,
            item_title,
            title,
            content,
            template,
            source_text,
            page,
            tags,
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    /// 更新笔记内容
    pub fn update_content(&mut self, title: String, content: String, tags: Vec<String>) {
        self.title = title;
        self.content = content;
        self.tags = tags;
        self.updated_at = chrono_timestamp();
        self.version += 1;
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = chrono_timestamp();
        }
    }

    /// 移除标签
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.updated_at = chrono_timestamp();
    }

    /// 导出为 Markdown 格式
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        // 笔记标题
        md.push_str(&format!("# {}\n\n", self.title));

        // 元信息
        md.push_str(&format!("**文献**: {}\n\n", self.item_title));

        if let Some(page) = self.page {
            md.push_str(&format!("**页码**: {}\n\n", page));
        }

        md.push_str(&format!("**模板**: {}\n\n", self.template.display_name()));

        if !self.tags.is_empty() {
            md.push_str(&format!("**标签**: {}\n\n", self.tags.join(", ")));
        }

        // 原文引用
        if let Some(source) = &self.source_text {
            md.push_str("## 原文引用\n\n");
            md.push_str(&format!("> {}\n\n", source));
        }

        // 笔记内容
        md.push_str("## 笔记内容\n\n");
        md.push_str(&self.content);
        md.push_str("\n\n");

        // 底部信息
        md.push_str("---\n\n");
        md.push_str(&format!(
            "*创建时间: {} | 版本: {}*\n",
            chrono_datetime_string(self.created_at),
            self.version
        ));

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

/// 生成简单的 UUID
fn generate_uuid() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let random: u32 = rand_simple();
    format!("{:x}-{:x}", now, random)
}

/// 简单的随机数生成器
fn rand_simple() -> u32 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let state = RandomState::new();
    let mut hasher = state.build_hasher();
    hasher.write_u64(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
    );
    hasher.finish() as u32
}

/// 笔记生成器
pub struct NoteGenerator {
    /// AI Provider
    provider: Arc<dyn AIProvider>,
    /// PDF 标注存储（用于获取用户高亮）
    annotation_storage: Option<AnnotationStorage>,
}

impl NoteGenerator {
    /// 创建新的笔记生成器
    pub fn new(provider: Arc<dyn AIProvider>) -> Self {
        Self {
            provider,
            annotation_storage: None,
        }
    }

    /// 创建带标注存储的笔记生成器
    pub fn with_annotation_storage(provider: Arc<dyn AIProvider>, storage: AnnotationStorage) -> Self {
        Self {
            provider,
            annotation_storage: Some(storage),
        }
    }

    /// 获取文献元数据
    fn fetch_item_metadata(&self, item_id: i32) -> Result<ItemInfo, NoteError> {
        // 使用同步方式获取（因为我们在 tokio::task::spawn_blocking 中调用）
        let guard = get_connection().map_err(|e| NoteError::DatabaseError(e.to_string()))?;
        let conn = guard.as_ref().ok_or_else(|| NoteError::DatabaseError("数据库连接未初始化".to_string()))?;

        // 获取动态元数据
        let metadata = get_cached_metadata(conn)
            .map_err(|e| NoteError::DatabaseError(format!("获取元数据失败: {}", e)))?;
        let dynamic = DynamicSqlBuilder::new(&metadata);

        // 动态获取表名
        let items_table = dynamic.find_table(ZoteroTableCandidates::ITEMS)
            .ok_or_else(|| NoteError::DatabaseError("未找到 items 表".to_string()))?;
        let item_data_table = dynamic.find_table(ZoteroTableCandidates::ITEM_DATA)
            .ok_or_else(|| NoteError::DatabaseError("未找到 itemData 表".to_string()))?;
        let item_data_values_table = dynamic.find_table(ZoteroTableCandidates::ITEM_DATA_VALUES)
            .ok_or_else(|| NoteError::DatabaseError("未找到 itemDataValues 表".to_string()))?;
        let fields_table = dynamic.find_table(ZoteroTableCandidates::FIELDS)
            .ok_or_else(|| NoteError::DatabaseError("未找到 fields 表".to_string()))?;
        let authors_table = dynamic.find_table(ZoteroTableCandidates::CREATORS)
            .ok_or_else(|| NoteError::DatabaseError("未找到 itemCreators 表".to_string()))?;
        let creators_table = dynamic.find_table(ZoteroTableCandidates::CREATOR)
            .ok_or_else(|| NoteError::DatabaseError("未找到 creators 表".to_string()))?;

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

        conn.query_row(&sql, params![item_id], |row| {
            Ok(ItemInfo {
                item_id: row.get(0)?,
                title: row.get::<_, String>(1).unwrap_or_default(),
                year: row.get::<_, String>(2).unwrap_or_default(),
                authors: row.get::<_, String>(3).unwrap_or_default(),
            })
        })
        .map_err(|e| NoteError::DatabaseError(format!("查询文献元数据失败: {}", e)))
    }

    /// 获取用户在高亮中的文本内容
    fn fetch_user_highlights(&self, pdf_key: &str) -> Vec<(String, u32)> {
        if let Some(storage) = &self.annotation_storage {
            if let Ok(annotations) = storage.get_annotations(pdf_key) {
                return annotations
                    .iter()
                    .filter(|a| matches!(a.annotation_type, AnnotationType::Highlight))
                    .filter_map(|a| {
                        if let AnnotationData::Highlight(h) = &a.data {
                            Some((h.text.clone(), a.page))
                        } else {
                            None
                        }
                    })
                    .collect();
            }
        }
        Vec::new()
    }

    /// 生成单个笔记
    pub async fn generate_note(
        &self,
        item_id: i32,
        source_text: Option<String>,
        page: Option<u32>,
        template: NoteTemplate,
    ) -> Result<Note, NoteError> {
        eprintln!("[笔记] 开始生成笔记: item_id={}, template={:?}", item_id, template);

        // 获取文献元数据
        let metadata = self.fetch_item_metadata(item_id)?;

        // 构建提示词
        let prompt = self.build_note_prompt(&metadata, &source_text, template.clone());

        // 调用 AI 生成笔记
        let messages = vec![
            Message::system("你是一个专业的学术笔记助手。请根据提供的论文信息和原文内容，生成结构化的学术笔记。"),
            Message::user(prompt),
        ];

        let response = self.provider.chat_completion(messages)
            .map_err(|e| NoteError::AIError(e.to_string()))?;

        // 解析 AI 响应，生成笔记结构体
        let (title, content) = self.parse_note_response(&response, &template)?;

        let note = Note::new(
            item_id,
            metadata.title.clone(),
            title,
            content,
            template,
            source_text,
            page,
            Vec::new(),
        );

        eprintln!("[笔记] 笔记生成完成: note_id={}", note.note_id);
        Ok(note)
    }

    /// 批量生成笔记（基于多个高亮段落）
    pub async fn generate_notes_batch(
        &self,
        item_id: i32,
        pdf_key: &str,
        template: NoteTemplate,
    ) -> Result<Vec<Note>, NoteError> {
        eprintln!("[笔记] 开始批量生成笔记: item_id={}, pdf_key={}", item_id, pdf_key);

        // 获取文献元数据（用于后续扩展）
        let _metadata = self.fetch_item_metadata(item_id)?;

        // 获取所有高亮
        let highlights = self.fetch_user_highlights(pdf_key);

        if highlights.is_empty() {
            return Err(NoteError::NoHighlightsFound);
        }

        let mut notes = Vec::new();

        // 为每个高亮生成笔记
        for (source_text, page) in highlights {
            match self.generate_note(item_id, Some(source_text.clone()), Some(page), template.clone()).await {
                Ok(note) => notes.push(note),
                Err(e) => {
                    eprintln!("[笔记] 生成单条笔记失败: {:?}", e);
                    // 继续处理其他高亮
                }
            }
        }

        eprintln!("[笔记] 批量生成完成: 共生成 {} 条笔记", notes.len());
        Ok(notes)
    }

    /// 构建笔记生成提示词
    fn build_note_prompt(
        &self,
        metadata: &ItemInfo,
        source_text: &Option<String>,
        template: NoteTemplate,
    ) -> String {
        let mut prompt = format!(
            "请为以下学术论文生成{}：\n\n\
            文献标题：{}\n\
            作者：{}\n\n",
            template.display_name(),
            metadata.title,
            metadata.authors
        );

        if let Some(text) = source_text {
            prompt.push_str("原文内容：\n");
            prompt.push_str(text);
            prompt.push_str("\n\n");
        }

        prompt.push_str(&match template {
            NoteTemplate::KeyPoints => {
                "请根据原文内容，生成要点笔记。要求：\n\
               1. 提取3-5 个关键信息点\n\
                2. 每个要点用简洁的语言描述\n\
                3. 可以使用 bullet points 格式\n\n\
                输出格式：\n\
                标题：[根据内容生成简短的标题]\n\
                内容：[Markdown格式的要点列表]".to_string()
            }
            NoteTemplate::Methods => {
                "请根据原文内容，生成方法笔记。要求：\n\
                1. 详细记录研究方法和技术路线\n\
                2. 说明实验设计和数据收集方法\n\
                3. 描述分析方法和工具\n\n\
                输出格式：\n\
                标题：[根据内容生成简短的标题]\n\
                内容：[Markdown 格式的方法描述]".to_string()
            }
            NoteTemplate::Conclusions => {
                "请根据原文内容，生成结论笔记。要求：\n\
                1. 总结主要研究发现\n\
                2. 阐述研究贡献和意义\n\
                3. 指出可能的实际应用\n\n\
                输出格式：\n\
                标题：[根据内容生成简短的标题]\n\
                内容：[Markdown 格式的结论描述]".to_string()
            }
            NoteTemplate::Critical => {
                "请根据原文内容，生成批判性笔记。要求：\n\
                1. 分析论文的主要优点\n\
                2. 指出研究的局限性和不足\n\
                3. 提出可能的改进方向\n\n\
                输出格式：\n\
                标题：[根据内容生成简短的标题]\n\
                内容：[Markdown 格式的批判性分析]".to_string()
            }
            NoteTemplate::General => {
                "请根据原文内容，生成通用笔记。要求：\n\
                1. 总结主要内容\n\
                2. 记录你的思考和感悟\n\
                3. 可以包含问题、想法、疑问等\n\n\
                输出格式：\n\
                标题：[根据内容生成简短的标题]\n\
                内容：[Markdown 格式的自由笔记]".to_string()
            }
        });

        prompt
    }

    /// 解析 AI 响应，提取标题和内容
    fn parse_note_response(
        &self,
        response: &str,
        template: &NoteTemplate,
    ) -> Result<(String, String), NoteError> {
        // 尝试解析标题（以 "标题：" 开头）
        let mut title = format!("{}笔记", template.display_name());
        let mut content = response.to_string();

        // 尝试从响应中提取标题
        for line in response.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("标题：") || trimmed.starts_with("标题:") {
                title = trimmed
                    .trim_start_matches("标题：")
                    .trim_start_matches("标题:")
                    .trim()
                    .to_string();
                // 移除这一行
                content = response.lines()
                    .filter(|l| !l.contains("标题：") && !l.contains("标题:"))
                    .collect::<Vec<_>>()
                    .join("\n");
                break;
            }
        }

        // 如果标题为空或太短，使用默认标题
        if title.len() < 3 {
            title = format!("{}笔记", template.display_name());
        }

        Ok((title, content))
    }
}

/// 笔记错误类型
#[derive(Debug)]
pub enum NoteError {
    /// AI 调用错误
    AIError(String),
    /// 数据库错误
    DatabaseError(String),
    /// 解析错误
    ParseError(String),
    /// 序列化错误
    SerializeError(String),
    /// 没有找到高亮
    NoHighlightsFound,
    /// 笔记不存在
    NoteNotFound,
}

impl std::fmt::Display for NoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoteError::AIError(msg) => write!(f, "AI 调用失败: {}", msg),
            NoteError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            NoteError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            NoteError::SerializeError(msg) => write!(f, "序列化错误: {}", msg),
            NoteError::NoHighlightsFound => write!(f, "没有找到高亮内容，请先在高亮文本上生成笔记"),
            NoteError::NoteNotFound => write!(f, "笔记不存在"),
        }
    }
}

impl std::error::Error for NoteError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_creation() {
        let note = Note::new(
            1,
            "测试文献".to_string(),
            "测试标题".to_string(),
            "测试内容".to_string(),
            NoteTemplate::General,
            Some("原文引用".to_string()),
            Some(1),
            vec!["标签1".to_string()],
        );

        assert_eq!(note.item_id, 1);
        assert_eq!(note.title, "测试标题");
        assert_eq!(note.content, "测试内容");
        assert_eq!(note.template, NoteTemplate::General);
        assert!(note.source_text.is_some());
        assert_eq!(note.version, 1);
    }

    #[test]
    fn test_note_markdown() {
        let note = Note::new(
            1,
            "测试文献".to_string(),
            "测试标题".to_string(),
            "测试内容".to_string(),
            NoteTemplate::KeyPoints,
            Some("原文引用".to_string()),
            Some(10),
            vec!["标签1".to_string(), "标签2".to_string()],
        );

        let md = note.to_markdown();
        assert!(md.contains("# 测试标题"));
        assert!(md.contains("**文献**: 测试文献"));
        assert!(md.contains("**页码**: 10"));
        assert!(md.contains("**模板**: 要点笔记"));
        assert!(md.contains("## 原文引用"));
        assert!(md.contains("> 原文引用"));
    }

    #[test]
    fn test_note_serialization() {
        let note = Note::new(
            1,
            "测试文献".to_string(),
            "测试标题".to_string(),
            "测试内容".to_string(),
            NoteTemplate::Methods,
            None,
            None,
            Vec::new(),
        );

        let json = note.to_json().unwrap();
        let parsed = Note::from_json(&json).unwrap();

        assert_eq!(parsed.item_id, note.item_id);
        assert_eq!(parsed.title, note.title);
        assert_eq!(parsed.content, note.content);
        assert_eq!(parsed.template, note.template);
    }
}