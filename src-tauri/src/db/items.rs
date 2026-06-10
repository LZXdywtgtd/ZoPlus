//! 文献信息查询模块
//!
//! 本模块提供文献基本信息的只读查询功能。
//! 查询数据包括：文献ID、标题、合并作者（分号分隔）、发表年份

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use super::connection::{query_no_params, query_with_mapper, DbError};

/// 作者关联表名缓存（避免每次查询都检测）
static AUTHOR_TABLE_NAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// 检测实际使用的作者关联表名
///
/// Zotero 数据库中作者关联表的表名可能是 itemCreators、itemAuthors 或 itemCreator
/// 本函数通过检测 sqlite_master 表来确定实际使用的表名
///
/// # 参数
/// * `conn` - 数据库连接
///
/// # 返回值
/// * `String` - 检测到的作者关联表名
fn detect_author_table_name(conn: &Connection) -> String {
    // 如果已经检测过，直接返回缓存值
    if let Some(cached) = AUTHOR_TABLE_NAME.get() {
        return cached.clone();
    }

    let candidates = ["itemCreators", "itemAuthors", "itemCreator"];
    for table in candidates {
        let exists = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?",
                [table],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if exists {
            eprintln!("[数据库] 检测到作者关联表: {}", table);
            let result = table.to_string();
            // 缓存检测结果
            let _ = AUTHOR_TABLE_NAME.set(result.clone());
            return result;
        }
    }
    // 默认值（标准 Zotero 表名）
    eprintln!("[数据库] 未检测到作者关联表，使用默认值: itemCreators");
    let default = "itemCreators".to_string();
    let _ = AUTHOR_TABLE_NAME.set(default.clone());
    default
}

/// 获取带作者信息的 SQL 查询语句
///
/// # 参数
/// * `author_table` - 作者关联表名
///
/// # 返回值
/// * `String` - 完整的 SQL 查询语句
fn build_items_sql(author_table: &str) -> String {
    format!(
        r#"
        SELECT
            i.itemID as item_id,
            title_data.value as title,
            date_data.value as year,
            (
                SELECT GROUP_CONCAT(
                    COALESCE(c.lastName, '') || COALESCE(c.firstName, ''),
                    '; '
                )
                FROM {} ia
                JOIN creators c ON ia.creatorID = c.creatorID
                WHERE ia.itemID = i.itemID
                ORDER BY ia.orderIndex
            ) as authors
        FROM items i
        LEFT JOIN itemData title_data ON i.itemID = title_data.itemID
            AND title_data.fieldID = (SELECT fieldID FROM itemDataFields WHERE fieldName = 'title')
        LEFT JOIN itemData date_data ON i.itemID = date_data.itemID
            AND date_data.fieldID = (SELECT fieldID FROM itemDataFields WHERE fieldName = 'date')
        WHERE i.itemID IS NOT NULL
        ORDER BY date_data.value DESC, title_data.value ASC
        LIMIT 100
        "#,
        author_table
    )
}

/// 获取带作者信息的分页 SQL 查询语句
///
/// # 参数
/// * `author_table` - 作者关联表名
///
/// # 返回值
/// * `String` - 完整的分页 SQL 查询语句
fn build_items_paginated_sql(author_table: &str) -> String {
    format!(
        r#"
        SELECT
            i.itemID as item_id,
            title_data.value as title,
            date_data.value as year,
            (
                SELECT GROUP_CONCAT(
                    COALESCE(c.lastName, '') || COALESCE(c.firstName, ''),
                    '; '
                )
                FROM {} ia
                JOIN creators c ON ia.creatorID = c.creatorID
                WHERE ia.itemID = i.itemID
                ORDER BY ia.orderIndex
            ) as authors
        FROM items i
        LEFT JOIN itemData title_data ON i.itemID = title_data.itemID
            AND title_data.fieldID = (SELECT fieldID FROM itemDataFields WHERE fieldName = 'title')
        LEFT JOIN itemData date_data ON i.itemID = date_data.itemID
            AND date_data.fieldID = (SELECT fieldID FROM itemDataFields WHERE fieldName = 'date')
        WHERE i.itemID IS NOT NULL
        ORDER BY date_data.value DESC, title_data.value ASC
        LIMIT ? OFFSET ?
        "#,
        author_table
    )
}

/// 获取单条文献信息的 SQL 查询语句
///
/// # 参数
/// * `author_table` - 作者关联表名
///
/// # 返回值
/// * `String` - 完整的 SQL 查询语句
fn build_item_by_id_sql(author_table: &str) -> String {
    format!(
        r#"
        SELECT
            i.itemID as item_id,
            title_data.value as title,
            date_data.value as year,
            (
                SELECT GROUP_CONCAT(
                    COALESCE(c.lastName, '') || COALESCE(c.firstName, ''),
                    '; '
                )
                FROM {} ia
                JOIN creators c ON ia.creatorID = c.creatorID
                WHERE ia.itemID = i.itemID
                ORDER BY ia.orderIndex
            ) as authors
        FROM items i
        LEFT JOIN itemData title_data ON i.itemID = title_data.itemID
            AND title_data.fieldID = (SELECT fieldID FROM itemDataFields WHERE fieldName = 'title')
        LEFT JOIN itemData date_data ON i.itemID = date_data.itemID
            AND date_data.fieldID = (SELECT fieldID FROM itemDataFields WHERE fieldName = 'date')
        WHERE i.itemID = ?
        "#,
        author_table
    )
}

