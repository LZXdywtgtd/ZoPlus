//! 文献删除模块
//!
//! 提供文献删除功能，支持单条和批量删除。
//!
//! # 核心功能
//! - 删除单条文献
//! - 批量删除文献
//! - 删除关联的附件文件（storage 目录）
//! - 使用事务确保原子性
//!
//! # 删除逻辑
//! 1. 在事务中删除 itemData、itemCreators、itemTags 等关联数据
//! 2. 从 items 表删除记录（或标记到 deletedItems）
//! 3. 删除 storage 中的附件文件
//!
//! # 安全规则
//! - 删除操作需要写权限，与只读查询连接分离
//! - 使用事务确保数据一致性
//! - 删除前验证 item 是否存在

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tokio::time::{timeout, Duration};

use super::path::get_zotero_database_path;

/// 删除结果结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteResult {
    /// 删除成功的文献ID列表
    pub deleted_ids: Vec<i32>,
    /// 删除失败的文献ID及错误信息
    pub failed_ids: Vec<DeleteFailure>,
    /// 删除的文件数量
    pub files_deleted: i32,
    /// 操作状态消息
    pub message: String,
}

/// 单条删除失败信息
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteFailure {
    /// 文献ID
    pub item_id: i32,
    /// 错误信息
    pub error: String,
}

/// 删除错误类型
#[derive(Debug)]
pub enum DeleteError {
    /// 数据库连接失败
    DbConnectionFailed(String),
    /// 数据库操作失败
    DbOperationFailed(String),
    /// 文献不存在
    ItemNotFound(i32),
    /// 事务失败
    TransactionFailed(String),
    /// 文件删除失败
    FileDeleteFailed(String),
}

impl std::fmt::Display for DeleteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteError::DbConnectionFailed(msg) => write!(f, "数据库连接失败: {}", msg),
            DeleteError::DbOperationFailed(msg) => write!(f, "数据库操作失败: {}", msg),
            DeleteError::ItemNotFound(id) => write!(f, "文献不存在: item_id={}", id),
            DeleteError::TransactionFailed(msg) => write!(f, "事务失败: {}", msg),
            DeleteError::FileDeleteFailed(msg) => write!(f, "文件删除失败: {}", msg),
        }
    }
}

impl std::error::Error for DeleteError {}

/// 操作超时时间（秒）
const DELETE_TIMEOUT_SECS: u64 = 60;

/// 获取 Zotero storage 目录路径
fn get_zotero_storage_path() -> Result<PathBuf, DeleteError> {
    let db_path = get_zotero_database_path().ok_or_else(|| {
        DeleteError::DbConnectionFailed("无法检测到 Zotero 数据库路径".to_string())
    })?;

    let storage_path = db_path
        .parent()
        .ok_or_else(|| {
            DeleteError::DbConnectionFailed("无法获取数据库父目录".to_string())
        })?
        .join("storage");

    Ok(storage_path)
}

/// 根据 itemID 和 itemKey 查找并删除 storage 目录中的附件
///
/// Zotero storage 结构: storage/{子文件夹}/{itemKey}/{filename}
fn delete_storage_for_item(storage_path: &PathBuf, item_key: &str) -> Result<bool, DeleteError> {
    if !storage_path.exists() {
        return Ok(false);
    }

    // 遍历 storage 目录下的所有子文件夹，查找匹配 itemKey 的目录
    let entries = fs::read_dir(storage_path).map_err(|e| {
        DeleteError::FileDeleteFailed(format!("读取 storage 目录失败: {}", e))
    })?;

    let mut deleted = false;
    for entry in entries.flatten() {
        let subfolder = entry.path();
        if subfolder.is_dir() {
            let item_dir = subfolder.join(item_key);
            if item_dir.exists() {
                // 删除整个 itemKey 目录
                fs::remove_dir_all(&item_dir).map_err(|e| {
                    DeleteError::FileDeleteFailed(format!("删除附件目录失败: {}", e))
                })?;
                eprintln!("[删除] 已删除附件目录: {}", item_dir.display());
                deleted = true;
            }
        }
    }

    Ok(deleted)
}

