//! Zotero 数据库连接管理模块
//!
//! 本模块实现数据库单例连接模式，避免重复创建连接。
//! 所有操作均为只读，严格禁止修改 Zotero 原生数据。

use rusqlite::{Connection, Params, Row};
use std::path::PathBuf;
use std::sync::Mutex;

use super::path::get_zotero_database_path;
use super::validation::{diagnose_database, validate_zotero_database, DatabaseDiagnosis};

/// 全局数据库连接单例（使用 lazy_static 模式）
/// 使用 Mutex 确保线程安全，首次调用时初始化
static DB_CONNECTION: Mutex<Option<Connection>> = Mutex::new(None);

/// 全局数据库路径缓存
static DB_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

/// 数据库连接错误类型
#[derive(Debug)]
pub enum DbError {
    /// 数据库文件不存在
    NotFound(PathBuf),
    /// 数据库连接失败
    ConnectionFailed(String),
    /// 数据库查询失败
    QueryFailed(String),
    /// 连接锁定失败
    LockFailed,
    /// 数据库验证失败
    ValidationFailed(String),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::NotFound(path) => {
                write!(f, "Zotero 数据库文件不存在: {:?}", path)
            }
            DbError::ConnectionFailed(msg) => {
                write!(f, "数据库连接失败: {}", msg)
            }
            DbError::QueryFailed(msg) => {
                write!(f, "数据库查询失败: {}", msg)
            }
            DbError::LockFailed => {
                write!(f, "无法锁定数据库连接")
            }
            DbError::ValidationFailed(msg) => {
                write!(f, "数据库验证失败: {}", msg)
            }
        }
    }
}

impl std::error::Error for DbError {}

impl From<rusqlite::Error> for DbError {
    fn from(err: rusqlite::Error) -> Self {
        DbError::QueryFailed(err.to_string())
    }
}

/// 获取当前缓存的数据库路径
///
/// # 返回值
/// * `Option<PathBuf>` - 当前数据库路径（如果已初始化）
pub fn get_current_db_path() -> Option<PathBuf> {
    DB_PATH.lock().ok().and_then(|guard| guard.clone())
}

/// 获取数据库诊断信息
///
/// # 返回值
/// * `Result<DatabaseDiagnosis, DbError>` - 诊断信息
pub fn get_database_diagnosis() -> Result<DatabaseDiagnosis, DbError> {
    let guard = get_connection()?;
    let conn = guard.as_ref().ok_or_else(|| {
        DbError::ConnectionFailed("数据库连接未初始化".to_string())
    })?;

    Ok(diagnose_database(conn))
}

/// 重置数据库连接（用于切换数据库路径后）
pub fn reset_connection() {
    if let Ok(mut guard) = DB_CONNECTION.lock() {
        *guard = None;
        eprintln!("[数据库] 连接已重置");
    }
    if let Ok(mut path_guard) = DB_PATH.lock() {
        *path_guard = None;
        eprintln!("[数据库] 路径缓存已清除");
    }
}

