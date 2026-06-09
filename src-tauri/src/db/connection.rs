//! Zotero 数据库连接管理模块
//!
//! 本模块实现数据库单例连接模式，避免重复创建连接。
//! 所有操作均为只读，严格禁止修改 Zotero 原生数据。

use rusqlite::{Connection, Params, Row};
use std::path::PathBuf;
use std::sync::Mutex;

use super::path::get_zotero_database_path;

/// 全局数据库连接单例（使用 lazy_static 模式）
/// 使用 Mutex 确保线程安全，首次调用时初始化
static DB_CONNECTION: Mutex<Option<Connection>> = Mutex::new(None);

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
        }
    }
}

impl std::error::Error for DbError {}

impl From<rusqlite::Error> for DbError {
    fn from(err: rusqlite::Error) -> Self {
        DbError::QueryFailed(err.to_string())
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
pub fn get_connection() -> Result<MutexGuard<'static, Option<Connection>>, DbError> {
    let mut guard = DB_CONNECTION.lock().map_err(|_| DbError::LockFailed)?;

    if guard.is_none() {
        let db_path = match get_zotero_database_path() {
            Some(path) => path,
            None => {
                return Err(DbError::NotFound(PathBuf::new()));
            }
        };

        if !db_path.exists() {
            return Err(DbError::NotFound(db_path));
        }

        // 以只读模式打开数据库连接
        let connection = Connection::open(&db_path)
            .map_err(|e: rusqlite::Error| DbError::ConnectionFailed(e.to_string()))?;

        // 设置只读事务模式，确保数据安全
        connection
            .execute_batch("PRAGMA query_only = ON;")
            .map_err(|e: rusqlite::Error| DbError::ConnectionFailed(e.to_string()))?;

        *guard = Some(connection);
    }

    // 将 guard 提升为 'static 生命周期
    // 由于 DB_CONNECTION 是 static，锁的生命周期可以安全地提升
    let static_guard: MutexGuard<'static, Option<Connection>> = unsafe {
        let ptr = &DB_CONNECTION as *const Mutex<Option<Connection>>;
        let static_ref: &'static Mutex<Option<Connection>> = &*ptr;
        static_ref.lock().map_err(|_| DbError::LockFailed)?
    };
    Ok(static_guard)
}

/// 执行只读查询并映射结果
///
/// # 参数
/// * `sql` - SQL 查询语句
/// * `params` - 查询参数
/// * `mapper` - 行映射函数
///
/// # 返回值
/// * `Result<Vec<T>, DbError>` - 映射后的结果列表
pub fn query_with_mapper<T, P>(
    sql: &str,
    params: P,
    mapper: impl FnMut(&Row<'_>) -> Result<T, rusqlite::Error>,
) -> Result<Vec<T>, DbError>
where
    P: Params,
{
    let guard = get_connection()?;
    let conn = guard.as_ref().ok_or_else(|| {
        DbError::ConnectionFailed("数据库连接未初始化".to_string())
    })?;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params, mapper)?;

    let mut result = Vec::new();
    for row_result in rows {
        result.push(row_result?);
    }

    Ok(result)
}

/// 执行只读查询（无参数）并映射结果
///
/// # 参数
/// * `sql` - SQL 查询语句
/// * `mapper` - 行映射函数
///
/// # 返回值
/// * `Result<Vec<T>, DbError>` - 映射后的结果列表
pub fn query_no_params<T>(
    sql: &str,
    mut mapper: impl FnMut(&Row<'_>) -> Result<T, rusqlite::Error>,
) -> Result<Vec<T>, DbError> {
    let guard = get_connection()?;
    let conn = guard.as_ref().ok_or_else(|| {
        DbError::ConnectionFailed("数据库连接未初始化".to_string())
    })?;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], &mut mapper)?;

    let mut result = Vec::new();
    for row_result in rows {
        result.push(row_result?);
    }

    Ok(result)
}

/// 类型别名：Mutex锁守卫
pub type MutexGuard<'a, T> = std::sync::MutexGuard<'a, T>;