//! PDF 标注 Tauri 命令模块
//!
//! 本模块定义了所有与 PDF 标注相关的 Tauri Command，用于前端与后端的 IPC 通信

use crate::error::{get_user_message, AppError};
use crate::pdf::annotations::{Annotation, AnnotationType, PdfAnnotations};
use crate::pdf::storage::AnnotationStorage;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// 获取应用数据目录下的标注存储目录
fn get_annotation_storage_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("annotations")
}

/// Tauri 命令：保存标注
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
/// * `annotation` - 标注数据（JSON 格式）
///
/// # 返回值
/// * `Result<(), String>` - 成功时返回 ()
#[tauri::command]
pub fn save_annotation(
    app: AppHandle,
    pdf_path: String,
    file_name: String,
    annotation: Annotation,
) -> Result<(), String> {
    let storage_dir = get_annotation_storage_dir(&app);
    let storage = AnnotationStorage::new(storage_dir).map_err(|e| {
        eprintln!("[PDF标注] 存储初始化失败: {:?}", e);
        get_user_message(&AppError::StorageInitFailed).to_string()
    })?;

    let pdf_key = AnnotationStorage::generate_pdf_key(&pdf_path);
    storage
        .save_annotation(&pdf_key, &file_name, annotation)
        .map_err(|e| {
            eprintln!("[PDF标注] 保存标注失败: pdf_key={}, error={:?}", pdf_key, e);
            get_user_message(&AppError::AnnotationSaveFailed).to_string()
        })
}

/// Tauri 命令：批量保存标注
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
/// * `file_name` - PDF 文件名
/// * `annotations` - 标注列表
///
/// # 返回值
/// * `Result<(), String>` - 成功时返回 ()
#[tauri::command]
pub fn save_annotations(
    app: AppHandle,
    pdf_path: String,
    file_name: String,
    annotations: Vec<Annotation>,
) -> Result<(), String> {
    let storage_dir = get_annotation_storage_dir(&app);
    let storage = AnnotationStorage::new(storage_dir).map_err(|e| {
        eprintln!("[PDF标注] 存储初始化失败: {:?}", e);
        get_user_message(&AppError::StorageInitFailed).to_string()
    })?;

    let pdf_key = AnnotationStorage::generate_pdf_key(&pdf_path);
    let mut pdf_annotations = PdfAnnotations::new(pdf_key.clone(), file_name);
    let count = annotations.len();

    for annotation in annotations {
        pdf_annotations.add_annotation(annotation);
    }

    storage.save(&pdf_annotations).map_err(|e| {
        eprintln!("[PDF标注] 批量保存标注失败: pdf_key={}, count={}, error={:?}",
            pdf_key, count, e);
        get_user_message(&AppError::AnnotationSaveFailed).to_string()
    })
}

/// Tauri 命令：加载指定 PDF 的所有标注
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
///
/// # 返回值
/// * `Result<Vec<Annotation>, String>` - 标注列表
#[tauri::command]
pub fn load_annotations(app: AppHandle, pdf_path: String) -> Result<Vec<Annotation>, String> {
    let storage_dir = get_annotation_storage_dir(&app);
    let storage = AnnotationStorage::new(storage_dir).map_err(|e| {
        eprintln!("[PDF标注] 存储初始化失败: {:?}", e);
        get_user_message(&AppError::StorageInitFailed).to_string()
    })?;

    let pdf_key = AnnotationStorage::generate_pdf_key(&pdf_path);
    storage.get_annotations(&pdf_key).map_err(|e| {
        eprintln!("[PDF标注] 加载标注失败: pdf_key={}, error={:?}", pdf_key, e);
        get_user_message(&AppError::AnnotationLoadFailed).to_string()
    })
}

