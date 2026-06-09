//! 文献信息查询模块
//!
//! 本模块提供文献基本信息的只读查询功能。
//! 查询数据包括：文献ID、标题、合并作者（分号分隔）、发表年份

use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::connection::{query_no_params, query_with_mapper, DbError};

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

/// 获取所有文献的基本信息
///
/// # 返回值
/// * `Result<Vec<ItemInfo>, DbError>` - 文献信息列表
///
/// # 查询逻辑
/// 1. 从 items 表查询 itemID、title、date
/// 2. 通过 itemAuthors 表关联 creators 表获取作者信息
/// 3. 多个作者按 order 排序后用分号合并
pub fn get_all_items() -> Result<Vec<ItemInfo>, DbError> {
    // SQL 查询：获取文献信息及作者
    // 子查询用于按 itemID 和 order 聚合作者姓名
    let sql = r#"
        SELECT
            i.itemID as item_id,
            i.title as title,
            i.date as year,
            (
                SELECT GROUP_CONCAT(
                    COALESCE(c.lastName, '') || COALESCE(c.firstName, ''),
                    '; '
                )
                FROM itemAuthors ia
                JOIN creators c ON ia.authorID = c.creatorID
                WHERE ia.itemID = i.itemID
                ORDER BY ia.order
            ) as authors
        FROM items i
        WHERE i.itemID IS NOT NULL
        ORDER BY i.date DESC, i.title ASC
    "#;

    query_no_params(sql, |row| {
        Ok(ItemInfo {
            item_id: row.get(0)?,
            title: row.get::<_, String>(1).unwrap_or_default(),
            year: row.get::<_, String>(2).unwrap_or_default(),
            authors: row.get::<_, String>(3).unwrap_or_default(),
        })
    })
}

/// 根据文献ID获取单条文献信息
///
/// # 参数
/// * `item_id` - 文献ID
///
/// # 返回值
/// * `Result<Option<ItemInfo>, DbError>` - 文献信息（不存在时返回 None）
pub fn get_item_by_id(item_id: i32) -> Result<Option<ItemInfo>, DbError> {
    let sql = r#"
        SELECT
            i.itemID as item_id,
            i.title as title,
            i.date as year,
            (
                SELECT GROUP_CONCAT(
                    COALESCE(c.lastName, '') || COALESCE(c.firstName, ''),
                    '; '
                )
                FROM itemAuthors ia
                JOIN creators c ON ia.authorID = c.creatorID
                WHERE ia.itemID = i.itemID
                ORDER BY ia.order
            ) as authors
        FROM items i
        WHERE i.itemID = ?
    "#;

    let results = query_with_mapper(sql, params![item_id], |row| {
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
pub fn get_items_paginated(
    offset: i32,
    limit: i32,
) -> Result<Vec<ItemInfo>, DbError> {
    let sql = r#"
        SELECT
            i.itemID as item_id,
            i.title as title,
            i.date as year,
            (
                SELECT GROUP_CONCAT(
                    COALESCE(c.lastName, '') || COALESCE(c.firstName, ''),
                    '; '
                )
                FROM itemAuthors ia
                JOIN creators c ON ia.authorID = c.creatorID
                WHERE ia.itemID = i.itemID
                ORDER BY ia.order
            ) as authors
        FROM items i
        WHERE i.itemID IS NOT NULL
        ORDER BY i.date DESC, i.title ASC
        LIMIT ? OFFSET ?
    "#;

    query_with_mapper(sql, params![limit, offset], |row| {
        Ok(ItemInfo {
            item_id: row.get(0)?,
            title: row.get::<_, String>(1).unwrap_or_default(),
            year: row.get::<_, String>(2).unwrap_or_default(),
            authors: row.get::<_, String>(3).unwrap_or_default(),
        })
    })
}