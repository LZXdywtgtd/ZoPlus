//! ZoPlus 论文管理软件 - Rust 后端入口
//!
//! 基于 Tauri + Rust 重构 Zotero 前端，集成 MiniMax AI 与阿里云云同步。

use db::{
    delete_item_async, delete_items_async, get_all_items_async, get_current_db_path, get_database_diagnosis,
    get_item_by_id_async, get_items_paginated_async as fetch_items_paginated_async, reset_connection,
    validate_sqlite_file, zotero_db_exists, DatabaseDiagnosis, DatabaseStructure, DbError as DbErr,
    DeleteResult, ItemInfo,
};
use db::path::get_zotero_database_path;
use error::{get_user_message, AppError};
use pdf::commands::{
    delete_all_annotations, delete_annotation, extract_pdf_text, extract_pdf_text_range,
    get_annotation_file_path, get_annotation_stats,
    has_annotations, load_annotations, load_annotations_by_page, save_annotation, save_annotations,
    update_annotation,
};
use search::commands::{
    build_search_index, clear_search_index, delete_from_index, get_index_status, init_search_index,
    search_papers, update_paper_index, SearchState,
};
use ai::commands::{
    get_ai_config, update_ai_config, update_ai_api_key, update_ai_model, update_ai_provider,
    set_ai_enabled, is_ai_configured, chat_completion, test_ai_connection,
    get_all_ai_models, get_ai_models_by_provider, get_model_price, AIState,
    get_article_summary, has_cached_summary, get_cached_summary, export_summary_as_markdown,
    generate_note, generate_notes_batch, save_note_to_item, get_notes_for_item,
    delete_note, update_note, export_note_as_markdown, export_all_notes_as_markdown,
};
use ai::citation_commands::{
    parse_citation_text, format_citation, format_citations_batch,
    enrich_citation_metadata, get_citation_formats, create_formatter_with_config,
};
use ai::rag_commands::{
    ai_chat, ai_chat_stream, get_chat_history, clear_chat_history,
    get_chat_context, update_rag_config, get_rag_config, RagState,
};
use ai::comparison_commands::{
    compare_articles, get_comparison_result, has_comparison_result,
    export_comparison, get_comparison_as_markdown, get_comparison_as_csv,
};
use ai::citation_graph_commands::{
    get_citation_graph, get_key_papers, get_paper_citations,
};
use ai::commands::answer_paper_question;
use import::{import_file_async, ImportResult};
use sync::commands::{
    configure_sync, get_sync_config, get_sync_status, start_background_sync, stop_background_sync,
    sync_now,
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
// 文件导入模块
pub mod import;
// 统一错误处理模块
pub mod error;
// 日志系统模块
pub mod logger;

/// Tauri 命令：获取所有文献列表（异步）
///
/// # 返回值
/// * `Result<Vec<ItemInfo>, String>` - 文献列表或错误信息
#[tauri::command]
async fn get_items() -> Result<Vec<ItemInfo>, String> {
    eprintln!("[命令] get_items 被调用");
    get_all_items_async().await.map_err(|e| {
        eprintln!("[数据库] 获取文献列表失败: {:?}", e);
        get_user_message(&AppError::QueryFailed).to_string()
    })
}

/// Tauri 命令：分页获取文献列表（异步）
///
/// # 参数
/// * `offset` - 跳过记录数
/// * `limit` - 返回记录数上限
///
/// # 返回值
/// * `Result<Vec<ItemInfo>, String>` - 文献列表或错误信息
#[tauri::command]
async fn get_items_paginated(offset: i32, limit: i32) -> Result<Vec<ItemInfo>, String> {
    eprintln!(
        "[命令] get_items_paginated 被调用: offset={}, limit={}",
        offset, limit
    );
    fetch_items_paginated_async(offset, limit).await.map_err(|e| {
        eprintln!("[数据库] 分页获取文献列表失败: {:?}", e);
        get_user_message(&AppError::QueryFailed).to_string()
    })
}

/// Tauri 命令：根据ID获取单条文献（异步）
///
/// # 参数
/// * `item_id` - 文献ID
///
/// # 返回值
/// * `Result<Option<ItemInfo>, String>` - 文献信息或错误信息
#[tauri::command]
async fn get_item(item_id: i32) -> Result<Option<ItemInfo>, String> {
    get_item_by_id_async(item_id).await.map_err(|e| {
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

/// Tauri 命令：探索数据库完整结构（用于生成文档）
///
/// # 返回值
/// * `Result<DatabaseStructure, String>` - 完整的数据库结构信息
#[tauri::command]
fn explore_database_structure() -> Result<DatabaseStructure, String> {
    eprintln!("[命令] explore_database_structure 被调用");
    let db_path = get_zotero_database_path()
        .ok_or_else(|| "无法检测到 Zotero 数据库路径".to_string())?;

    let guard = db::connection::get_connection()
        .map_err(|e| format!("获取数据库连接失败: {:?}", e))?;
    let conn = guard.as_ref().ok_or_else(|| "数据库连接未初始化".to_string())?;

    db::explore_database_structure(conn, &db_path).map_err(|e| {
        eprintln!("[数据库] 探索数据库结构失败: {:?}", e);
        format!("探索数据库结构失败: {}", e)
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

/// Tauri 命令：导入本地 PDF 文件
///
/// # 参数
/// * `file_path` - PDF 文件的完整路径
/// * `max_file_size` - 最大文件大小（字节），可选，默认 100MB
///
/// # 返回值
/// * `Result<ImportResult, String>` - 导入结果或错误信息
#[tauri::command]
async fn import_file(
    file_path: String,
    max_file_size: Option<u64>,
) -> Result<ImportResult, String> {
    eprintln!("[命令] import_file 被调用: file_path={}", file_path);
    import_file_async(file_path, max_file_size).await
}

/// Tauri 命令：删除单条文献
///
/// # 参数
/// * `item_id` - 要删除的文献ID
///
/// # 返回值
/// * `Result<DeleteResult, String>` - 删除结果或错误信息
#[tauri::command]
async fn delete_item(item_id: i32) -> Result<DeleteResult, String> {
    eprintln!("[命令] delete_item 被调用: item_id={}", item_id);
    delete_item_async(item_id).await
}

/// Tauri 命令：批量删除文献
///
/// # 参数
/// * `item_ids` - 要删除的文献ID列表
///
/// # 返回值
/// * `Result<DeleteResult, String>` - 删除结果或错误信息
#[tauri::command]
async fn delete_items(item_ids: Vec<i32>) -> Result<DeleteResult, String> {
    eprintln!("[命令] delete_items 被调用: count={}", item_ids.len());
    delete_items_async(item_ids).await
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
    // 初始化日志系统（必须最先）
    logger::init_logger("ZoPlus");

    //初始化搜索状态
    let search_index_path = get_search_index_path();
    let search_state = SearchState::new(search_index_path);

    //初始化 AI 状态
    let ai_state = AIState::new();

    // 初始化 RAG 状态
    let rag_state = RagState::new();

    log_info!("ZoPlus 应用启动");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(search_state)
        .manage(ai_state)
        .manage(rag_state)
        .invoke_handler(tauri::generate_handler![
            get_items,
            get_items_paginated,
            get_item,
            check_db_status,
            get_db_diagnosis,
            explore_database_structure,
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
            extract_pdf_text,
            extract_pdf_text_range,
            // 文件导入命令
            import_file,
            // 文献删除命令
            delete_item,
            delete_items,
            // 全文搜索相关命令
            init_search_index,
            build_search_index,
            search_papers,
            clear_search_index,
            get_index_status,
            update_paper_index,
            delete_from_index,
            // AI 相关命令
            get_ai_config,
            update_ai_config,
            update_ai_api_key,
            update_ai_model,
            update_ai_provider,
            set_ai_enabled,
            is_ai_configured,
            chat_completion,
            test_ai_connection,
            get_all_ai_models,
            get_ai_models_by_provider,
            get_model_price,
            // 文献摘要命令
            get_article_summary,
            has_cached_summary,
            get_cached_summary,
            export_summary_as_markdown,
            // 智能笔记命令
            generate_note,
            generate_notes_batch,
            save_note_to_item,
            get_notes_for_item,
            delete_note,
            update_note,
            export_note_as_markdown,
            export_all_notes_as_markdown,
            // 参考文献格式化命令
            parse_citation_text,
            format_citation,
            format_citations_batch,
            enrich_citation_metadata,
            get_citation_formats,
            create_formatter_with_config,
            // RAG 跨文献问答命令
            ai_chat,
            ai_chat_stream,
            get_chat_history,
            clear_chat_history,
            get_chat_context,
            update_rag_config,
            get_rag_config,
            // 文献对比命令
            compare_articles,
            get_comparison_result,
            has_comparison_result,
            export_comparison,
            get_comparison_as_markdown,
            get_comparison_as_csv,
            // 引用图谱命令
            get_citation_graph,
            get_key_papers,
            get_paper_citations,
            // 单篇文献问答命令
            answer_paper_question,
            // 云同步命令
            sync_now,
            get_sync_status,
            configure_sync,
            get_sync_config,
            start_background_sync,
            stop_background_sync,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
