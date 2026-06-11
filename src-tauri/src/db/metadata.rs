//! 数据库元数据扫描系统
//!
//! 提供完全动态的数据库自省能力，自动检测 Zotero 数据库的实际表结构和字段。
//!
//! # 核心功能
//! - 扫描数据库所有表和字段元数据
//! - 动态构建 SQL 语句，不依赖硬编码表名
//! - 缓存元数据避免重复扫描
//! - 智能查找表（支持多候选表名）
//!
//! # 使用方式
//! ```rust
//! let metadata = DatabaseMetadata::scan_database(&conn)?;
//! if metadata.table_exists("items") {
//!     let table = metadata.get_table("items").unwrap();
//!     // 动态构建查询...
//! }
//! ```

use rusqlite::{Connection, Result as SqliteResult};
use std::collections::HashMap;

/// 表元数据
#[derive(Debug, Clone)]
pub struct TableMetadata {
    /// 表名
    pub name: String,
    /// 字段列表
    pub columns: Vec<ColumnMetadata>,
}

/// 字段元数据
#[derive(Debug, Clone)]
pub struct ColumnMetadata {
    /// 字段名
    pub name: String,
    /// 数据类型
    pub data_type: String,
    /// 是否主键
    pub is_primary_key: bool,
    /// 是否可为空
    pub is_nullable: bool,
}

/// 数据库元数据中心
///
/// 通过扫描数据库获取所有表结构信息，支持大小写不敏感查询。
#[derive(Debug, Clone)]
pub struct DatabaseMetadata {
    /// 表名到元数据的映射（key 为小写）
    tables: HashMap<String, TableMetadata>,
}

impl DatabaseMetadata {
    /// 扫描数据库所有表
    ///
    /// # 参数
    /// * `conn` - 数据库连接
    ///
    /// # 返回值
    /// * `SqliteResult<Self>` - 元数据中心实例
    pub fn scan_database(conn: &Connection) -> SqliteResult<Self> {
        let mut tables = HashMap::new();

        // 获取所有用户表（排除 sqlite 系统表）
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
        )?;

        let table_names: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        tracing::info!("[元数据] 扫描到 {} 个表", table_names.len());

        // 扫描每个表的字段
        for name in table_names {
            let columns = Self::scan_columns(conn, &name)?;
            let meta = TableMetadata {
                name: name.clone(),
                columns,
            };
            tables.insert(name.to_lowercase(), meta);
        }

        Ok(Self { tables })
    }

    /// 扫描表的字段
    fn scan_columns(conn: &Connection, table_name: &str) -> SqliteResult<Vec<ColumnMetadata>> {
        let mut stmt = conn.prepare(&format!("PRAGMA table_info('{}')", table_name))?;
        let columns = stmt
            .query_map([], |row| {
                Ok(ColumnMetadata {
                    name: row.get(1)?,
                    data_type: row.get(2)?,
                    is_primary_key: row.get::<_, i32>(5)? == 1,
                    is_nullable: row.get::<_, i32>(3)? == 0,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(columns)
    }

    /// 检查表是否存在（大小写不敏感）
    pub fn table_exists(&self, name: &str) -> bool {
        self.tables.contains_key(&name.to_lowercase())
    }

    /// 获取表元数据（大小写不敏感）
    pub fn get_table(&self, name: &str) -> Option<&TableMetadata> {
        self.tables.get(&name.to_lowercase())
    }

    /// 检查字段是否存在（大小写不敏感）
    pub fn column_exists(&self, table_name: &str, column_name: &str) -> bool {
        self.get_table(table_name)
            .map(|t| t.columns.iter().any(|c| c.name.eq_ignore_ascii_case(column_name)))
            .unwrap_or(false)
    }

    /// 获取所有表名
    pub fn table_names(&self) -> Vec<&String> {
        self.tables.keys().collect()
    }

    /// 获取表数量
    pub fn table_count(&self) -> usize {
        self.tables.len()
    }

    /// 智能查找表（支持多候选表名）
    ///
    /// 按顺序尝试每个候选表名，返回第一个存在的表。
    /// 用于处理 Zotero 数据库表名可能因版本而异的情况（如 itemCreators/itemAuthors）。
    ///
    /// # 参数
    /// * `candidates` - 候选表名列表，按优先级排序
    ///
    /// # 返回值
    /// * `Option<&str>` - 找到的表名，或 None
    pub fn find_table<'a>(&self, candidates: &[&'a str]) -> Option<&'a str> {
        for name in candidates {
            if self.table_exists(name) {
                return Some(name);
            }
        }
        None
    }

    /// 获取字段信息（大小写不敏感）
    pub fn get_column(&self, table_name: &str, column_name: &str) -> Option<&ColumnMetadata> {
        self.get_table(table_name)
            .and_then(|t| t.columns.iter().find(|c| c.name.eq_ignore_ascii_case(column_name)))
    }
}

// ============================================================
// 动态 SQL 构建器扩展
// ============================================================

