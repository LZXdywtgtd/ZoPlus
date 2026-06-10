//! Tantivy 搜索引擎模块
//!
//! 本模块负责执行全文搜索查询，支持多字段搜索、模糊搜索和结果高亮。
//!
//! #核心功能
//! - [x] 多字段同时搜索
//! - [x] 模糊搜索支持
//! - [x] 搜索结果高亮

use std::sync::Arc;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, TantivyDocument};

use super::indexer::{IndexerError, SearchIndexer};
use super::schema::get_field;

/// 搜索结果结构体
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
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
    /// 相关度得分
    pub score: f32,
    /// 高亮后的标题片段
    pub title_highlighted: String,
    /// 高亮后的作者片段
    pub authors_highlighted: String,
    /// 高亮后的摘要片段
    pub abstract_highlighted: String,
}

/// 分页搜索参数
#[derive(Debug, Clone)]
pub struct SearchParams {
    /// 搜索关键词
    pub query: String,
    /// 跳过的记录数（从0开始）
    pub offset: usize,
    /// 返回的记录数上限
    pub limit: usize,
    /// 是否启用模糊搜索
    pub fuzzy: bool,
    /// 模糊搜索的编辑距离（0表示精确匹配）
    pub fuzzy_distance: u8,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            query: String::new(),
            offset: 0,
            limit: 20,
            fuzzy: false,
            fuzzy_distance: 2,
        }
    }
}

///搜索引擎
///
///负责执行搜索查询，支持多字段搜索、模糊搜索、分页和结果高亮。
pub struct SearchEngine {
    /// 索引构建器引用（使用 Arc 共享）
    indexer: Arc<SearchIndexer>,
    /// Schema 副本（用于构建查询）
    schema: Schema,
}

impl SearchEngine {
    /// 创建新的搜索引擎
    pub fn new(indexer: Arc<SearchIndexer>) -> Self {
        let schema = indexer.get_schema().clone();
        Self { indexer, schema }
    }

