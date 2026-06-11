//! 动态 SQL 构建器
//!
//! 基于 DatabaseMetadata 的动态 SQL 构建功能，支持通用的 CRUD 操作。
//!
//! # 核心功能
//! - 自动检测表和字段是否存在
//! - 智能构建 INSERT/SELECT/UPDATE/DELETE 语句
//! - 支持多候选表名（处理 Zotero 版本差异）
//!
//! # 使用方式
//! ```rust
//! let metadata = DatabaseMetadata::scan_database(&conn)?;
//! let dynamic = DynamicSqlBuilder::new(&metadata);
//!
//! // 构建 INSERT
//! if let Some((sql, params)) = dynamic.insert("items", data) { ... }
//!
//! // 构建 SELECT
//! if let Some(sql) = dynamic.select("itemData", &["itemID", "fieldID"], "itemID = ?") { ... }
//! ```

use crate::db::metadata::DatabaseMetadata;
use std::collections::HashMap;

/// 动态 SQL 构建器
#[derive(Debug, Clone)]
pub struct DynamicSqlBuilder<'a> {
    /// 元数据中心引用
    metadata: &'a DatabaseMetadata,
}

impl<'a> DynamicSqlBuilder<'a> {
    /// 创建新的动态 SQL 构建器
    pub fn new(metadata: &'a DatabaseMetadata) -> Self {
        Self { metadata }
    }

    /// 构建 INSERT 语句
    ///
    /// # 参数
    /// * `table` - 表名
    /// * `data` - 字段名到值的映射
    ///
    /// # 返回值
    /// * `Option<(SQL, 参数值)>` - 成功返回 (SQL语句, 参数列表)
    pub fn insert<'b>(
        &self,
        table: &str,
        data: &HashMap<&str, &'b str>,
    ) -> Option<(String, Vec<&'b str>)> {
        self.metadata.build_insert(table, data)
    }

    /// 构建 SELECT 语句
    ///
    /// # 参数
    /// * `table` - 表名
    /// * `columns` - 要查询的字段列表，空表示所有字段
    /// * `where_clause` - WHERE 子句（不含 WHERE），空表示无条件
    ///
    /// # 返回值
    /// * `Option<String>` - 成功返回 SQL 语句
    pub fn select(
        &self,
        table: &str,
        columns: &[&str],
        where_clause: &str,
    ) -> Option<String> {
        self.metadata.build_select(table, columns, where_clause)
    }

    /// 构建 UPDATE 语句
    ///
    /// # 参数
    /// * `table` - 表名
    /// * `data` - 字段名到新值的映射
    /// * `where_clause` - WHERE 子句（不含 WHERE），空表示无条件（危险操作）
    ///
    /// # 返回值
    /// * `Option<(SQL, 参数值)>` - 成功返回 (SQL语句, 参数列表)
    pub fn update<'b>(
        &self,
        table: &str,
        data: &HashMap<&str, &'b str>,
        where_clause: &str,
    ) -> Option<(String, Vec<&'b str>)> {
        self.metadata.build_update(table, data, where_clause)
    }

    /// 构建 DELETE 语句
    ///
    /// # 参数
    /// * `table` - 表名
    /// * `where_clause` - WHERE 子句（不含 WHERE）
    ///
    /// # 返回值
    /// * `Option<String>` - 成功返回 SQL 语句
    pub fn delete(&self, table: &str, where_clause: &str) -> Option<String> {
        self.metadata.build_delete(table, where_clause)
    }

    /// 查找表（支持多候选表名）
    ///
    /// # 参数
    /// * `candidates` - 候选表名列表，按优先级排序
    ///
    /// # 返回值
    /// * `Option<&'a str>` - 找到的表名
    pub fn find_table(&self, candidates: &[&'a str]) -> Option<&'a str> {
        self.metadata.find_table(candidates)
    }

    /// 检查表是否存在
    pub fn table_exists(&self, name: &str) -> bool {
        self.metadata.table_exists(name)
    }

    /// 检查字段是否存在
    pub fn column_exists(&self, table: &str, column: &str) -> bool {
        self.metadata.column_exists(table, column)
    }

    /// 获取表元数据
    pub fn get_table(&self, name: &str) -> Option<&crate::db::metadata::TableMetadata> {
        self.metadata.get_table(name)
    }
}

// ============================================================
// Zotero 专用表名检测器
// ============================================================

/// Zotero 常用表名的候选列表
pub struct ZoteroTableCandidates;

impl ZoteroTableCandidates {
    /// 作者关联表（itemCreators/itemAuthors/itemCreator）
    pub const CREATORS: &'static [&'static str] = &["itemCreators", "itemAuthors", "itemCreator"];

    /// 作者表（creators）
    pub const CREATOR: &'static [&'static str] = &["creators"];

    /// 文献数据表（itemData）
    pub const ITEM_DATA: &'static [&'static str] = &["itemData"];

    /// 文献数据值表（itemDataValues）
    pub const ITEM_DATA_VALUES: &'static [&'static str] = &["itemDataValues"];

    /// 文献表（items）
    pub const ITEMS: &'static [&'static str] = &["items"];

    /// 文献标签表（itemTags）
    pub const ITEM_TAGS: &'static [&'static str] = &["itemTags", "tags"];

    /// 集合表（collections）
    pub const COLLECTIONS: &'static [&'static str] = &["collections"];

    /// 文献附件表（itemAttachments）
    pub const ITEM_ATTACHMENTS: &'static [&'static str] =
        &["itemAttachments", "attachments"];

    /// 字段表（fields）
    pub const FIELDS: &'static [&'static str] = &["fields"];

    /// 文献类型表（itemTypes）
    pub const ITEM_TYPES: &'static [&'static str] = &["itemTypes"];

    /// 删除文献表（deletedItems）
    pub const DELETED_ITEMS: &'static [&'static str] = &["deletedItems"];
}

