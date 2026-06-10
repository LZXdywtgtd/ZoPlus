//! Tantivy 搜索模块 Tauri Command 接口
//!
//! 本模块提供与前端交互的 Tauri Command 接口，
//! 包括：构建索引、执行搜索、清除索引等操作。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

use super::indexer::{IndexDocument, IndexerError, SearchIndexer};
use super::query::{SearchEngine, SearchParams, SearchResult};

/// 全局搜索状态
pub struct SearchState {
    /// 索引构建器（使用 Arc 共享）
    pub indexer: Mutex<Option<Arc<SearchIndexer>>>,
    /// 搜索引擎
    pub engine: Mutex<Option<SearchEngine>>,
    /// 索引路径
    pub index_path: PathBuf,
}

impl SearchState {
    /// 创建新的搜索状态
    pub fn new(index_path: PathBuf) -> Self {
        Self {
            indexer: Mutex::new(None),
            engine: Mutex::new(None),
            index_path,
        }
    }

    /// 初始化索引和搜索引擎
    pub fn initialize(&self) -> Result<(), IndexerError> {
        let mut indexer_guard = self.indexer.lock().unwrap();
        let mut engine_guard = self.engine.lock().unwrap();

        // 创建并初始化索引构建器
        let mut indexer = SearchIndexer::new(self.index_path.clone())?;
        indexer.initialize_index()?;

        // 使用 Arc 共享索引构建器
        let indexer_arc = Arc::new(indexer);

        // 创建搜索引擎（克隆 Arc）
        let engine = SearchEngine::new(indexer_arc.clone());

        *indexer_guard = Some(indexer_arc);
        *engine_guard = Some(engine);

        Ok(())
    }

    /// 获取索引构建器
    pub fn get_indexer(
        &self,
    ) -> Result<std::sync::MutexGuard<Option<Arc<SearchIndexer>>>, IndexerError> {
        Ok(self.indexer.lock().unwrap())
    }

    /// 获取搜索引擎
    pub fn get_engine(&self) -> Result<std::sync::MutexGuard<Option<SearchEngine>>, IndexerError> {
        Ok(self.engine.lock().unwrap())
    }
}

/// 搜索请求参数
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    /// 搜索关键词
    pub query: String,
    /// 跳过的记录数
    pub offset: Option<usize>,
    /// 返回的记录数上限
    pub limit: Option<usize>,
    /// 是否启用模糊搜索
    pub fuzzy: Option<bool>,
    /// 模糊搜索的编辑距离
    pub fuzzy_distance: Option<u8>,
}

impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            offset: Some(0),
            limit: Some(20),
            fuzzy: Some(false),
            fuzzy_distance: Some(2),
        }
    }
}

/// 搜索响应结构
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    /// 搜索结果列表
    pub results: Vec<SearchResult>,
    /// 总结果数
    pub total: usize,
    /// 当前页偏移
    pub offset: usize,
    /// 当前页返回数
    pub limit: usize,
}