    /// 执行搜索查询
    ///
    /// # 参数
    /// * `params` - 搜索参数
    ///
    /// # 返回值
    /// * `Result<Vec<SearchResult>, IndexerError>` - 搜索结果列表
    pub fn search(&self, params: SearchParams) -> Result<Vec<SearchResult>, IndexerError> {
        let reader = self.indexer.get_reader()?;
        let searcher = reader.searcher();

        // 获取字段
        let item_id_field = get_field(&self.schema, "item_id");
        let title_field = get_field(&self.schema, "title");
        let authors_field = get_field(&self.schema, "authors");
        let year_field = get_field(&self.schema, "year");
        let abstract_field = get_field(&self.schema, "abstract");
        let keywords_field = get_field(&self.schema, "keywords");
        let tags_field = get_field(&self.schema, "tags");

        // 打开索引以创建 QueryParser
        let index = Index::open_in_dir(self.indexer.get_index_path())?;

        // 构建查询解析器
        let query_parser = QueryParser::for_index(
            &index,
            vec![
                title_field,
                authors_field,
                abstract_field,
                keywords_field,
                tags_field,
            ],
        );

        // 解析查询字符串
        let query_string = if params.fuzzy {
            // 模糊搜索：使用 ~ 表示模糊匹配
            format!("{}~{}", params.query, params.fuzzy_distance)
        } else {
            params.query.clone()
        };

        let parsed_query = query_parser
            .parse_query(&query_string)
            .map_err(|e| IndexerError::QueryParseError(format!("查询解析失败: {}", e)))?;

        // 执行搜索
        let top_docs = searcher
            .search(
                &parsed_query,
                &TopDocs::with_limit(params.limit + params.offset).order_by_score(),
            )
            .map_err(|e| IndexerError::IndexError(e))?;

        // 提取结果
        let mut results = Vec::new();
        for (score, doc_address) in top_docs.into_iter().skip(params.offset) {
            let doc: TantivyDocument = searcher.doc(doc_address)?;

            let item_id = doc
                .get_first(item_id_field)
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            let title = doc
                .get_first(title_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let authors = doc
                .get_first(authors_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let year = doc
                .get_first(year_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let abstract_text = doc
                .get_first(abstract_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let keywords = doc
                .get_first(keywords_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let fulltext_path = doc
                .get_first(get_field(&self.schema, "fulltext_path"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let tags = doc
                .get_first(tags_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            // 生成高亮片段（简单实现：对匹配词加粗）
            let title_highlighted = highlight_text(&title, &params.query);
            let authors_highlighted = highlight_text(&authors, &params.query);
            let abstract_highlighted = highlight_text(&abstract_text, &params.query);

            results.push(SearchResult {
                item_id,
                title,
                authors,
                year,
                abstract_text,
                keywords,
                fulltext_path,
                tags,
                score,
                title_highlighted,
                authors_highlighted,
                abstract_highlighted,
            });
        }

        Ok(results)
    }

    /// 执行多字段精确搜索
    ///
    /// # 参数
    /// * `field` - 字段名
    /// * `value` - 搜索值
    /// * `limit` - 返回结果数量上限
    pub fn search_by_field(
        &self,
        field: &str,
        value: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, IndexerError> {
        let reader = self.indexer.get_reader()?;
        let searcher = reader.searcher();

        let field = get_field(&self.schema, field);
        let term = tantivy::Term::from_field_text(field, value);
        let term_query =
            tantivy::query::TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);

        let top_docs = searcher
            .search(&term_query, &TopDocs::with_limit(limit).order_by_score())
            .map_err(|e| IndexerError::IndexError(e))?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher.doc(doc_address)?;

            let item_id_field = get_field(&self.schema, "item_id");
            let item_id = doc
                .get_first(item_id_field)
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            let title = doc
                .get_first(get_field(&self.schema, "title"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let authors = doc
                .get_first(get_field(&self.schema, "authors"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let year = doc
                .get_first(get_field(&self.schema, "year"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            results.push(SearchResult {
                item_id,
                title: title.clone(),
                authors: authors.clone(),
                year,
                abstract_text: doc
                    .get_first(get_field(&self.schema, "abstract"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                keywords: doc
                    .get_first(get_field(&self.schema, "keywords"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                fulltext_path: doc
                    .get_first(get_field(&self.schema, "fulltext_path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                tags: doc
                    .get_first(get_field(&self.schema, "tags"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                score,
                title_highlighted: title,
                authors_highlighted: authors,
                abstract_highlighted: String::new(),
            });
        }

        Ok(results)
    }

    /// 获取搜索结果总数
    pub fn get_total_count(&self, query: &str) -> Result<usize, IndexerError> {
        let reader = self.indexer.get_reader()?;
        let searcher = reader.searcher();

        let title_field = get_field(&self.schema, "title");
        let authors_field = get_field(&self.schema, "authors");
        let abstract_field = get_field(&self.schema, "abstract");
        let keywords_field = get_field(&self.schema, "keywords");
        let tags_field = get_field(&self.schema, "tags");

        let index = Index::open_in_dir(self.indexer.get_index_path())?;

        let query_parser = QueryParser::for_index(
            &index,
            vec![
                title_field,
                authors_field,
                abstract_field,
                keywords_field,
                tags_field,
            ],
        );

        let parsed_query = query_parser
            .parse_query(query)
            .map_err(|e| IndexerError::QueryParseError(format!("查询解析失败: {}", e)))?;

        let top_docs =
            searcher.search(&parsed_query, &TopDocs::with_limit(10000).order_by_score())?;
        Ok(top_docs.len())
    }
}

/// 对文本中的搜索关键词进行高亮标记
///
/// # 参数
/// * `text` - 原始文本
/// * `query` - 搜索关键词
///
/// # 返回值
/// * 高亮后的文本（使用 **包裹匹配词）
fn highlight_text(text: &str, query: &str) -> String {
    if query.is_empty() || text.is_empty() {
        return text.to_string();
    }

    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    let mut result = String::new();
    let mut last_end = 0;
    let mut search_start = 0;

    while let Some(pos) = text_lower[search_start..].find(&query_lower) {
        let abs_pos = search_start + pos;
        // 添加匹配前的文本
        result.push_str(&text[last_end..abs_pos]);
        // 添加高亮标记（匹配词）
        result.push_str("**");
        result.push_str(&text[abs_pos..abs_pos + query.len()]);
        result.push_str("**");
        last_end = abs_pos + query.len();
        search_start = last_end;
    }

    // 添加剩余文本
    result.push_str(&text[last_end..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_text() {
        let text = "这是一段测试文本，用于测试高亮功能";
        let query = "测试";
        let result = highlight_text(text, query);
        assert!(result.contains("**测试**"));
    }

    #[test]
    fn test_highlight_text_no_match() {
        let text = "这是一段测试文本";
        let query = "不存在";
        let result = highlight_text(text, query);
        assert_eq!(result, text);
    }
}
