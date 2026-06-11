//! 本地文件导入模块
//!
//! 支持本地 PDF 文件导入到 Zotero 数据库，完全兼容 Zotero 存储结构。
//!
//! #核心功能
//! - 支持本地 PDF 文件导入
//! - 自动生成 Zotero 标准的 8 字符随机子文件夹
//! - 自动复制文件到 Zotero 的 storage 目录
//! - 自动生成元数据（标题、itemID、时间等）
//! - 数据库操作使用事务
//!
//! # Zotero 存储结构
//! `storage/{随机8字符}/{itemKey}/{filename}.pdf`

use rand::Rng;
use rusqlite::{params, Connection, Transaction};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tokio::time::{timeout, Duration};

use crate::db::path::get_zotero_database_path;

/// 导入结果结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    /// 新建的文献ID
    pub item_id: i32,
    /// 文献标题
    pub title: String,
    /// 文件路径
    pub file_path: String,
    /// 导入状态消息
    pub message: String,
}

/// 导入错误类型
#[derive(Debug)]
pub enum ImportError {
    /// 文件不存在
    FileNotFound(String),
    /// 文件不是 PDF 格式
    NotPdf(String),
    /// 文件大小超限
    FileTooLarge(String),
    /// 数据库连接失败
    DbConnectionFailed(String),
    /// 数据库操作失败
    DbOperationFailed(String),
    /// 存储目录创建失败
    StorageCreationFailed(String),
    /// 文件复制失败
    FileCopyFailed(String),
    /// 事务失败
    TransactionFailed(String),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::FileNotFound(path) => write!(f, "文件不存在: {}", path),
            ImportError::NotPdf(path) => write!(f, "文件不是 PDF 格式: {}", path),
            ImportError::FileTooLarge(size) => write!(f, "文件大小超限: {}", size),
            ImportError::DbConnectionFailed(msg) => write!(f, "数据库连接失败: {}", msg),
            ImportError::DbOperationFailed(msg) => write!(f, "数据库操作失败: {}", msg),
            ImportError::StorageCreationFailed(msg) => {
                write!(f, "存储目录创建失败: {}", msg)
            }
            ImportError::FileCopyFailed(msg) => write!(f, "文件复制失败: {}", msg),
            ImportError::TransactionFailed(msg) => write!(f, "事务失败: {}", msg),
        }
    }
}

impl std::error::Error for ImportError {}

impl From<ImportError> for String {
    fn from(err: ImportError) -> Self {
        err.to_string()
    }
}

/// 默认最大文件大小（100MB）
const DEFAULT_MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// 查询超时时间（秒）
const QUERY_TIMEOUT_SECS: u64 = 30;

/// 生成 Zotero 标准的 8 字符随机子文件夹名
fn generate_random_subfolder() -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + idx - 10) as char
            }
        })
        .collect();
    chars.iter().collect()
}

/// 生成 Zotero 标准的 itemKey（8字符）
fn generate_item_key() -> String {
    generate_random_subfolder()
}

/// 从文件路径提取标题
fn extract_title_from_filename(file_path: &str) -> String {
    let path = PathBuf::from(file_path);
    let filename = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("未知标题");
    // 移除常见后缀如 _final, v1, v2 等
    let title = filename
        .replace(|c: char| c == '_' || c == '-', " ")
        .trim()
        .to_string();
    if title.is_empty() {
        "未知标题".to_string()
    } else {
        title
    }
}

/// 获取 Zotero 存储目录路径
fn get_zotero_storage_path() -> Result<PathBuf, ImportError> {
    let db_path = get_zotero_database_path().ok_or_else(|| {
        ImportError::DbConnectionFailed("无法检测到 Zotero 数据库路径".to_string())
    })?;

    // 存储目录与数据库在同一目录
    let storage_path = db_path
        .parent()
        .ok_or_else(|| {
            ImportError::DbConnectionFailed("无法获取数据库父目录".to_string())
        })?
        .join("storage");

    Ok(storage_path)
}

/// 检测字段ID（动态查询，避免硬编码）
fn get_field_id(conn: &Connection, field_name: &str) -> Result<i32, ImportError> {
    let field_id: i32 = conn
        .query_row(
            "SELECT fieldID FROM fields WHERE fieldName = ?",
            [field_name],
            |row| row.get(0),
        )
        .map_err(|e| {
            ImportError::DbOperationFailed(format!("查询字段 '{}' 失败: {}", field_name, e))
        })?;
    Ok(field_id)
}

