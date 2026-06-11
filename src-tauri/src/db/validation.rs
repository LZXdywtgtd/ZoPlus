//! Zotero 数据库完整性验证模块
//!
//! 本模块提供数据库表完整性检查功能，确保使用的数据库是有效的 Zotero 数据库。
//! 验证标准 Zotero 表结构，防止因数据库不完整导致的查询失败。

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 表字段信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    /// 列序号（从0开始）
    pub cid: i32,
    /// 列名
    pub name: String,
    /// 数据类型
    pub column_type: String,
    /// 是否可为空
    pub notnull: bool,
    /// 默认值
    pub dflt_value: Option<String>,
    /// 是否为主键
    pub pk: bool,
}

/// 表结构信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStructure {
    /// 表名
    pub name: String,
    /// 字段列表
    pub columns: Vec<ColumnInfo>,
    /// 行数
    pub row_count: i64,
}

/// 数据库完整结构探索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStructure {
    /// 数据库路径
    pub db_path: String,
    /// 文件大小（字节）
    pub file_size: u64,
    /// 总表数
    pub total_tables: usize,
    /// 所有表名
    pub all_tables: Vec<String>,
    /// 表结构详情
    pub table_structures: Vec<TableStructure>,
}

/// Zotero 数据库关键表（必须存在）
const ZOTERO_REQUIRED_TABLES: &[&str] = &[
    "items",
    "collections",
    "tags",
];

/// Zotero 数据库可选但推荐的表
const ZOTERO_OPTIONAL_TABLES: &[&str] = &[
    "itemCreators",
    "creators",
    "itemTags",
    "itemTypes",
    "deletedItems",
    "settings",
];

/// 数据库验证错误类型
#[derive(Debug)]
pub enum DbValidationError {
    /// 数据库文件不存在
    NotFound(PathBuf),
    /// 无法读取表列表
    QueryFailed(String),
    /// 缺少必需的表
    MissingTables(Vec<&'static str>, Vec<String>),
    /// 数据库文件损坏或不是有效的 SQLite 数据库
    InvalidDatabase(String),
}

/// 获取数据库验证失败的友好提示（简体中文）
pub fn get_db_validation_message(
    missing_tables: &[&str],
    available_tables: &[String],
) -> String {
    let missing_str = if missing_tables.is_empty() {
        "无".to_string()
    } else {
        missing_tables.join(", ")
    };

    let available_str = if available_tables.is_empty() {
        "无".to_string()
    } else {
        // 限制显示的表数量，避免消息过长
        let count = available_tables.len();
        let display: Vec<String> = if count > 20 {
            let mut v: Vec<String> = available_tables[..20].to_vec();
            v.push(format!("... (共 {} 个表)", count));
            v
        } else {
            available_tables.to_vec()
        };
        display.join(", ")
    };

    format!(
        "检测到的数据库缺少必要的表: {}\n\n\
        可用表: {}\n\n\
        请确保：\n\
        - 已正确安装并运行最新版 Zotero\n\
        - 没有使用损坏或备份不完整的数据库文件\n\
        - 可以在设置中手动选择正确的 Zotero 数据库",
        missing_str, available_str
    )
}

impl std::fmt::Display for DbValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbValidationError::NotFound(path) => {
                write!(f, "Zotero 数据库文件不存在: {:?}", path)
            }
            DbValidationError::QueryFailed(msg) => {
                write!(f, "无法读取数据库表信息: {}", msg)
            }
            DbValidationError::MissingTables(missing, _) => {
                write!(f, "数据库缺少必要的表: {}", missing.join(", "))
            }
            DbValidationError::InvalidDatabase(msg) => {
                write!(f, "数据库文件无效或已损坏: {}", msg)
            }
        }
    }
}

impl std::error::Error for DbValidationError {}

/// 验证数据库文件是否是有效的 SQLite 数据库
///
/// # 参数
/// * `db_path` - 数据库文件路径
///
/// # 返回值
/// * `Result<(), DbValidationError>` - 验证成功返回 Ok
pub fn validate_sqlite_file(db_path: &PathBuf) -> Result<(), DbValidationError> {
    if !db_path.exists() {
        return Err(DbValidationError::NotFound(db_path.clone()));
    }

    // 尝试打开数据库连接，如果失败说明不是有效的 SQLite 数据库
    Connection::open(db_path)
        .map_err(|e| DbValidationError::InvalidDatabase(e.to_string()))?;

    Ok(())
}