/// 索引构建进度信息
#[derive(Debug, Serialize)]
pub struct IndexBuildProgress {
    /// 已处理记录数
    pub processed: usize,
    /// 总记录数
    pub total: usize,
    /// 是否完成
    pub completed: bool,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

/// 初始化搜索索引
///
/// # 返回值
/// * `Result<bool, String>` - 初始化是否成功
#[tauri::command]
pub fn init_search_index(state: State<SearchState>) -> Result<bool, String> {
    state.initialize().map_err(|e| format!("{e}"))?;
    Ok(true)
}

/// 构建全文索引
///
/// # 返回值
/// * `Result<IndexBuildProgress, String>` - 构建进度
#[tauri::command]
pub fn build_search_index(state: State<SearchState>) -> Result<IndexBuildProgress, String> {
    let indexer_guard = state
        .get_indexer()
        .map_err(|e: IndexerError| format!("{e}"))?;
    let indexer = indexer_guard.as_ref().ok_or("索引未初始化")?;

    let total = indexer
        .build_index_from_database(|processed, total| {
            println!("索引构建进度: {}/{}", processed, total);
        })
        .map_err(|e: IndexerError| format!("{e}"))?;

    Ok(IndexBuildProgress {
        processed: total,
        total,
        completed: true,
        error: None,
    })
}

/// 执行搜索
///
/// # 参数
/// * `request` - 搜索请求参数
///
/// # 返回值
/// * `Result<SearchResponse, String>` - 搜索响应
#[tauri::command]
pub fn search_papers(
    state: State<SearchState>,
    request: SearchRequest,
) -> Result<SearchResponse, String> {
    let engine_guard = state
        .get_engine()
        .map_err(|e: IndexerError| format!("{e}"))?;
    let engine = engine_guard.as_ref().ok_or("搜索引擎未初始化")?;

    let params = SearchParams {
        query: request.query,
        offset: request.offset.unwrap_or(0),
        limit: request.limit.unwrap_or(20),
        fuzzy: request.fuzzy.unwrap_or(false),
        fuzzy_distance: request.fuzzy_distance.unwrap_or(2),
    };

    let results = engine
        .search(params.clone())
        .map_err(|e: IndexerError| format!("{e}"))?;
    let total = engine
        .get_total_count(&params.query)
        .map_err(|e: IndexerError| format!("{e}"))?;

    Ok(SearchResponse {
        results,
        total,
        offset: params.offset,
        limit: params.limit,
    })
}

/// 清除搜索索引
///
/// # 返回值
/// * `Result<bool, String>` - 清除是否成功
#[tauri::command]
pub fn clear_search_index(state: State<SearchState>) -> Result<bool, String> {
    let indexer_guard = state
        .get_indexer()
        .map_err(|e: IndexerError| format!("{e}"))?;
    let indexer = indexer_guard.as_ref().ok_or("索引未初始化")?;

    indexer
        .clear_index()
        .map_err(|e: IndexerError| format!("{e}"))?;
    Ok(true)
}

/// 获取索引状态
///
/// # 返回值
/// * `Result<IndexStatus, String>` - 索引状态信息
#[derive(Debug, Serialize)]
pub struct IndexStatus {
    /// 索引路径
    pub index_path: String,
    /// 文档总数
    pub document_count: usize,
    ///索引是否存在
    pub exists: bool,
}

#[tauri::command]
pub fn get_index_status(state: State<SearchState>) -> Result<IndexStatus, String> {
    let indexer_guard = state
        .get_indexer()
        .map_err(|e: IndexerError| format!("{e}"))?;
    let indexer = indexer_guard.as_ref();

    let document_count = indexer
        .map(|i| i.get_document_count().unwrap_or(0))
        .unwrap_or(0);
    let exists = indexer.is_some();

    Ok(IndexStatus {
        index_path: state.index_path.to_string_lossy().to_string(),
        document_count,
        exists,
    })
}

/// 更新单篇文献的索引
///
/// # 参数
/// * `item_id` - 文献ID
/// * `title` - 标题
/// * `authors` - 作者
/// * `year` - 年份
/// * `abstract_text` - 摘要
/// * `keywords` - 关键词
/// * `tags` - 标签
///
/// # 返回值
/// * `Result<bool, String>` - 更新是否成功
#[tauri::command]
pub fn update_paper_index(
    state: State<SearchState>,
    item_id: i32,
    title: String,
    authors: String,
    year: String,
    abstract_text: String,
    keywords: String,
    tags: String,
) -> Result<bool, String> {
    let indexer_guard = state
        .get_indexer()
        .map_err(|e: IndexerError| format!("{e}"))?;
    let indexer = indexer_guard.as_ref().ok_or("索引未初始化")?;

    let doc = IndexDocument {
        item_id,
        title,
        authors,
        year,
        abstract_text,
        keywords,
        fulltext_path: String::new(),
        tags,
    };

    indexer
        .update_document(doc)
        .map_err(|e: IndexerError| format!("{e}"))?;
    indexer.commit().map_err(|e: IndexerError| format!("{e}"))?;

    Ok(true)
}

/// 从索引中删除文献
///
/// # 参数
/// * `item_id` - 文献ID
///
/// # 返回值
/// * `Result<bool, String>` - 删除是否成功
#[tauri::command]
pub fn delete_from_index(state: State<SearchState>, item_id: i32) -> Result<bool, String> {
    let indexer_guard = state
        .get_indexer()
        .map_err(|e: IndexerError| format!("{e}"))?;
    let indexer = indexer_guard.as_ref().ok_or("索引未初始化")?;

    indexer
        .delete_document(item_id)
        .map_err(|e: IndexerError| format!("{e}"))?;
    indexer.commit().map_err(|e: IndexerError| format!("{e}"))?;

    Ok(true)
}
