//! ZoPlus 论文管理软件 - Rust 后端入口
//!
//! 基于 Tauri + Rust 重构 Zotero 前端，集成 MiniMax AI 与阿里云云同步。

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

use db::{get_all_items, get_item_by_id, zotero_db_exists, ItemInfo};

/// Tauri 命令：获取所有文献列表
///
/// # 返回值
/// * `Result<Vec<ItemInfo>, String>` - 文献列表或错误信息
#[tauri::command]
fn get_items() -> Result<Vec<ItemInfo>, String> {
    get_all_items().map_err(|e| e.to_string())
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
    get_item_by_id(item_id).map_err(|e| e.to_string())
}

/// Tauri 命令：检查数据库状态
///
/// # 返回值
/// * `bool` - 数据库是否存在
#[tauri::command]
fn check_db_status() -> bool {
    zotero_db_exists()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_items,
            get_item,
            check_db_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}