/// 获取数据库中所有表名
///
/// # 参数
/// * `conn` - 数据库连接
///
/// # 返回值
/// * `Result<Vec<String>, DbValidationError>` - 表名列表
pub fn get_all_table_names(conn: &Connection) -> Result<Vec<String>, DbValidationError> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .map_err(|e| DbValidationError::QueryFailed(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| DbValidationError::QueryFailed(e.to_string()))?;

    let mut tables = Vec::new();
    for row_result in rows {
        match row_result {
            Ok(table_name) => tables.push(table_name),
            Err(e) => eprintln!("[数据库验证] 读取表名失败: {}", e),
        }
    }

    Ok(tables)
}

/// 验证 Zotero 数据库完整性
///
/// # 参数
/// * `conn` - 数据库连接
///
/// # 返回值
/// * `Result<Vec<String>, DbValidationError>` - 成功时返回所有表名列表
///
/// # 验证规则
/// 1. 检查必需表是否存在（items, collections, tags）
/// 2. 如果缺少必需表，返回错误并列出缺失的表
/// 3. 可选表（itemAuthors, creators 等）缺失不影响验证通过
pub fn validate_zotero_database(conn: &Connection) -> Result<Vec<String>, DbValidationError> {
    let tables = get_all_table_names(conn)?;

    eprintln!("[数据库验证] 检测到 {} 个表", tables.len());

    // 检查必需表是否存在
    let missing: Vec<&'static str> = ZOTERO_REQUIRED_TABLES
        .iter()
        .filter(|t| !tables.iter().any(|name| name == *t))
        .copied()
        .collect();

    if !missing.is_empty() {
        eprintln!("[数据库验证] 缺少必需表: {:?}", missing);
        return Err(DbValidationError::MissingTables(missing, tables));
    }

    // 检查可选表
    let missing_optional: Vec<&'static str> = ZOTERO_OPTIONAL_TABLES
        .iter()
        .filter(|t| !tables.iter().any(|name| name == *t))
        .copied()
        .collect();

    if !missing_optional.is_empty() {
        eprintln!(
            "[数据库验证] 缺少可选表（可能影响部分功能）: {:?}",
            missing_optional
        );
    }

    eprintln!("[数据库验证] 数据库验证通过");
    Ok(tables)
}

/// 验证数据库并返回详细诊断信息
pub fn diagnose_database(conn: &Connection) -> DatabaseDiagnosis {
    let tables = get_all_table_names(conn).unwrap_or_default();

    let required_present: Vec<&str> = ZOTERO_REQUIRED_TABLES
        .iter()
        .filter(|t| tables.iter().any(|name| name == *t))
        .copied()
        .collect();

    let required_missing: Vec<&str> = ZOTERO_REQUIRED_TABLES
        .iter()
        .filter(|t| !tables.iter().any(|name| name == *t))
        .copied()
        .collect();

    let optional_present: Vec<&str> = ZOTERO_OPTIONAL_TABLES
        .iter()
        .filter(|t| tables.iter().any(|name| name == *t))
        .copied()
        .collect();

    let optional_missing: Vec<&str> = ZOTERO_OPTIONAL_TABLES
        .iter()
        .filter(|t| !tables.iter().any(|name| name == *t))
        .copied()
        .collect();

    DatabaseDiagnosis {
        total_tables: tables.len(),
        required_present,
        required_missing,
        optional_present,
        optional_missing,
        all_tables: tables,
    }
}

