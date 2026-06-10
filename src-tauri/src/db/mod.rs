//! Zotero 数据库访问层模块
//!
//! 本模块提供对 Zotero 原生 SQLite 数据库的只读访问功能。
//!
//! # 模块结构
//! - `path` - 跨平台数据库路径检测
//! - `connection` - 单例连接管理
//! - `items` - 文献信息查询
//!
//! # 安全规则
//! - 所有数据库操作均为只读
//! - 禁止修改 Zotero 原生数据
//! - 使用 PRAGMA query_only = ON 强制只读模式

pub mod connection;
pub mod items;
pub mod path;

pub use connection::DbError;
pub use items::{get_all_items, get_item_by_id, get_items_paginated, ItemInfo};
pub use path::{get_zotero_database_path, zotero_db_exists};
