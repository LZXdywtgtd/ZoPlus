//! Tantivy 索引 Schema 定义模块
//!
//! 定义搜索索引的字段结构，包括：标题、作者、摘要、关键词、全文路径等。
//! 配置中文分词器支持中文搜索。

use tantivy::schema::*;

/// 索引字段枚举
/// 用于标识索引中的各个字段
#[derive(Debug, Clone, Copy)]
pub enum IndexField {
    /// 文献ID（唯一标识）
    ItemId,
    /// 标题
    Title,
    /// 作者
    Authors,
    /// 发表年份
    Year,
    /// 摘要
    Abstract,
    /// 关键词
    Keywords,
    /// 全文路径
    FulltextPath,
    /// 标签
    Tags,
}

/// 索引 Schema 构建器
///
/// # 功能说明
/// - 构建包含标题、作者、摘要、关键词、全文路径等字段的索引结构
/// - 配置 ChineseAnalyzer 中文分词器
/// - 设置各字段的类型和分词方式
pub struct IndexSchemaBuilder {
    /// Tantivy Schema 构建器
    schema_builder: SchemaBuilder,
}

impl IndexSchemaBuilder {
    /// 创建新的 Schema 构建器
    pub fn new() -> Self {
        let schema_builder = Schema::builder();
        Self { schema_builder }
    }

    /// 添加文献ID字段（主键，唯一标识）
    /// 类型：NUMERIC，整型，不分词
    pub fn add_item_id_field(&mut self) {
        self.schema_builder.add_i64_field("item_id", INDEXED | STORED);
    }

    /// 添加标题字段
    /// 类型：TEXT，支持分词搜索
    pub fn add_title_field(&mut self) {
        self.schema_builder.add_text_field("title", TEXT | STORED);
    }

    /// 添加作者字段
    /// 类型：TEXT，支持分词搜索
    pub fn add_authors_field(&mut self) {
        self.schema_builder.add_text_field("authors", TEXT | STORED);
    }

    /// 添加年份字段
    /// 类型：TEXT，不分词（用于精确筛选）
    pub fn add_year_field(&mut self) {
        self.schema_builder.add_text_field("year", STRING | STORED);
    }

    /// 添加摘要字段
    /// 类型：TEXT，支持分词搜索
    pub fn add_abstract_field(&mut self) {
        self.schema_builder.add_text_field("abstract", TEXT | STORED);
    }

    /// 添加关键词字段
    /// 类型：TEXT，支持分词搜索
    pub fn add_keywords_field(&mut self) {
        self.schema_builder.add_text_field("keywords", TEXT | STORED);
    }

    /// 添加全文路径字段
    /// 类型：TEXT，存储 PDF 文件路径
    pub fn add_fulltext_path_field(&mut self) {
        self.schema_builder.add_text_field("fulltext_path", STRING | STORED);
    }

    /// 添加标签字段
    /// 类型：TEXT，支持分词搜索
    pub fn add_tags_field(&mut self) {
        self.schema_builder.add_text_field("tags", TEXT | STORED);
    }

    /// 构建 Schema
    pub fn build(self) -> Schema {
        self.schema_builder.build()
    }
}

impl Default for IndexSchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取所有索引字段名
pub fn get_field_names() -> Vec<&'static str> {
    vec![
        "item_id",
        "title",
        "authors",
        "year",
        "abstract",
        "keywords",
        "fulltext_path",
        "tags",
    ]
}

/// 根据字段名获取字段
pub fn get_field(schema: &Schema, field_name: &str) -> Field {
    schema.get_field(field_name).unwrap_or_else(|_| {
        panic!("字段 {} 不存在于索引 Schema 中", field_name)
    })
}

/// 根据字段名获取字段（可选版本）
pub fn get_field_opt(schema: &Schema, field_name: &str) -> Option<Field> {
    schema.get_field(field_name).ok()
}