/// Tauri 命令：加载指定 PDF 指定页面的标注
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
/// * `page` - 页码（从 1 开始）
///
/// # 返回值
/// * `Result<Vec<Annotation>, String>` - 标注列表
#[tauri::command]
pub fn load_annotations_by_page(
    app: AppHandle,
    pdf_path: String,
    page: u32,
) -> Result<Vec<Annotation>, String> {
    let storage_dir = get_annotation_storage_dir(&app);
    let storage = AnnotationStorage::new(storage_dir).map_err(|e| {
        eprintln!("[PDF标注] 存储初始化失败: {:?}", e);
        get_user_message(&AppError::StorageInitFailed).to_string()
    })?;

    let pdf_key = AnnotationStorage::generate_pdf_key(&pdf_path);
    storage
        .get_annotations_by_page(&pdf_key, page)
        .map_err(|e| {
            eprintln!("[PDF标注] 按页加载标注失败: pdf_key={}, page={}, error={:?}",
                pdf_key, page, e);
            get_user_message(&AppError::AnnotationLoadFailed).to_string()
        })
}

/// Tauri 命令：更新单条标注
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
/// * `annotation` - 更新后的标注数据
///
/// # 返回值
/// * `Result<bool, String>` - 是否更新成功
#[tauri::command]
pub fn update_annotation(
    app: AppHandle,
    pdf_path: String,
    annotation: Annotation,
) -> Result<bool, String> {
    let storage_dir = get_annotation_storage_dir(&app);
    let storage = AnnotationStorage::new(storage_dir).map_err(|e| {
        eprintln!("[PDF标注] 存储初始化失败: {:?}", e);
        get_user_message(&AppError::StorageInitFailed).to_string()
    })?;

    let pdf_key = AnnotationStorage::generate_pdf_key(&pdf_path);
    storage
        .update_annotation(&pdf_key, annotation)
        .map_err(|e| {
            eprintln!("[PDF标注] 更新标注失败: pdf_key={}, error={:?}", pdf_key, e);
            get_user_message(&AppError::AnnotationSaveFailed).to_string()
        })
}

/// Tauri 命令：删除单条标注
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
/// * `annotation_id` - 标注 ID
///
/// # 返回值
/// * `Result<bool, String>` - 是否删除成功
#[tauri::command]
pub fn delete_annotation(
    app: AppHandle,
    pdf_path: String,
    annotation_id: String,
) -> Result<bool, String> {
    let storage_dir = get_annotation_storage_dir(&app);
    let storage = AnnotationStorage::new(storage_dir).map_err(|e| {
        eprintln!("[PDF标注] 存储初始化失败: {:?}", e);
        get_user_message(&AppError::StorageInitFailed).to_string()
    })?;

    let pdf_key = AnnotationStorage::generate_pdf_key(&pdf_path);
    storage
        .delete_annotation(&pdf_key, &annotation_id)
        .map_err(|e| {
            eprintln!("[PDF标注] 删除标注失败: pdf_key={}, annotation_id={}, error={:?}",
                pdf_key, annotation_id, e);
            get_user_message(&AppError::AnnotationDeleteFailed).to_string()
        })
}

/// Tauri 命令：删除指定 PDF 的所有标注
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
///
/// # 返回值
/// * `Result<bool, String>` - 是否删除成功
#[tauri::command]
pub fn delete_all_annotations(app: AppHandle, pdf_path: String) -> Result<bool, String> {
    let storage_dir = get_annotation_storage_dir(&app);
    let storage = AnnotationStorage::new(storage_dir).map_err(|e| {
        eprintln!("[PDF标注] 存储初始化失败: {:?}", e);
        get_user_message(&AppError::StorageInitFailed).to_string()
    })?;

    let pdf_key = AnnotationStorage::generate_pdf_key(&pdf_path);
    storage.delete(&pdf_key).map_err(|e| {
        eprintln!("[PDF标注] 删除所有标注失败: pdf_key={}, error={:?}", pdf_key, e);
        get_user_message(&AppError::AnnotationDeleteFailed).to_string()
    })
}

/// Tauri 命令：检查指定 PDF 是否有标注
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
///
/// # 返回值
/// * `bool` - 是否有标注
#[tauri::command]
pub fn has_annotations(app: AppHandle, pdf_path: String) -> bool {
    let storage_dir = get_annotation_storage_dir(&app);
    if let Ok(storage) = AnnotationStorage::new(storage_dir) {
        let pdf_key = AnnotationStorage::generate_pdf_key(&pdf_path);
        storage.exists(&pdf_key)
    } else {
        false
    }
}

