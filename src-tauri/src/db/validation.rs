//! Zotero 数据库完整性验证模块
//!
//! 本模块提供数据库表完整性检查功能，确保使用的数据库是有效的 Zotero 数据库。
//! 验证标准 Zotero 表结构，防止因数据库不完整导致的查询失败。

use rusqlite::Connection;
use std::path::PathBuf;

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
///
/// # 参数
/// * `conn` - 数据库连接
///
/// # 返回值
/// * `DatabaseDiagnosis` - 诊断信息结构体
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