/// 获取数据库连接单例
///
/// # 返回值
/// * `Result<MutexGuard<'static, Option<Connection>>, DbError>` - 数据库连接锁
///
/// # 错误处理
/// * 如果数据库文件不存在，返回 DbError::NotFound
/// * 如果连接创建失败，返回 DbError::ConnectionFailed
/// * 如果数据库验证失败，返回 DbError::ValidationFailed
pub fn get_connection() -> Result<MutexGuard<'static, Option<Connection>>, DbError> {
    let mut guard = DB_CONNECTION.lock().map_err(|_| {
        eprintln!("[数据库] 获取连接锁失败: 可能有其他进程正在访问数据库");
        DbError::LockFailed
    })?;

    if guard.is_none() {
        let db_path = match get_zotero_database_path() {
            Some(path) => path,
            None => {
                eprintln!("[数据库] 无法检测到 Zotero 数据库路径");
                return Err(DbError::NotFound(PathBuf::new()));
            }
        };

        if !db_path.exists() {
            eprintln!("[数据库] Zotero 数据库文件不存在: {:?}", db_path);
            return Err(DbError::NotFound(db_path));
        }

        eprintln!("[数据库] 正在打开 Zotero 数据库: {:?}", db_path);

        // 以只读模式打开数据库连接
        let connection = Connection::open(&db_path).map_err(|e| {
            eprintln!("[数据库] 连接打开失败: {:?}", e);
            DbError::ConnectionFailed(e.to_string())
        })?;

        // 设置只读事务模式，确保数据安全
        connection.execute_batch("PRAGMA query_only = ON;").map_err(|e| {
            eprintln!("[数据库] 设置只读模式失败: {:?}", e);
            DbError::ConnectionFailed(e.to_string())
        })?;

        // 验证数据库完整性
        match validate_zotero_database(&connection) {
            Ok(tables) => {
                eprintln!("[数据库] 数据库验证通过，共有 {} 个表", tables.len());
            }
            Err(e) => {
                eprintln!("[数据库] 数据库验证失败: {:?}", e);
                return Err(DbError::ValidationFailed(e.to_string()));
            }
        }

        // 缓存数据库路径
        if let Ok(mut path_guard) = DB_PATH.lock() {
            *path_guard = Some(db_path.clone());
        }

        eprintln!("[数据库] 数据库连接成功");
        *guard = Some(connection);
    }

    Ok(guard)
}

/// 使用外部传入的连接执行只读查询并映射结果（避免重入死锁）
///
/// # 参数
/// * `conn` - 数据库连接（由调用方保证有效）
/// * `sql` - SQL 查询语句
/// * `params` - 查询参数
/// * `mapper` - 行映射函数
///
/// # 返回值
/// * `Result<Vec<T>, DbError>` - 映射后的结果列表
pub fn query_with_mapper_on_connection<T, P>(
    conn: &Connection,
    sql: &str,
    params: P,
    mapper: impl FnMut(&Row<'_>) -> Result<T, rusqlite::Error>,
) -> Result<Vec<T>, DbError>
where
    P: Params,
{
    let mut stmt = conn.prepare(sql).map_err(|e| {
        eprintln!("[数据库] SQL 预处理失败: {:?}", e);
        DbError::QueryFailed(e.to_string())
    })?;
    let rows = stmt.query_map(params, mapper).map_err(|e| {
        eprintln!("[数据库] 查询执行失败: {:?}", e);
        DbError::QueryFailed(e.to_string())
    })?;

    let mut result = Vec::new();
    for row_result in rows {
        result.push(row_result?);
    }

    Ok(result)
}

/// 使用外部传入的连接执行只读查询（无参数）并映射结果（避免重入死锁）
///
/// # 参数
/// * `conn` - 数据库连接（由调用方保证有效）
/// * `sql` - SQL 查询语句
/// * `mapper` - 行映射函数
///
/// # 返回值
/// * `Result<Vec<T>, DbError>` - 映射后的结果列表
pub fn query_no_params_on_connection<T>(
    conn: &Connection,
    sql: &str,
    mut mapper: impl FnMut(&Row<'_>) -> Result<T, rusqlite::Error>,
) -> Result<Vec<T>, DbError> {
    let mut stmt = conn.prepare(sql).map_err(|e| {
        eprintln!("[数据库] SQL 预处理失败: {:?}", e);
        DbError::QueryFailed(e.to_string())
    })?;
    let rows = stmt.query_map([], &mut mapper).map_err(|e| {
        eprintln!("[数据库] 查询执行失败: {:?}", e);
        DbError::QueryFailed(e.to_string())
    })?;

    let mut result = Vec::new();
    for row_result in rows {
        result.push(row_result?);
    }

    Ok(result)
}

/// 类型别名：Mutex锁守卫
pub type MutexGuard<'a, T> = std::sync::MutexGuard<'a, T>;