/// Tauri 命令：获取标注文件路径
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
///
/// # 返回值
/// * `Option<String>` - 标注文件路径，不存在时返回 None
#[tauri::command]
pub fn get_annotation_file_path(app: AppHandle, pdf_path: String) -> Option<String> {
    let storage_dir = get_annotation_storage_dir(&app);
    if let Ok(storage) = AnnotationStorage::new(storage_dir) {
        let pdf_key = AnnotationStorage::generate_pdf_key(&pdf_path);
        if storage.exists(&pdf_key) {
            return Some(
                storage
                    .get_annotation_file_path(&pdf_key)
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }
    None
}

/// Tauri 命令：获取标注统计信息
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
///
/// # 返回值
/// * `AnnotationStats` - 标注统计信息
#[tauri::command]
pub fn get_annotation_stats(app: AppHandle, pdf_path: String) -> Result<AnnotationStats, String> {
    let storage_dir = get_annotation_storage_dir(&app);
    let storage = AnnotationStorage::new(storage_dir).map_err(|e| {
        eprintln!("[PDF标注] 存储初始化失败: {:?}", e);
        get_user_message(&AppError::StorageInitFailed).to_string()
    })?;

    let pdf_key = AnnotationStorage::generate_pdf_key(&pdf_path);
    let annotations = storage
        .get_annotations(&pdf_key)
        .map_err(|e| {
            eprintln!("[PDF标注] 获取标注统计失败: pdf_key={}, error={:?}", pdf_key, e);
            get_user_message(&AppError::AnnotationLoadFailed).to_string()
        })?;

    let mut stats = AnnotationStats::default();

    for annotation in &annotations {
        match annotation.annotation_type {
            AnnotationType::Highlight => stats.highlight_count += 1,
            AnnotationType::Rectangle => stats.rectangle_count += 1,
            AnnotationType::Ellipse => stats.ellipse_count += 1,
            AnnotationType::Arrow => stats.arrow_count += 1,
            AnnotationType::FreeDraw => stats.free_draw_count += 1,
            AnnotationType::TextNote => stats.text_note_count += 1,
        }
    }

    stats.total_count = annotations.len() as u32;
    stats.page_count = annotations
        .iter()
        .map(|a| a.page)
        .collect::<std::collections::HashSet<_>>()
        .len() as u32;

    Ok(stats)
}

/// 标注统计信息结构体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct AnnotationStats {
    /// 总标注数
    pub total_count: u32,
    /// 高亮标注数
    pub highlight_count: u32,
    /// 矩形标注数
    pub rectangle_count: u32,
    /// 椭圆标注数
    pub ellipse_count: u32,
    /// 箭头标注数
    pub arrow_count: u32,
    /// 自由绘制标注数
    pub free_draw_count: u32,
    /// 文本笔记标注数
    pub text_note_count: u32,
    /// 涉及的页数
    pub page_count: u32,
}

// ============== PDF 文本提取命令 ==============

use crate::pdf::text_extract::{extract_text_from_pdf, extract_text_from_pdf_range, PdfTextError};

/// Tauri 命令：提取 PDF 全文文本
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
///
/// # 返回值
/// * `Result<String, String>` - 提取的文本内容
#[tauri::command]
pub fn extract_pdf_text(pdf_path: String) -> Result<String, String> {
    extract_text_from_pdf(&std::path::PathBuf::from(&pdf_path)).map_err(|e| {
        eprintln!("[PDF文本提取] 提取失败: pdf_path={}, error={}", pdf_path, e);
        e.to_string()
    })
}

/// Tauri 命令：提取 PDF 指定页码范围的文本
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
/// * `start_page` - 起始页码（从 1 开始）
/// * `end_page` - 结束页码
///
/// # 返回值
/// * `Result<String, String>` - 提取的文本内容
#[tauri::command]
pub fn extract_pdf_text_range(
    pdf_path: String,
    start_page: u32,
    end_page: u32,
) -> Result<String, String> {
    extract_text_from_pdf_range(
        &std::path::PathBuf::from(&pdf_path),
        start_page,
        end_page,
    )
    .map_err(|e| {
        eprintln!(
            "[PDF文本提取] 范围提取失败: pdf_path={}, pages={}-{}, error={}",
            pdf_path, start_page, end_page, e
        );
        e.to_string()
    })
}
