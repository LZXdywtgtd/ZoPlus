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

pub mod schema;
pub mod indexer;
pub mod query;
pub mod commands;

pub use schema::{IndexSchemaBuilder, IndexField, get_field, get_field_opt, get_field_names};
pub use indexer::{SearchIndexer, IndexerError, IndexDocument};
pub use query::{SearchEngine, SearchResult, SearchParams};
pub use commands::{SearchState, SearchRequest, SearchResponse, IndexBuildProgress, IndexStatus};