/// Zotero 表查找辅助函数
///
/// 返回找到的表名（如果存在）。返回的是 metadata 中存储的表名引用。
pub fn find_zotero_table<'a>(metadata: &'a DatabaseMetadata, hint: &'a str) -> Option<&'a str> {
    // 使用 metadata 的 find_table 方法，它会返回 metadata 中存储的表名引用
    metadata.find_table(&[hint])
}

/// 检测 itemDataValues 表中是否已有相同的值
///
/// # 参数
/// * `conn` - 数据库连接
/// * `metadata` - 元数据中心
/// * `value` - 要查找的值
///
/// # 返回值
/// * `Option<i32>` - 找到则返回 valueID，否则返回 None
pub fn find_existing_value_id(
    conn: &rusqlite::Connection,
    metadata: &DatabaseMetadata,
    value: &str,
) -> Option<i32> {
    let table = metadata.find_table(ZoteroTableCandidates::ITEM_DATA_VALUES)?;

    let sql = format!("SELECT valueID FROM {} WHERE value = ?", table);
    conn.query_row(&sql, [value], |row| row.get(0)).ok()
}

/// 插入新值到 itemDataValues 表
///
/// # 参数
/// * `tx` - 数据库事务
/// * `metadata` - 元数据中心
/// * `value` - 要插入的值
///
/// # 返回值
/// * `Result<i32, String>` - 新插入的 valueID
pub fn insert_value(
    tx: &rusqlite::Transaction,
    metadata: &DatabaseMetadata,
    value: &str,
) -> Result<i32, String> {
    let table = metadata
        .find_table(ZoteroTableCandidates::ITEM_DATA_VALUES)
        .ok_or_else(|| "未找到 itemDataValues 表".to_string())?;

    tx.execute(&format!("INSERT INTO {} (value) VALUES (?)", table), [value])
        .map_err(|e| format!("插入 itemDataValues 失败: {}", e))?;

    tx.query_row("SELECT last_insert_rowid()", [], |row| row.get(0))
        .map_err(|e| format!("获取 last_insert_rowid 失败: {}", e))
}

/// 查找或插入值（如果已存在则返回现有 ID，否则插入并返回新 ID）
///
/// # 参数
/// * `tx` - 数据库事务
/// * `metadata` - 元数据中心
/// * `value` - 要查找/插入的值
///
/// # 返回值
/// * `Result<i32, String>` - valueID
pub fn find_or_insert_value(
    tx: &rusqlite::Transaction,
    metadata: &DatabaseMetadata,
    value: &str,
) -> Result<i32, String> {
    // 先查找是否已存在
    let table = metadata
        .find_table(ZoteroTableCandidates::ITEM_DATA_VALUES)
        .ok_or_else(|| "未找到 itemDataValues 表".to_string())?;

    let sql = format!("SELECT valueID FROM {} WHERE value = ?", table);
    if let Some(value_id) = tx.query_row(&sql, [value], |row| row.get(0)).ok() {
        return Ok(value_id);
    }

    // 不存在则插入
    insert_value(tx, metadata, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zotero_candidates() {
        assert_eq!(ZoteroTableCandidates::CREATORS.len(), 3);
        assert_eq!(ZoteroTableCandidates::ITEMS.len(), 1);
    }
}