/// 在事务中插入 itemDataValue 并返回 valueID
///
/// 先查询是否已存在相同的值，如果存在则返回已存在的 valueID
/// 如果不存在则插入新值并返回新生成的 valueID
fn insert_item_data_value(
    tx: &Transaction,
    value: &str,
) -> Result<i32, ImportError> {
    // 先查询是否已存在相同的值（通过事务查询）
    let existing_id: Option<i32> = tx
        .query_row(
            "SELECT valueID FROM itemDataValues WHERE value = ?",
            [value],
            |row| row.get(0),
        )
        .ok();

    if let Some(value_id) = existing_id {
        return Ok(value_id);
    }

    // 插入新值
    tx.execute("INSERT INTO itemDataValues (value) VALUES (?)", [value])
        .map_err(|e| {
            ImportError::DbOperationFailed(format!("插入 itemDataValues 失败: {}", e))
        })?;

    // 获取刚插入的 valueID（通过事务查询）
    let value_id = tx
        .query_row("SELECT last_insert_rowid()", [], |row| row.get(0))
        .map_err(|e| {
            ImportError::DbOperationFailed(format!("获取 last_insert_rowid失败: {}", e))
        })?;

    Ok(value_id)
}

/// 在事务中执行导入操作
fn import_file_internal(
    file_path: &str,
    max_file_size: u64,
) -> Result<ImportResult, ImportError> {
    let path = PathBuf::from(file_path);

    // 验证文件是否存在
    if !path.exists() {
        return Err(ImportError::FileNotFound(file_path.to_string()));
    }

    // 验证文件扩展名
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    if extension != "pdf" {
        return Err(ImportError::NotPdf(file_path.to_string()));
    }

    //验证文件大小
    let metadata = fs::metadata(&path).map_err(|e| {
        ImportError::FileNotFound(format!("无法读取文件元数据: {}", e))
    })?;
    if metadata.len() > max_file_size {
        return Err(ImportError::FileTooLarge(format!(
            "文件大小 {}超过限制 {}",
            metadata.len(),
            max_file_size
        )));
    }

    // 获取存储目录
    let storage_path = get_zotero_storage_path()?;

    // 创建存储目录
    if !storage_path.exists() {
        fs::create_dir_all(&storage_path).map_err(|e| {
            ImportError::StorageCreationFailed(format!("创建存储目录失败: {}", e))
        })?;
    }

    // 生成随机子文件夹和 itemKey
    let subfolder = generate_random_subfolder();
    let item_key = generate_item_key();

    // 创建目标目录
    let target_dir = storage_path.join(&subfolder).join(&item_key);
    fs::create_dir_all(&target_dir).map_err(|e| {
        ImportError::StorageCreationFailed(format!("创建目标目录失败: {}: {}", target_dir.display(), e))
    })?;

    // 复制文件
    let filename = path.file_name().unwrap_or_default();
    let target_path = target_dir.join(filename);
    fs::copy(&path, &target_path).map_err(|e| {
        ImportError::FileCopyFailed(format!("复制文件失败: {}", e))
    })?;

    eprintln!(
        "[导入] 文件已复制到: {}",
        target_path.to_string_lossy()
    );

    // 连接数据库
    let db_path = get_zotero_database_path().ok_or_else(|| {
        ImportError::DbConnectionFailed("无法检测到 Zotero 数据库路径".to_string())
    })?;

    let mut conn = Connection::open(&db_path).map_err(|e| {
        ImportError::DbConnectionFailed(format!("打开数据库失败: {}", e))
    })?;

    // 在事务开始前获取所有需要的 field ID
    let title_field_id = get_field_id(&conn, "title")?;
    let date_field_id = get_field_id(&conn, "dateAdded").ok();

    eprintln!("[导入] 字段 ID 获取完成: title_field_id={}, date_field_id={:?}", title_field_id, date_field_id);

    // 开始事务
    let tx = conn.transaction().map_err(|e| {
        ImportError::TransactionFailed(format!("开始事务失败: {}", e))
    })?;

    // 获取当前最大 itemID
    let max_item_id: i32 = tx
        .query_row("SELECT COALESCE(MAX(itemID), 0) FROM items", [], |row| {
            row.get(0)
        })
        .map_err(|e| {
            ImportError::DbOperationFailed(format!("获取最大 itemID 失败: {}", e))
        })?;
    let new_item_id = max_item_id + 1;

    eprintln!("[导入] 新 itemID: {}", new_item_id);

    // 从文件名提取标题
    let title = extract_title_from_filename(file_path);

    // 插入 items 表
    tx.execute(
        "INSERT INTO items (itemID, itemKey, libraryID, keyByUser, itemTypeID, note, sig, clientDate, serverDate, synced, changed)
         VALUES (?, ?, 0, 0, 1, '', '', datetime('now'), datetime('now'), 0, datetime('now'))",
        params![new_item_id, item_key],
    )
    .map_err(|e| {
        ImportError::DbOperationFailed(format!("插入 items 表失败: {}", e))
    })?;

    eprintln!("[导入] 已插入 items 表: itemID={}, itemKey={}", new_item_id, item_key);

    // 插入标题到 itemDataValues（通过事务）
    let title_value_id = insert_item_data_value(&tx, &title)?;

    // 插入 itemData 记录（标题）
    tx.execute(
        "INSERT INTO itemData (itemID, fieldID, valueID) VALUES (?, ?, ?)",
        params![new_item_id, title_field_id, title_value_id],
    )
    .map_err(|e| {
        ImportError::DbOperationFailed(format!("插入 itemData（标题）失败: {}", e))
    })?;

    eprintln!(
        "[导入] 已插入 itemData（标题）: fieldID={}, valueID={}",
        title_field_id, title_value_id
    );

    // 如果 dateAdded 字段存在，插入日期数据
    if let Some(date_fid) = date_field_id {
        let date_value_id = insert_item_data_value(&tx, "")?;
        tx.execute(
            "INSERT INTO itemData (itemID, fieldID, valueID) VALUES (?, ?, ?)",
            params![new_item_id, date_fid, date_value_id],
        )
        .map_err(|e| {
            ImportError::DbOperationFailed(format!("插入 itemData（日期）失败: {}", e))
        })?;
        eprintln!(
            "[导入] 已插入 itemData（日期）: fieldID={}, valueID={}",
            date_fid, date_value_id
        );
    }

    // 提交事务
    tx.commit().map_err(|e| {
        ImportError::TransactionFailed(format!("提交事务失败: {}", e))
    })?;

    eprintln!("[导入] 事务已提交: itemID={}", new_item_id);

    Ok(ImportResult {
        item_id: new_item_id,
        title: title.clone(),
        file_path: target_path.to_string_lossy().to_string(),
        message: format!("成功导入文献: {}", title),
    })
}

