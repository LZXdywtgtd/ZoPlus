//! 文献信息查询模块
//!
//! 本模块提供文献基本信息的只读查询功能。
//! 查询数据包括：文献ID、标题、合并作者（分号分隔）、发表年份

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, Duration};

use super::connection::{query_no_params_on_connection, query_with_mapper_on_connection, DbError};
use super::metadata::get_cached_metadata;
use super::dynamic::{DynamicSqlBuilder, ZoteroTableCandidates};

/// 查询超时时间（秒）
const QUERY_TIMEOUT_SECS: u64 = 10;

/// 动态构建文献查询 SQL
///
/// # 参数
/// * `conn` - 数据库连接
/// * `use_limit` - 是否添加 LIMIT 子句
/// * `item_id_filter` - 可选的单条文献 ID 过滤
///
/// # 返回值
/// * `String` - 动态构建的 SQL 查询语句
fn build_items_sql(conn: &Connection, use_limit: bool, item_id_filter: Option<i32>) -> Result<String, DbError> {
    let metadata = get_cached_metadata(conn)
        .map_err(|e| DbError::QueryFailed(format!("获取元数据失败: {}", e)))?;
    let dynamic = DynamicSqlBuilder::new(&metadata);

    // 动态获取表名
    let items_table = dynamic.find_table(ZoteroTableCandidates::ITEMS)
        .ok_or_else(|| DbError::QueryFailed("未找到 items 表".to_string()))?;
    let item_data_table = dynamic.find_table(ZoteroTableCandidates::ITEM_DATA)
        .ok_or_else(|| DbError::QueryFailed("未找到 itemData 表".to_string()))?;
    let item_data_values_table = dynamic.find_table(ZoteroTableCandidates::ITEM_DATA_VALUES)
        .ok_or_else(|| DbError::QueryFailed("未找到 itemDataValues 表".to_string()))?;
    let fields_table = dynamic.find_table(ZoteroTableCandidates::FIELDS)
        .ok_or_else(|| DbError::QueryFailed("未找到 fields 表".to_string()))?;
    let authors_table = dynamic.find_table(ZoteroTableCandidates::CREATORS)
        .ok_or_else(|| DbError::QueryFailed("未找到 itemCreators 表".to_string()))?;
    let creators_table = dynamic.find_table(ZoteroTableCandidates::CREATOR)
        .ok_or_else(|| DbError::QueryFailed("未找到 creators 表".to_string()))?;

    // 构建动态 SQL
    let sql = format!(
        r#"
        SELECT
            i.itemID as item_id,
            fv_title.value as title,
            fv_date.value as year,
            (
                SELECT GROUP_CONCAT(
                    COALESCE(c.lastName, '') || COALESCE(c.firstName, ''),
                    '; '
                )
                FROM {authors_table} ia
                JOIN {creators_table} c ON ia.creatorID = c.creatorID
                WHERE ia.itemID = i.itemID
                ORDER BY ia.orderIndex
            ) as authors
        FROM {items_table} i
        LEFT JOIN {item_data_table} id_title ON i.itemID = id_title.itemID
            AND id_title.fieldID = (SELECT fieldID FROM {fields_table} WHERE fieldName = 'title')
        LEFT JOIN {item_data_values_table} fv_title ON id_title.valueID = fv_title.valueID
        LEFT JOIN {item_data_table} id_date ON i.itemID = id_date.itemID
            AND id_date.fieldID = (SELECT fieldID FROM {fields_table} WHERE fieldName = 'date')
        LEFT JOIN {item_data_values_table} fv_date ON id_date.valueID = fv_date.valueID
        "#,
        items_table = items_table,
        item_data_table = item_data_table,
        item_data_values_table = item_data_values_table,
        fields_table = fields_table,
        authors_table = authors_table,
        creators_table = creators_table
    );

    let mut result = sql;
    if let Some(item_id) = item_id_filter {
        result.push_str(&format!(" WHERE i.itemID = {}", item_id));
    } else {
        result.push_str(" WHERE i.itemID IS NOT NULL");
    }
    result.push_str(" ORDER BY fv_date.value DESC, fv_title.value ASC");
    if use_limit {
        result.push_str(" LIMIT 100");
    }

    Ok(result)
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

/// 同步获取所有文献的基本信息（用于 indexer 等同步模块）
///
/// 此函数不使用 spawn_blocking 和超时控制，直接在当前线程执行查询。
///
/// # 返回值
/// * `Result<Vec<ItemInfo>, DbError>` - 文献信息列表
pub fn get_all_items() -> Result<Vec<ItemInfo>, DbError> {
    let start = std::time::Instant::now();
    eprintln!("[文献查询] 开始查询文献列表（同步模式）...");

    // 直接获取连接，在当前线程执行查询
    let guard = super::connection::get_connection()?;
    let conn = guard.as_ref().ok_or_else(|| {
        DbError::ConnectionFailed("数据库连接未初始化".to_string())
    })?;

    // 检查文献数量（诊断用）
    let metadata = get_cached_metadata(conn)
        .map_err(|e| DbError::QueryFailed(format!("获取元数据失败: {}", e)))?;
    eprintln!("[文献查询] 数据库元数据已加载，包含 {} 个表", metadata.table_count());

    let sql = build_items_sql(conn, true, None)?;

    let result = query_no_params_on_connection(conn, &sql, |row| {
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

/// 异步获取所有文献的基本信息（带分页限制）
///
/// # 返回值
/// * `Result<Vec<ItemInfo>, DbError>` - 文献信息列表
///
/// # 查询逻辑
/// 1. 通过动态元数据系统获取表名
/// 2. 从 items 表查询 itemID、title、date
/// 3. 通过 itemCreators 表关联 creators 表获取作者信息
/// 4. 多个作者按 orderIndex 排序后用分号合并
/// 5. 默认限制返回 100 条记录，避免查询过慢
pub async fn get_all_items_async() -> Result<Vec<ItemInfo>, DbError> {
    let start = std::time::Instant::now();
    eprintln!("[文献查询] 开始查询文献列表...");

    let timeout_duration = Duration::from_secs(QUERY_TIMEOUT_SECS);

    // 在闭包内部获取连接，避免 MutexGuard 跨 await 死锁
    let result = timeout(
        timeout_duration,
        tokio::task::spawn_blocking(|| {
            let guard = super::connection::get_connection()?;
            let conn = guard.as_ref().ok_or_else(|| {
                DbError::ConnectionFailed("数据库连接未初始化".to_string())
            })?;

            // 动态获取元数据
            let metadata = get_cached_metadata(conn)
                .map_err(|e| DbError::QueryFailed(format!("获取元数据失败: {}", e)))?;
            eprintln!("[文献查询] 数据库元数据已加载，包含 {} 个表", metadata.table_count());

            let sql = build_items_sql(conn, true, None)?;

            query_no_params_on_connection(conn, &sql, |row| {
                Ok(ItemInfo {
                    item_id: row.get(0)?,
                    title: row.get::<_, String>(1).unwrap_or_default(),
                    year: row.get::<_, String>(2).unwrap_or_default(),
                    authors: row.get::<_, String>(3).unwrap_or_default(),
                })
            })
        }),
    )
    .await
    .map_err(|e| {
        eprintln!("[文献查询] 查询超时（{}秒）: {:?}", QUERY_TIMEOUT_SECS, e);
        DbError::QueryFailed(format!("查询超时（{}秒），请检查数据库是否被其他程序占用", QUERY_TIMEOUT_SECS))
    })?
    .map_err(|e| {
        eprintln!("[文献查询] spawn_blocking 执行失败: {:?}", e);
        DbError::QueryFailed(format!("spawn_blocking 执行失败: {}", e))
    })?;

    let elapsed = start.elapsed();
    eprintln!(
        "[文献查询] 查询完成，返回 {} 条记录，耗时: {:?}",
        result.as_ref().map(|v| v.len()).unwrap_or(0),
        elapsed
    );

    result
}

/// 根据文献ID获取单条文献信息（异步）
///
/// # 参数
/// * `item_id` - 文献ID
///
/// # 返回值
/// * `Result<Option<ItemInfo>, DbError>` - 文献信息（不存在时返回 None）
pub async fn get_item_by_id_async(item_id: i32) -> Result<Option<ItemInfo>, DbError> {
    let timeout_duration = Duration::from_secs(QUERY_TIMEOUT_SECS);

    // 在闭包内部获取连接，避免重入死锁
    timeout(
        timeout_duration,
        tokio::task::spawn_blocking(move || {
            let guard = super::connection::get_connection()?;
            let conn = guard.as_ref().ok_or_else(|| {
                DbError::ConnectionFailed("数据库连接未初始化".to_string())
            })?;

            // 动态获取元数据
            let metadata = get_cached_metadata(conn)
                .map_err(|e| DbError::QueryFailed(format!("获取元数据失败: {}", e)))?;
            let dynamic = DynamicSqlBuilder::new(&metadata);

            // 获取表名用于日志
            let author_table = dynamic.find_table(ZoteroTableCandidates::CREATORS)
                .unwrap_or("itemCreators");
            eprintln!("[文献查询] 根据ID查询文献: item_id={}, 作者表={}", item_id, author_table);

            // 动态构建 SQL（不使用 WHERE i.itemID = ?，而是在 SQL 末尾添加条件）
            let base_sql = build_items_sql(conn, false, None)?;
            let sql = format!("{} AND i.itemID = ?", base_sql);

            let results = query_with_mapper_on_connection(conn, &sql, params![item_id], |row| {
                Ok(ItemInfo {
                    item_id: row.get(0)?,
                    title: row.get::<_, String>(1).unwrap_or_default(),
                    year: row.get::<_, String>(2).unwrap_or_default(),
                    authors: row.get::<_, String>(3).unwrap_or_default(),
                })
            })?;

            Ok(results.into_iter().next())
        }),
    )
    .await
    .map_err(|e| {
        eprintln!("[文献查询] 查询超时（{}秒）: {:?}", QUERY_TIMEOUT_SECS, e);
        DbError::QueryFailed(format!("查询超时（{}秒），请检查数据库是否被其他程序占用", QUERY_TIMEOUT_SECS))
    })?
    .map_err(|e| {
        eprintln!("[文献查询] spawn_blocking 执行失败: {:?}", e);
        DbError::QueryFailed(format!("spawn_blocking 执行失败: {}", e))
    })?
}

/// 分页获取文献列表（异步）
///
/// # 参数
/// * `offset` - 跳过记录数
/// * `limit` - 返回记录数上限
///
/// # 返回值
/// * `Result<Vec<ItemInfo>, DbError>> - 文献信息列表
pub async fn get_items_paginated_async(offset: i32, limit: i32) -> Result<Vec<ItemInfo>, DbError> {
    let start = std::time::Instant::now();
    eprintln!(
        "[文献查询] 开始分页查询文献列表: offset={}, limit={}",
        offset, limit
    );

    let timeout_duration = Duration::from_secs(QUERY_TIMEOUT_SECS);

    // 在闭包内部获取连接，避免重入死锁
    let result = timeout(
        timeout_duration,
        tokio::task::spawn_blocking(move || {
            let guard = super::connection::get_connection()?;
            let conn = guard.as_ref().ok_or_else(|| {
                DbError::ConnectionFailed("数据库连接未初始化".to_string())
            })?;

            // 动态获取元数据
            let metadata = get_cached_metadata(conn)
                .map_err(|e| DbError::QueryFailed(format!("获取元数据失败: {}", e)))?;
            let dynamic = DynamicSqlBuilder::new(&metadata);

            // 获取表名用于日志
            let author_table = dynamic.find_table(ZoteroTableCandidates::CREATORS)
                .unwrap_or("itemCreators");
            eprintln!("[文献查询] 使用的作者表: {}", author_table);

            // 动态构建分页 SQL
            let base_sql = build_items_sql(conn, false, None)?;
            let sql = format!("{}\nLIMIT ? OFFSET ?", base_sql);

            query_with_mapper_on_connection(conn, &sql, params![limit, offset], |row| {
                Ok(ItemInfo {
                    item_id: row.get(0)?,
                    title: row.get::<_, String>(1).unwrap_or_default(),
                    year: row.get::<_, String>(2).unwrap_or_default(),
                    authors: row.get::<_, String>(3).unwrap_or_default(),
                })
            })
        }),
    )
    .await
    .map_err(|e| {
        eprintln!("[文献查询] 查询超时（{}秒）: {:?}", QUERY_TIMEOUT_SECS, e);
        DbError::QueryFailed(format!("查询超时（{}秒），请检查数据库是否被其他程序占用", QUERY_TIMEOUT_SECS))
    })?
    .map_err(|e| {
        eprintln!("[文献查询] spawn_blocking 执行失败: {:?}", e);
        DbError::QueryFailed(format!("spawn_blocking 执行失败: {}", e))
    })?;

    let elapsed = start.elapsed();
    eprintln!(
        "[文献查询] 分页查询完成，返回 {} 条记录，耗时: {:?}",
        result.as_ref().map(|v| v.len()).unwrap_or(0),
        elapsed
    );

    result
}