/// 验证 item 是否存在
fn verify_item_exists(conn: &Connection, item_id: i32) -> Result<String, DeleteError> {
    // 获取 itemKey 用于后续删除 storage
    let item_key: Option<String> = conn
        .query_row(
            "SELECT itemKey FROM items WHERE itemID = ?",
            [item_id],
            |row| row.get(0),
        )
        .ok();

    match item_key {
        Some(key) => Ok(key),
        None => Err(DeleteError::ItemNotFound(item_id)),
    }
}

/// 在事务中执行单条文献删除
fn delete_item_internal(item_id: i32) -> Result<bool, DeleteError> {
    // 获取数据库路径
    let db_path = get_zotero_database_path().ok_or_else(|| {
        DeleteError::DbConnectionFailed("无法检测到 Zotero 数据库路径".to_string())
    })?;

    // 打开写权限的数据库连接（不使用 query_only）
    let mut conn = Connection::open(&db_path).map_err(|e| {
        DeleteError::DbConnectionFailed(format!("打开数据库失败: {}", e))
    })?;

    // 验证 item 是否存在，获取 itemKey
    let item_key = verify_item_exists(&conn, item_id)?;

    // 开始事务
    let tx = conn.transaction().map_err(|e| {
        DeleteError::TransactionFailed(format!("开始事务失败: {}", e))
    })?;

    // 1. 删除 itemData 记录
    tx.execute("DELETE FROM itemData WHERE itemID = ?", [item_id])
        .map_err(|e| {
            DeleteError::DbOperationFailed(format!("删除 itemData 失败: {}", e))
        })?;
    eprintln!("[删除] 已删除 itemData: item_id={}", item_id);

    // 2. 删除 itemCreators 记录
    tx.execute("DELETE FROM itemCreators WHERE itemID = ?", [item_id])
        .map_err(|e| {
            DeleteError::DbOperationFailed(format!("删除 itemCreators 失败: {}", e))
        })?;
    eprintln!("[删除] 已删除 itemCreators: item_id={}", item_id);

    // 3. 删除 itemTags 记录
    tx.execute("DELETE FROM itemTags WHERE itemID = ?", [item_id])
        .map_err(|e| {
            DeleteError::DbOperationFailed(format!("删除 itemTags 失败: {}", e))
        })?;
    eprintln!("[删除] 已删除 itemTags: item_id={}", item_id);

    // 4. 删除 itemAttachments 记录（如果有）
    tx.execute("DELETE FROM itemAttachments WHERE itemID = ?", [item_id])
        .ok(); // 忽略错误，可能表不存在
    eprintln!("[删除] 已删除 itemAttachments: item_id={}", item_id);

    // 5. 删除 items 表记录
    let rows_affected = tx
        .execute("DELETE FROM items WHERE itemID = ?", [item_id])
        .map_err(|e| {
            DeleteError::DbOperationFailed(format!("删除 items 记录失败: {}", e))
        })?;

    if rows_affected == 0 {
        return Err(DeleteError::ItemNotFound(item_id));
    }
    eprintln!("[删除] 已删除 items: item_id={}", item_id);

    // 6. 提交事务
    tx.commit().map_err(|e| {
        DeleteError::TransactionFailed(format!("提交事务失败: {}", e))
    })?;

    // 7. 删除 storage 中的附件文件
    let storage_path = get_zotero_storage_path()?;
    if storage_path.exists() {
        match delete_storage_for_item(&storage_path, &item_key) {
            Ok(true) => eprintln!("[删除] 已删除附件: item_id={}, item_key={}", item_id, item_key),
            Ok(false) => eprintln!("[删除] 未找到附件: item_id={}", item_id),
            Err(e) => eprintln!("[删除] 删除附件失败（继续）: item_id={}, error={}", item_id, e),
        }
    }

    Ok(true)
}