/// 导入本地 PDF 文件（异步）
///
/// # 参数
/// * `file_path` - PDF 文件的完整路径
/// * `max_file_size` - 最大文件大小（字节），默认 100MB
///
/// # 返回值
/// * `Result<ImportResult, String>` - 导入结果或错误信息
pub async fn import_file_async(
    file_path: String,
    max_file_size: Option<u64>,
) -> Result<ImportResult, String> {
    let max_size = max_file_size.unwrap_or(DEFAULT_MAX_FILE_SIZE);
    eprintln!("[导入] 开始导入文件: {}", file_path);
    eprintln!("[导入] 最大文件大小限制: {} bytes", max_size);

    let timeout_duration = Duration::from_secs(QUERY_TIMEOUT_SECS);

    let result = timeout(
        timeout_duration,
        tokio::task::spawn_blocking(move || {
            import_file_internal(&file_path, max_size).map_err(|e| e.to_string())
        }),
    )
    .await;

    match result {
        Ok(Ok(inner_result)) => inner_result,
        Ok(Err(e)) => {
            eprintln!("[导入]导入失败: {}", e);
            Err(e.to_string())
        }
        Err(_) => {
            eprintln!("[导入] 导入超时（{}秒）", QUERY_TIMEOUT_SECS);
            Err(format!("导入超时（{}秒）", QUERY_TIMEOUT_SECS))
        }
    }
}

/// 同步版本的导入函数（用于非异步上下文）
pub fn import_file_sync(file_path: &str, max_file_size: Option<u64>) -> Result<ImportResult, String> {
    let max_size = max_file_size.unwrap_or(DEFAULT_MAX_FILE_SIZE);
    import_file_internal(file_path, max_size).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_subfolder() {
        let subfolder = generate_random_subfolder();
        assert_eq!(subfolder.len(), 8);
        assert!(subfolder.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_item_key() {
        let item_key = generate_item_key();
        assert_eq!(item_key.len(), 8);
        assert!(item_key.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_extract_title_from_filename() {
        assert_eq!(
            extract_title_from_filename("/path/to/my_document.pdf"),
            "my document"
        );
        assert_eq!(
            extract_title_from_filename("/path/to/Paper_Title_v1.pdf"),
            "Paper Title v1"
        );
        assert_eq!(
            extract_title_from_filename("/path/to/12345.pdf"),
            "12345"
        );
    }
}