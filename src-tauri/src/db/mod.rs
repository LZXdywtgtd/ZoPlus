//! Zotero 数据库访问层模块
//!
//! 本模块提供对 Zotero 原生 SQLite 数据库的只读访问功能。
//!
//! # 模块结构
//! - `path` - 跨平台数据库路径检测
//! - `connection` - 单例连接管理
//! - `items` - 文献信息查询
//! - `validation` - 数据库完整性验证
//! - `delete` - 文献删除功能（写操作）
//! - `metadata` - 数据库元数据扫描（动态自省）
//! - `dynamic` - 动态 SQL 构建器
//!
//! # 安全规则
//! - 查询操作使用 PRAGMA query_only = ON 强制只读模式
//! - 删除操作使用独立连接，支持写事务
//! - 禁止修改 Zotero 原生业务字段
//! - 所有表名/字段名通过动态自省获取，禁止硬编码

pub mod connection;
pub mod delete;
pub mod dynamic;
pub mod items;
pub mod metadata;
pub mod path;
pub mod validation;

pub use connection::{get_current_db_path, get_database_diagnosis, reset_connection, DbError};
pub use delete::{delete_item_async, delete_items_async, DeleteFailure, DeleteResult};
pub use dynamic::{DynamicSqlBuilder, ZoteroTableCandidates, find_zotero_table, find_or_insert_value};
pub use items::{get_all_items, get_all_items_async, get_item_by_id_async, get_items_paginated_async, ItemInfo};
pub use metadata::{DatabaseMetadata, TableMetadata, ColumnMetadata, get_cached_metadata, invalidate_metadata_cache};
pub use path::{get_zotero_database_path, zotero_db_exists};
pub use validation::{
    diagnose_database, explore_database_structure, get_all_table_names, validate_sqlite_file, validate_zotero_database,
    ColumnInfo, DatabaseDiagnosis, DatabaseStructure, DbValidationError, TableStructure,
};