/// 文献基本信息结构体
///
/// # 字段说明
/// * `item_id` - Zotero 数据库中的文献唯一标识符
/// * `title` - 文献标题
/// * `authors` - 合并后的作者字符串，多个作者用分号分隔
/// * `year` - 发表年份
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemInfo {
    /// 文献ID
    pub item_id: i32,
    /// 标题
    pub title: String,
    /// 合并作者（分号分隔）
    pub authors: String,
    /// 发表年份
    pub year: String,
}

/// 获取所有文献的基本信息（带分页限制）
///
/// # 返回值
/// * `Result<Vec<ItemInfo>, DbError>` - 文献信息列表
///
/// # 查询逻辑
/// 1. 检测数据库中实际的作者关联表名
/// 2. 从 items 表查询 itemID、title、date
/// 3. 通过 itemCreators 表关联 creators 表获取作者信息
/// 4. 多个作者按 orderIndex 排序后用分号合并
/// 5. 默认限制返回 100 条记录，避免查询过慢
pub fn get_all_items() -> Result<Vec<ItemInfo>, DbError> {
    let start = std::time::Instant::now();
    eprintln!("[文献查询] 开始查询文献列表...");

    // 获取数据库连接并检测作者表名
    let guard = super::connection::get_connection()?;
    let conn = guard.as_ref().ok_or_else(|| {
        DbError::ConnectionFailed("数据库连接未初始化".to_string())
    })?;
    let author_table = detect_author_table_name(conn);
    let sql = build_items_sql(&author_table);

    eprintln!("[文献查询] 使用的作者表: {}", author_table);
    eprintln!("[文献查询] SQL: {}", sql);

    let result = query_no_params(&sql, |row| {
        Ok(ItemInfo {
            item_id: row.get(0)?,
            title: row.get::<_, String>(1).unwrap_or_default(),
            year: row.get::<_, String>(2).unwrap_or_default(),
            authors: row.get::<_, String>(3).unwrap_or_default(),
        })
    });

    let elapsed = start.elapsed();
    eprintln!(
        "[文献查询] 查询完成，返回 {} 条记录，耗时: {:?}",
        result.as_ref().map(|v| v.len()).unwrap_or(0),
        elapsed
    );

    result
}

/// 根据文献ID获取单条文献信息
///
/// # 参数
/// * `item_id` - 文献ID
///
/// # 返回值
/// * `Result<Option<ItemInfo>, DbError>` - 文献信息（不存在时返回 None）
pub fn get_item_by_id(item_id: i32) -> Result<Option<ItemInfo>, DbError> {
    // 获取数据库连接并检测作者表名
    let guard = super::connection::get_connection()?;
    let conn = guard.as_ref().ok_or_else(|| {
        DbError::ConnectionFailed("数据库连接未初始化".to_string())
    })?;
    let author_table = detect_author_table_name(conn);
    let sql = build_item_by_id_sql(&author_table);

    eprintln!("[文献查询] 根据ID查询文献: item_id={}, 作者表={}", item_id, author_table);

    let results = query_with_mapper(&sql, params![item_id], |row| {
        Ok(ItemInfo {
            item_id: row.get(0)?,
            title: row.get::<_, String>(1).unwrap_or_default(),
            year: row.get::<_, String>(2).unwrap_or_default(),
            authors: row.get::<_, String>(3).unwrap_or_default(),
        })
    })?;

    Ok(results.into_iter().next())
}

/// 分页获取文献列表
///
/// # 参数
/// * `offset` - 跳过记录数
/// * `limit` - 返回记录数上限
///
/// # 返回值
/// * `Result<Vec<ItemInfo>, DbError>` - 文献信息列表
pub fn get_items_paginated(offset: i32, limit: i32) -> Result<Vec<ItemInfo>, DbError> {
    let start = std::time::Instant::now();
    eprintln!(
        "[文献查询] 开始分页查询文献列表: offset={}, limit={}",
        offset, limit
    );

    // 获取数据库连接并检测作者表名
    let guard = super::connection::get_connection()?;
    let conn = guard.as_ref().ok_or_else(|| {
        DbError::ConnectionFailed("数据库连接未初始化".to_string())
    })?;
    let author_table = detect_author_table_name(conn);
    let sql = build_items_paginated_sql(&author_table);

    eprintln!("[文献查询] 使用的作者表: {}", author_table);

    let result = query_with_mapper(&sql, params![limit, offset], |row| {
        Ok(ItemInfo {
            item_id: row.get(0)?,
            title: row.get::<_, String>(1).unwrap_or_default(),
            year: row.get::<_, String>(2).unwrap_or_default(),
            authors: row.get::<_, String>(3).unwrap_or_default(),
        })
    });

    let elapsed = start.elapsed();
    eprintln!(
        "[文献查询] 分页查询完成，返回 {} 条记录，耗时: {:?}",
        result.as_ref().map(|v| v.len()).unwrap_or(0),
        elapsed
    );

    result
}