impl DatabaseMetadata {
    /// 通用 INSERT 构建器
    ///
    /// 自动过滤不存在的字段，只插入实际存在的列。
    ///
    /// # 参数
    /// * `table` - 表名
    /// * `data` - 字段名到值的映射
    ///
    /// # 返回值
    /// * `Option<(SQL语句, 参数值列表)>` - 构建成功返回 SQL 和参数
    pub fn build_insert<'a>(
        &self,
        table: &str,
        data: &HashMap<&str, &'a str>,
    ) -> Option<(String, Vec<&'a str>)> {
        let table_meta = self.get_table(table)?;

        // 过滤出实际存在的字段（大小写不敏感匹配）
        let valid_fields: Vec<&str> = data
            .keys()
            .filter(|k| {
                table_meta
                    .columns
                    .iter()
                    .any(|c| c.name.eq_ignore_ascii_case(k))
            })
            .copied()
            .collect();

        if valid_fields.is_empty() {
            return None;
        }

        let cols = valid_fields.join(", ");
        let placeholders = valid_fields.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let values: Vec<&str> = valid_fields.iter().map(|k| *data.get(k).unwrap()).collect();

        let sql = format!("INSERT INTO {} ({}) VALUES ({})", table, cols, placeholders);
        Some((sql, values))
    }

    /// 通用 SELECT 构建器
    ///
    /// # 参数
    /// * `table` - 表名
    /// * `columns` - 要查询的字段列表，空表示所有字段
    /// * `where_clause` - WHERE 子句（不含 WHERE 关键字），空表示无条件
    ///
    /// # 返回值
    /// * `Option<String>` - 构建成功的 SQL 语句
    pub fn build_select(&self, table: &str, columns: &[&str], where_clause: &str) -> Option<String> {
        if !self.table_exists(table) {
            return None;
        }

        let cols = if columns.is_empty() {
            "*".to_string()
        } else {
            columns.join(", ")
        };

        let sql = if where_clause.is_empty() {
            format!("SELECT {} FROM {}", cols, table)
        } else {
            format!("SELECT {} FROM {} WHERE {}", cols, table, where_clause)
        };

        Some(sql)
    }

    /// 通用 DELETE 构建器
    ///
    /// # 参数
    /// * `table` - 表名
    /// * `where_clause` - WHERE 子句（不含 WHERE 关键字）
    ///
    /// # 返回值
    /// * `Option<String>` - 构建成功的 SQL 语句
    pub fn build_delete(&self, table: &str, where_clause: &str) -> Option<String> {
        if !self.table_exists(table) {
            return None;
        }
        Some(format!("DELETE FROM {} WHERE {}", table, where_clause))
    }

    /// 通用 UPDATE 构建器
    ///
    /// # 参数
    /// * `table` - 表名
    /// * `data` - 字段名到新值的映射
    /// * `where_clause` - WHERE 子句（不含 WHERE 关键字）
    ///
    /// # 返回值
    /// * `Option<(SQL语句, 参数值列表)>` - 构建成功返回 SQL 和参数
    pub fn build_update<'a>(
        &self,
        table: &str,
        data: &HashMap<&str, &'a str>,
        where_clause: &str,
    ) -> Option<(String, Vec<&'a str>)> {
        let table_meta = self.get_table(table)?;

        // 过滤出实际存在的字段
        let valid_fields: Vec<&str> = data
            .keys()
            .filter(|k| {
                table_meta
                    .columns
                    .iter()
                    .any(|c| c.name.eq_ignore_ascii_case(k))
            })
            .copied()
            .collect();

        if valid_fields.is_empty() {
            return None;
        }

        let set_clause = valid_fields
            .iter()
            .map(|k| format!("{} = ?", k))
            .collect::<Vec<_>>()
            .join(", ");

        let values: Vec<&str> = valid_fields.iter().map(|k| *data.get(k).unwrap()).collect();

        let sql = if where_clause.is_empty() {
            format!("UPDATE {} SET {}", table, set_clause)
        } else {
            format!("UPDATE {} SET {} WHERE {}", table, set_clause, where_clause)
        };

        Some((sql, values))
    }
}

// ============================================================
// 全局元数据缓存（避免重复扫描）
// ============================================================

use std::sync::Mutex;

/// 全局元数据缓存
static METADATA_CACHE: std::sync::OnceLock<Mutex<Option<DatabaseMetadata>>> =
    std::sync::OnceLock::new();

/// 获取缓存的元数据（如果已缓存则返回，否则扫描并缓存）
pub fn get_cached_metadata(conn: &Connection) -> SqliteResult<DatabaseMetadata> {
    let cache = METADATA_CACHE.get_or_init(|| Mutex::new(None));

    // 尝试从缓存获取
    {
        let guard = cache.lock().unwrap();
        if let Some(metadata) = guard.as_ref() {
            // 验证缓存的表数量是否与数据库匹配（检测表数量变化）
            let current_tables: Vec<String> = {
                let mut stmt = conn
                    .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
                    .unwrap();
                stmt.query_map([], |row| row.get(0))
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .collect()
            };

            if current_tables.len() == metadata.table_count() {
                tracing::debug!("[元数据] 使用缓存的元数据（{} 个表）", metadata.table_count());
                return Ok(metadata.clone());
            }
            tracing::info!("[元数据] 缓存表数量不匹配，重新扫描");
        }
    }

    // 扫描并缓存
    let metadata = DatabaseMetadata::scan_database(conn)?;
    let mut guard = cache.lock().unwrap();
    *guard = Some(metadata.clone());
    Ok(metadata)
}

/// 清除元数据缓存（数据库变更后调用）
pub fn invalidate_metadata_cache() {
    if let Some(cache) = METADATA_CACHE.get() {
        let mut guard = cache.lock().unwrap();
        *guard = None;
        tracing::info!("[元数据] 缓存已失效");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_metadata_debug() {
        let col = ColumnMetadata {
            name: "itemID".to_string(),
            data_type: "INTEGER".to_string(),
            is_primary_key: true,
            is_nullable: false,
        };
        assert_eq!(col.name, "itemID");
    }
}