/// 删除单条文献（异步）
///
/// # 参数
/// * `item_id` - 要删除的文献ID
///
/// # 返回值
/// * `Result<DeleteResult, String>` - 删除结果或错误信息
pub async fn delete_item_async(item_id: i32) -> Result<DeleteResult, String> {
    eprintln!("[删除] 开始删除文献: item_id={}", item_id);

    let timeout_duration = Duration::from_secs(DELETE_TIMEOUT_SECS);

    let result = timeout(
        timeout_duration,
        tokio::task::spawn_blocking(move || {
            match delete_item_internal(item_id) {
                Ok(true) => Ok(DeleteResult {
                    deleted_ids: vec![item_id],
                    failed_ids: vec![],
                    files_deleted: 1,
                    message: format!("成功删除文献: item_id={}", item_id),
                }),
                Ok(false) => Ok(DeleteResult {
                    deleted_ids: vec![],
                    failed_ids: vec![DeleteFailure {
                        item_id,
                        error: "删除未完成".to_string(),
                    }],
                    files_deleted: 0,
                    message: format!("删除未完成: item_id={}", item_id),
                }),
                Err(e) => Ok(DeleteResult {
                    deleted_ids: vec![],
                    failed_ids: vec![DeleteFailure {
                        item_id,
                        error: e.to_string(),
                    }],
                    files_deleted: 0,
                    message: format!("删除失败: item_id={}, error={}", item_id, e),
                }),
            }
        }),
    )
    .await;

    match result {
        Ok(Ok(delete_result)) => delete_result,
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err(format!("删除超时（{}秒）", DELETE_TIMEOUT_SECS)),
    }
}

/// 批量删除文献（异步）
///
/// # 参数
/// * `item_ids` - 要删除的文献ID列表
///
/// # 返回值
/// * `Result<DeleteResult, String>` - 删除结果或错误信息
pub async fn delete_items_async(item_ids: Vec<i32>) -> Result<DeleteResult, String> {
    eprintln!("[删除] 开始批量删除文献: count={}", item_ids.len());

    let timeout_duration = Duration::from_secs(DELETE_TIMEOUT_SECS);

    let result = timeout(
        timeout_duration,
        tokio::task::spawn_blocking(move || {
            let mut deleted_ids = Vec::new();
            let mut failed_ids = Vec::new();
            let mut files_deleted = 0;

            for item_id in &item_ids {
                match delete_item_internal(*item_id) {
                    Ok(true) => {
                        deleted_ids.push(*item_id);
                        files_deleted += 1;
                    }
                    Ok(false) => {
                        eprintln!("[删除] 删除未完成: item_id={}", item_id);
                        failed_ids.push(DeleteFailure {
                            item_id: *item_id,
                            error: "删除未完成".to_string(),
                        });
                    }
                    Err(e) => {
                        eprintln!("[删除] 删除失败: item_id={}, error={}", item_id, e);
                        failed_ids.push(DeleteFailure {
                            item_id: *item_id,
                            error: e.to_string(),
                        });
                    }
                }
            }

            let message = if failed_ids.is_empty() {
                format!("成功删除 {} 篇文献", deleted_ids.len())
            } else {
                format!(
                    "删除完成: 成功 {} 篇, 失败 {} 篇",
                    deleted_ids.len(),
                    failed_ids.len()
                )
            };

            Ok(DeleteResult {
                deleted_ids,
                failed_ids,
                files_deleted,
                message,
            })
        }),
    )
    .await;

    match result {
        Ok(Ok(delete_result)) => delete_result,
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err(format!("批量删除超时（{}秒）", DELETE_TIMEOUT_SECS)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_result_struct() {
        let result = DeleteResult {
            deleted_ids: vec![1, 2, 3],
            failed_ids: vec![],
            files_deleted: 2,
            message: "测试消息".to_string(),
        };
        assert_eq!(result.deleted_ids.len(), 3);
        assert_eq!(result.files_deleted, 2);
    }

    #[test]
    fn test_delete_failure_struct() {
        let failure = DeleteFailure {
            item_id: 123,
            error: "测试错误".to_string(),
        };
        assert_eq!(failure.item_id, 123);
        assert_eq!(failure.error, "测试错误");
    }
}