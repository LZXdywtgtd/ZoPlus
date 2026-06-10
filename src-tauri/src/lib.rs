//! ZoPlus 论文管理软件 - Rust 后端入口
//!
//! 基于 Tauri + Rust 重构 Zotero 前端，集成 MiniMax AI 与阿里云云同步。

use db::{
    get_all_items, get_current_db_path, get_database_diagnosis, get_item_by_id,
    get_items_paginated as fetch_items_paginated, reset_connection, validate_sqlite_file,
    zotero_db_exists, DatabaseDiagnosis, DbError as DbErr, ItemInfo,
};
use error::{get_user_message, AppError};
use pdf::commands::{
    delete_all_annotations, delete_annotation, get_annotation_file_path, get_annotation_stats,
    has_annotations, load_annotations, load_annotations_by_page, save_annotation, save_annotations,
    update_annotation,
};
use search::commands::{
    build_search_index, clear_search_index, delete_from_index, get_index_status, init_search_index,
    search_papers, update_paper_index, SearchState,
};
use std::path::PathBuf;

// 数据库访问模块
pub mod db;
// 全文搜索模块
pub mod search;
// AI 模块
pub mod ai;
// 云同步模块
pub mod sync;
// PDF 模块
pub mod pdf;
// 统一错误处理模块
pub mod error;

/// Tauri 命令：获取所有文献列表
///
/// # 返回值
/// * `Result<Vec<ItemInfo>, String>` - 文献列表或错误信息
#[tauri::command]
fn get_items() -> Result<Vec<ItemInfo>, String> {
    eprintln!("[命令] get_items 被调用");
    get_all_items().map_err(|e| {
        eprintln!("[数据库] 获取文献列表失败: {:?}", e);
        get_user_message(&AppError::QueryFailed).to_string()
    })
}

/// Tauri 命令：分页获取文献列表
///
/// # 参数
/// * `offset` - 跳过记录数
/// * `limit` - 返回记录数上限
///
/// # 返回值
/// * `Result<Vec<ItemInfo>, String>` - 文献列表或错误信息
#[tauri::command]
fn get_items_paginated(offset: i32, limit: i32) -> Result<Vec<ItemInfo>, String> {
    eprintln!(
        "[命令] get_items_paginated 被调用: offset={}, limit={}",
        offset, limit
    );
    fetch_items_paginated(offset, limit).map_err(|e| {
        eprintln!("[数据库] 分页获取文献列表失败: {:?}", e);
        get_user_message(&AppError::QueryFailed).to_string()
    })
}

/// Tauri 命令：根据ID获取单条文献
///
/// # 参数
/// * `item_id` - 文献ID
///
/// # 返回值
/// * `Result<Option<ItemInfo>, String>` - 文献信息或错误信息
#[tauri::command]
fn get_item(item_id: i32) -> Result<Option<ItemInfo>, String> {
    get_item_by_id(item_id).map_err(|e| {
        eprintln!("[数据库] 获取文献详情失败: item_id={}, error={:?}", item_id, e);
        get_user_message(&AppError::QueryFailed).to_string()
    })
}

/// Tauri 命令：检查数据库状态
///
/// # 返回值
/// * `bool` - 数据库是否存在
#[tauri::command]
fn check_db_status() -> bool {
    zotero_db_exists()
}

/// Tauri 命令：获取数据库诊断信息
///
/// # 返回值
/// * `Result<DatabaseDiagnosis, String>` - 诊断信息或错误
#[tauri::command]
fn get_db_diagnosis() -> Result<DatabaseDiagnosis, String> {
    eprintln!("[命令] get_db_diagnosis 被调用");
    get_database_diagnosis().map_err(|e| {
        eprintln!("[数据库] 获取诊断信息失败: {:?}", e);
        get_user_message(&AppError::QueryFailed).to_string()
    })
}

/// Tauri 命令：手动选择数据库路径
///
/// # 参数
/// * `path` - 用户选择的数据库文件路径
///
/// # 返回值
/// * `Result<bool, String>` - 验证成功返回 true
#[tauri::command]
fn select_database_path(path: String) -> Result<bool, String> {
    eprintln!("[命令] select_database_path 被调用: {}", path);
    let db_path = PathBuf::from(&path);

    // 验证文件是否是有效的 SQLite 数据库
    validate_sqlite_file(&db_path).map_err(|e| {
        eprintln!("[数据库] 数据库文件验证失败: {:?}", e);
        format!("选择的文件不是有效的 SQLite 数据库: {}", e)
    })?;

    // 重置连接并设置新的数据库路径
    reset_connection();

    // 通过环境变量设置自定义路径（会被 path.rs 模块使用）
    std::env::set_var("ZOTERO_DB_PATH", &path);
    eprintln!("[数据库] 已设置自定义数据库路径: {}", path);

    Ok(true)
}

/// Tauri 命令：获取当前数据库路径
///
/// # 返回值
/// * `Option<String>` - 当前数据库路径
#[tauri::command]
fn get_current_database_path() -> Option<String> {
    get_current_db_path().map(|p: PathBuf| p.to_string_lossy().to_string())
}

/// Tauri 命令：重置数据库连接
///
/// 用于在切换数据库路径后重新初始化连接
#[tauri::command]
fn reset_db_connection() -> bool {
    eprintln!("[命令] reset_db_connection 被调用");
    reset_connection();
    true
}

/// 将 DbError 转换为用户友好的错误消息
fn db_error_to_user_message(err: &DbErr) -> String {
    match err {
        DbErr::ValidationFailed(msg) => {
            if msg.contains("MissingTables") {
                format!(
                    "数据库验证失败：缺少必要的表。{}",
                    get_user_message(&AppError::DatabaseNotFound)
                )
            } else {
                format!(
                    "数据库验证失败：{}",
                    get_user_message(&AppError::DatabaseConnectionFailed)
                )
            }
        }
        DbErr::NotFound(_) => get_user_message(&AppError::DatabaseNotFound).to_string(),
        DbErr::ConnectionFailed(_) => {
            get_user_message(&AppError::DatabaseConnectionFailed).to_string()
        }
        DbErr::QueryFailed(msg) => {
            if msg.contains("no such table") {
                format!(
                    "数据库表缺失：{}",
                    get_user_message(&AppError::DatabaseNotFound)
                )
            } else {
                get_user_message(&AppError::QueryFailed).to_string()
            }
        }
        DbErr::LockFailed => get_user_message(&AppError::DatabaseLocked).to_string(),
    }
}

/// 获取应用数据目录下的搜索索引路径
fn get_search_index_path() -> PathBuf {
    // 获取应用数据目录
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ZoPlus")
        .join("search_index");

    app_data_dir
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    //初始化搜索状态
    let search_index_path = get_search_index_path();
    let search_state = SearchState::new(search_index_path);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(search_state)
        .invoke_handler(tauri::generate_handler![
            get_items,
            get_items_paginated,
            get_item,
            check_db_status,
            get_db_diagnosis,
            select_database_path,
            get_current_database_path,
            reset_db_connection,
            // PDF 标注相关命令
            save_annotation,
            save_annotations,
            load_annotations,
            load_annotations_by_page,
            update_annotation,
            delete_annotation,
            delete_all_annotations,
            has_annotations,
            get_annotation_file_path,
            get_annotation_stats,
            // 全文搜索相关命令
            init_search_index,
            build_search_index,
            search_papers,
            clear_search_index,
            get_index_status,
            update_paper_index,
            delete_from_index,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
