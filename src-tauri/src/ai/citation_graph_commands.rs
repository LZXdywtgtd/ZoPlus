//! 文献引用图谱 Tauri 命令接口
//!
//! 提供引用图谱相关的前端调用接口

use crate::ai::citation_graph::{
    build_citation_graph as build_graph, get_key_papers as get_key_papers_impl,
    get_paper_citations as get_paper_citations_impl, CitationGraph, KeyPaper, PaperCitations,
};

/// Tauri 命令：获取引用图谱数据
///
/// # 参数
/// * `min_citations` - 最小被引次数（过滤条件）
///
/// # 返回值
/// * `Result<CitationGraph, String>` - 图谱数据
#[tauri::command]
pub fn get_citation_graph(min_citations: i32) -> Result<CitationGraph, String> {
    eprintln!(
        "[命令] get_citation_graph 被调用: min_citations={}",
        min_citations
    );

    // 获取 Zotero 数据库路径
    let db_path = crate::db::path::get_zotero_database_path()
        .ok_or_else(|| "无法检测到 Zotero 数据库路径".to_string())?
        .to_string_lossy()
        .to_string();

    eprintln!("[命令] 使用数据库路径: {}", db_path);

    let result = build_graph(&db_path, min_citations);

    eprintln!(
        "[命令] get_citation_graph 完成: nodes={}, edges={}, time={}ms",
        result.as_ref().map(|g| g.total_nodes).unwrap_or(0),
        result.as_ref().map(|g| g.total_edges).unwrap_or(0),
        result.as_ref().map(|g| g.compute_time_ms).unwrap_or(0),
    );

    result
}

/// Tauri 命令：获取关键文献推荐列表
///
/// # 参数
/// * `limit` - 返回数量限制
///
/// # 返回值
/// * `Result<Vec<KeyPaper>, String>` - 关键文献列表
#[tauri::command]
pub fn get_key_papers(limit: i32) -> Result<Vec<KeyPaper>, String> {
    eprintln!("[命令] get_key_papers 被调用: limit={}", limit);

    let db_path = crate::db::path::get_zotero_database_path()
        .ok_or_else(|| "无法检测到 Zotero 数据库路径".to_string())?
        .to_string_lossy()
        .to_string();

    let result = get_key_papers_impl(&db_path, limit);

    eprintln!(
        "[命令] get_key_papers 完成: count={}",
        result.as_ref().map(|p| p.len()).unwrap_or(0)
    );

    result
}

/// Tauri 命令：获取指定文献的引用关系
///
/// # 参数
/// * `item_id` - 文献ID
///
/// # 返回值
/// * `Result<PaperCitations, String>` - 引用关系详情
#[tauri::command]
pub fn get_paper_citations(item_id: i32) -> Result<PaperCitations, String> {
    eprintln!("[命令] get_paper_citations 被调用: item_id={}", item_id);

    let db_path = crate::db::path::get_zotero_database_path()
        .ok_or_else(|| "无法检测到 Zotero 数据库路径".to_string())?
        .to_string_lossy()
        .to_string();

    let result = get_paper_citations_impl(&db_path, item_id);

    eprintln!(
        "[命令] get_paper_citations 完成: cited_by={}, references={}",
        result.as_ref().map(|p| p.total_cited_by).unwrap_or(0),
        result.as_ref().map(|p| p.total_references).unwrap_or(0),
    );

    result
}