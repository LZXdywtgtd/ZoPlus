//! Tantivy 全文搜索引擎模块
//!
//! 本模块用于构建和维护 Zotero 文献的全文搜索索引。
//! 支持中文分词和搜索结果高亮。
//!
//! # 模块结构
//! - `schema` - 索引结构定义
//! - `indexer` - 索引构建器
//! - `query` - 搜索引擎
//! - `commands` - Tauri Command 接口
//!
//! # 功能说明
//! - [x] 索引构建与增量更新
//! - [x] 中文分词支持（通过 SimpleAnalyzer）
//! - [x] 搜索结果高亮
//! - [x] 数据库变更监听（预留接口）

pub mod commands;
pub mod indexer;
pub mod query;
pub mod schema;

pub use commands::{IndexBuildProgress, IndexStatus, SearchRequest, SearchResponse, SearchState};
pub use indexer::{IndexDocument, IndexerError, SearchIndexer};
pub use query::{SearchEngine, SearchParams, SearchResult};
pub use schema::{get_field, get_field_names, get_field_opt, IndexField, IndexSchemaBuilder};