/// 探索数据库完整结构（用于生成文档）
///
/// # 参数
/// * `conn` - 数据库连接
/// * `db_path` - 数据库文件路径
///
/// # 返回值
/// * `Result<DatabaseStructure, DbValidationError>` - 完整的数据库结构信息
pub fn explore_database_structure(
    conn: &Connection,
    db_path: &PathBuf,
) -> Result<DatabaseStructure, DbValidationError> {
    use std::fs;

    // 获取文件大小
    let file_size = fs::metadata(db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // 获取所有表名
    let all_tables = get_all_table_names(conn)?;

    // 获取每个表的结构
    let mut table_structures = Vec::new();
    for table_name in &all_tables {
        let columns = get_table_columns(conn, table_name)?;
        let row_count = get_table_row_count(conn, table_name)?;
        table_structures.push(TableStructure {
            name: table_name.clone(),
            columns,
            row_count,
        });
    }

    Ok(DatabaseStructure {
        db_path: db_path.to_string_lossy().to_string(),
        file_size,
        total_tables: all_tables.len(),
        all_tables,
        table_structures,
    })
}

/// 获取指定表的字段信息
fn get_table_columns(conn: &Connection, table_name: &str) -> Result<Vec<ColumnInfo>, DbValidationError> {
    let sql = format!("PRAGMA table_info({})", table_name);
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| DbValidationError::QueryFailed(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ColumnInfo {
                cid: row.get(0)?,
                name: row.get(1)?,
                column_type: row.get(2)?,
                notnull: row.get::<_, i32>(3)? != 0,
                dflt_value: row.get(4)?,
                pk: row.get::<_, i32>(5)? != 0,
            })
        })
        .map_err(|e| DbValidationError::QueryFailed(e.to_string()))?;

    let mut columns = Vec::new();
    for row_result in rows {
        match row_result {
            Ok(col) => columns.push(col),
            Err(e) => eprintln!("[数据库验证] 读取表 {} 字段失败: {}", table_name, e),
        }
    }

    Ok(columns)
}

/// 获取指定表的行数
fn get_table_row_count(conn: &Connection, table_name: &str) -> Result<i64, DbValidationError> {
    let sql = format!("SELECT COUNT(*) FROM {}", table_name);
    conn.query_row(&sql, [], |row| row.get(0))
        .map_err(|e| DbValidationError::QueryFailed(e.to_string()))
}

/// 数据库诊断结果结构体
#[derive(Debug, serde::Serialize)]
pub struct DatabaseDiagnosis {
    /// 总表数
    pub total_tables: usize,
    /// 存在的必需表
    pub required_present: Vec<&'static str>,
    /// 缺失的必需表
    pub required_missing: Vec<&'static str>,
    /// 存在的可选表
    pub optional_present: Vec<&'static str>,
    /// 缺失的可选表
    pub optional_missing: Vec<&'static str>,
    /// 所有表名
    pub all_tables: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_message_format() {
        let missing = vec!["itemCreators", "creators"];
        let available = vec!["items".to_string(), "collections".to_string(), "tags".to_string()];
        let msg = get_db_validation_message(&missing, &available);
        assert!(msg.contains("itemCreators"));
        assert!(msg.contains("creators"));
        assert!(msg.contains("items"));
    }
}

/// 运行数据库结构探索测试（用于生成文档）
/// 通过 cargo test -- --nocapture 运行
#[test]
fn test_explore_database_structure_output() {
    use std::path::PathBuf;

    // 设置数据库路径
    let db_path = PathBuf::from(r"D:\Zotero\Date-Directary\zotero.sqlite");

    if !db_path.exists() {
        eprintln!("[测试] 数据库文件不存在: {:?}", db_path);
        return;
    }

    eprintln!("[测试] 开始探索数据库结构...");

    let conn = rusqlite::Connection::open(&db_path).expect("无法打开数据库");

    let result = explore_database_structure(&conn, &db_path);
    match result {
        Ok(structure) => {
            eprintln!("\n========================================");
            eprintln!("数据库路径: {}", structure.db_path);
            eprintln!("文件大小: {} bytes ({:.2} MB)", structure.file_size, structure.file_size as f64 / 1024.0 / 1024.0);
            eprintln!("总表数: {}", structure.total_tables);
            eprintln!("========================================\n");

            for table in &structure.table_structures {
                eprintln!("\n### {} 表 ({} 行)", table.name, table.row_count);
                eprintln!("| 序号 | 字段名 | 类型 | 非空 | 默认值 | 主键 |");
                eprintln!("| ---- | ------ | ---- | ---- | ------ | ---- |");
                for col in &table.columns {
                    let notnull_str = if col.notnull { "是" } else { "否" };
                    let pk_str = if col.pk { "是" } else { "否" };
                    let dflt = col.dflt_value.as_ref().map(|s| s.as_str()).unwrap_or("-");
                    eprintln!("| {} | {} | {} | {} | {} | {} |", col.cid, col.name, col.column_type, notnull_str, dflt, pk_str);
                }
            }

            eprintln!("\n========================================");
            eprintln!("JSON 输出:");
            eprintln!("========================================");
            let json = serde_json::to_string_pretty(&structure).expect("序列化失败");
            println!("{}", json);
        }
        Err(e) => {
            eprintln!("[测试] 探索失败: {:?}", e);
        }
    }
}