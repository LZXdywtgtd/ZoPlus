//! Tantivy 索引构建器模块
//!
//! 本模块负责构建和维护 Zotero 文献的全文搜索索引。
//! 支持批量构建索引和增量更新（新增、修改、删除）。
//!
//! #核心功能
//! - [x] 索引构建与增量更新
//! - [x] 中文分词支持（通过 SimpleAnalyzer）
//! - [x] 数据库变更监听（预留接口）

use std::path::PathBuf;
use std::sync::Mutex;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy};

use crate::db::items::{get_all_items, ItemInfo};

use super::schema::{get_field, IndexSchemaBuilder};

/// 索引构建器错误类型
#[derive(Debug, thiserror::Error)]
pub enum IndexerError {
    /// Tantivy 索引操作错误
    #[error("索引操作失败: {0}")]
    IndexError(#[from] tantivy::TantivyError),
    /// Schema 构建错误
    #[error("Schema 构建失败: {0}")]
    SchemaError(String),
    /// IO 错误
    #[error("文件操作失败: {0}")]
    IoError(#[from] std::io::Error),
    /// 数据库错误
    #[error("数据库查询失败: {0}")]
    DbError(#[from] crate::db::connection::DbError),
    /// 索引路径未初始化
    #[error("索引路径未初始化，请先调用 initialize_index()")]
    IndexPathNotInitialized,
    /// 查询解析错误
    #[error("查询解析失败: {0}")]
    QueryParseError(String),
}

/// 文献文档结构
/// 对应索引中的一篇文献记录
#[derive(Debug, Clone)]
pub struct IndexDocument {
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
    /// 全文路径
    pub fulltext_path: String,
    /// 标签
    pub tags: String,
}

impl From<ItemInfo> for IndexDocument {
    fn from(item: ItemInfo) -> Self {
        Self {
            item_id: item.item_id,
            title: item.title,
            authors: item.authors,
            year: item.year,
            abstract_text: String::new(), // ItemInfo 不包含摘要，从其他表获取
            keywords: String::new(),
            fulltext_path: String::new(),
            tags: String::new(),
        }
    }
}

/// 索引构建器
///
///负责创建和维护 Tantivy 索引，支持批量构建和增量更新。
pub struct SearchIndexer {
    /// 索引路径
    index_path: PathBuf,
    /// 索引 Schema
    schema: Schema,
    /// Index writer（写索引用）
    writer: Option<Mutex<IndexWriter>>,
    /// Index reader（读索引用）
    reader: Option<IndexReader>,
}

impl SearchIndexer {
    /// 创建新的索引构建器
    pub fn new(index_path: PathBuf) -> Result<Self, IndexerError> {
        let mut schema_builder = IndexSchemaBuilder::new();
        schema_builder.add_item_id_field();
        schema_builder.add_title_field();
        schema_builder.add_authors_field();
        schema_builder.add_year_field();
        schema_builder.add_abstract_field();
        schema_builder.add_keywords_field();
        schema_builder.add_fulltext_path_field();
        schema_builder.add_tags_field();
        let schema = schema_builder.build(); // 消费 schema_builder

        Ok(Self {
            index_path,
            schema,
            writer: None,
            reader: None,
        })
    }

    ///初始化索引目录
    /// 如果索引目录不存在，则创建
    pub fn initialize_index(&mut self) -> Result<(), IndexerError> {
        // 创建索引目录（如果不存在）
        if !self.index_path.exists() {
            std::fs::create_dir_all(&self.index_path)?;
        }

        // 打开或创建索引
        let index = if self.index_path.join("meta.json").exists() {
            Index::open_in_dir(&self.index_path)?
        } else {
            Index::create_in_dir(&self.index_path, self.schema.clone())?
        };

        // 创建 Writer（内存缓冲 50MB）
        let writer = index.writer(50_000_000)?;
        self.writer = Some(Mutex::new(writer));

        // 创建 Reader（实时刷新）
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;
        self.reader = Some(reader);

        Ok(())
    }

    /// 获取索引写入器
    fn get_writer(&self) -> Result<std::sync::MutexGuard<IndexWriter>, IndexerError> {
        let writer = self
            .writer
            .as_ref()
            .ok_or(IndexerError::IndexPathNotInitialized)?;
        Ok(writer
            .lock()
            .map_err(|_| IndexerError::IndexPathNotInitialized)?)
    }

    /// 获取索引读取器
    pub fn get_reader(&self) -> Result<&IndexReader, IndexerError> {
        self.reader
            .as_ref()
            .ok_or(IndexerError::IndexPathNotInitialized)
    }

    /// 清空索引（删除所有文档）
    pub fn clear_index(&self) -> Result<(), IndexerError> {
        let mut writer = self.get_writer()?;
        writer.delete_all_documents()?;
        writer.commit()?;
        Ok(())
    }

    /// 添加单篇文献到索引
    pub fn add_document(&self, doc: IndexDocument) -> Result<(), IndexerError> {
        let writer = self.get_writer()?;
        let item_id_field = get_field(&self.schema, "item_id");
        let title_field = get_field(&self.schema, "title");
        let authors_field = get_field(&self.schema, "authors");
        let year_field = get_field(&self.schema, "year");
        let abstract_field = get_field(&self.schema, "abstract");
        let keywords_field = get_field(&self.schema, "keywords");
        let fulltext_path_field = get_field(&self.schema, "fulltext_path");
        let tags_field = get_field(&self.schema, "tags");

        let tantivy_doc = doc!(
            item_id_field => doc.item_id as i64,
            title_field => doc.title,
            authors_field => doc.authors,
            year_field => doc.year,
            abstract_field => doc.abstract_text,
            keywords_field => doc.keywords,
            fulltext_path_field => doc.fulltext_path,
            tags_field => doc.tags,
        );

        writer.add_document(tantivy_doc)?;
        Ok(())
    }

    /// 批量添加文献到索引
    pub fn add_documents(&self, docs: Vec<IndexDocument>) -> Result<(), IndexerError> {
        let writer = self.get_writer()?;
        let item_id_field = get_field(&self.schema, "item_id");
        let title_field = get_field(&self.schema, "title");
        let authors_field = get_field(&self.schema, "authors");
        let year_field = get_field(&self.schema, "year");
        let abstract_field = get_field(&self.schema, "abstract");
        let keywords_field = get_field(&self.schema, "keywords");
        let fulltext_path_field = get_field(&self.schema, "fulltext_path");
        let tags_field = get_field(&self.schema, "tags");

        for doc_struct in docs {
            let tantivy_doc = doc!(
                item_id_field => doc_struct.item_id as i64,
                title_field => doc_struct.title,
                authors_field => doc_struct.authors,
                year_field => doc_struct.year,
                abstract_field => doc_struct.abstract_text,
                keywords_field => doc_struct.keywords,
                fulltext_path_field => doc_struct.fulltext_path,
                tags_field => doc_struct.tags,
            );
            writer.add_document(tantivy_doc)?;
        }

        Ok(())
    }

    /// 更新索引中的文献（先删除后添加）
    pub fn update_document(&self, doc: IndexDocument) -> Result<(), IndexerError> {
        let writer = self.get_writer()?;
        let item_id_field = get_field(&self.schema, "item_id");

        // 删除旧文档
        let term = tantivy::Term::from_field_i64(item_id_field, doc.item_id as i64);
        writer.delete_term(term);

        // 添加新文档
        drop(writer);
        self.add_document(doc)?;

        Ok(())
    }

    /// 从索引中删除文献
    pub fn delete_document(&self, item_id: i32) -> Result<(), IndexerError> {
        let writer = self.get_writer()?;
        let item_id_field = get_field(&self.schema, "item_id");

        let term = tantivy::Term::from_field_i64(item_id_field, item_id as i64);
        writer.delete_term(term);

        Ok(())
    }

    /// 提交索引变更
    pub fn commit(&self) -> Result<(), IndexerError> {
        let mut writer = self.get_writer()?;
        writer.commit()?;
        Ok(())
    }

    /// 获取索引中的文档总数
    pub fn get_document_count(&self) -> Result<usize, IndexerError> {
        let reader = self.get_reader()?;
        let searcher = reader.searcher();
        Ok(searcher.num_docs() as usize)
    }

    /// 从 Zotero 数据库批量构建索引
    ///
    /// # 参数
    /// * `progress_callback` - 进度回调函数，接收 (已处理数, 总数)
    pub fn build_index_from_database<F>(&self, progress_callback: F) -> Result<usize, IndexerError>
    where
        F: Fn(usize, usize),
    {
        // 清空现有索引
        self.clear_index()?;

        // 从数据库获取所有文献
        let items = get_all_items()?;
        let total = items.len();
        let mut docs: Vec<IndexDocument> = Vec::with_capacity(total);

        for (idx, item) in items.into_iter().enumerate() {
            docs.push(item.into());
            // 每100 条提交一次
            if (idx + 1) % 100 == 0 || idx + 1 == total {
                self.add_documents(docs.clone())?;
                docs.clear();
                progress_callback(idx + 1, total);
            }
        }

        // 提交最终变更
        self.commit()?;

        Ok(total)
    }

    /// 获取索引路径
    pub fn get_index_path(&self) -> &PathBuf {
        &self.index_path
    }

    /// 获取 Schema
    pub fn get_schema(&self) -> &Schema {
        &self.schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_indexer() {
        let temp_dir = TempDir::new().unwrap();
        let indexer = SearchIndexer::new(temp_dir.path().to_path_buf()).unwrap();
        assert!(indexer.get_index_path().exists() == false); // 初始化前不存在
    }

    #[test]
    fn test_add_document() {
        let temp_dir = TempDir::new().unwrap();
        let mut indexer = SearchIndexer::new(temp_dir.path().to_path_buf()).unwrap();
        indexer.initialize_index().unwrap();

        let doc = IndexDocument {
            item_id: 1,
            title: "测试标题".to_string(),
            authors: "张三;李四".to_string(),
            year: "2024".to_string(),
            abstract_text: "这是摘要".to_string(),
            keywords: "测试;关键词".to_string(),
            fulltext_path: "/path/to/pdf".to_string(),
            tags: "论文".to_string(),
        };

        indexer.add_document(doc).unwrap();
        indexer.commit().unwrap();

        assert_eq!(indexer.get_document_count().unwrap(), 1);
    }
}
