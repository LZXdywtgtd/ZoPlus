//! Tauri 命令调用封装模块
//!
//! 本模块封装所有与 Rust 后端的 IPC 通信，确保前端不直接访问数据库

import { invoke } from '@tauri-apps/api/core';
import type { ItemInfo } from '../store/appStore';

// ============== Zotero 数据库相关 ==============

/// 调用 Rust 后端获取所有文献列表（带超时控制）
///
/// @returns Promise<ItemInfo[]> 文献列表
/// @throws 数据库连接失败、查询错误或超时时抛出异常
export async function getItems(): Promise<ItemInfo[]> {
  //10秒超时控制
  const timeoutPromise = new Promise<never>((_, reject) => {
    setTimeout(() => reject(new Error('文献列表加载超时（10秒）')), 10000);
  });

  const fetchPromise = invoke<ItemInfo[]>('get_items');

  return Promise.race([fetchPromise, timeoutPromise]);
}

/// 分页获取文献列表
///
/// @param offset - 跳过记录数
/// @param limit - 返回记录数上限
/// @returns Promise<ItemInfo[]> 文献列表
/// @throws 数据库连接失败、查询错误或超时时抛出异常
export async function getItemsPaginated(offset: number, limit: number): Promise<ItemInfo[]> {
  // 10秒超时控制
  const timeoutPromise = new Promise<never>((_, reject) => {
    setTimeout(() => reject(new Error('文献列表加载超时（10秒）')), 10000);
  });

  const fetchPromise = invoke<ItemInfo[]>('get_items_paginated', { offset, limit });

  return Promise.race([fetchPromise, timeoutPromise]);
}

/// 根据 ID 获取单条文献信息
///
/// @param itemId - 文献 ID
/// @returns Promise<ItemInfo | null> 文献信息，不存在时返回 null
/// @throws 数据库连接失败或查询错误时抛出异常
export async function getItem(itemId: number): Promise<ItemInfo | null> {
  return await invoke<ItemInfo | null>('get_item', { item_id: itemId });
}

/// 检查 Zotero 数据库状态
///
/// @returns Promise<boolean> 数据库是否存在
export async function checkDbStatus(): Promise<boolean> {
  return await invoke<boolean>('check_db_status');
}

/// 数据库诊断信息结构
export interface DatabaseDiagnosis {
  total_tables: number;
  required_present: string[];
  required_missing: string[];
  optional_present: string[];
  optional_missing: string[];
  all_tables: string[];
}

/// 获取数据库诊断信息
///
/// @returns Promise<DatabaseDiagnosis> 诊断信息
export async function getDbDiagnosis(): Promise<DatabaseDiagnosis> {
  return await invoke<DatabaseDiagnosis>('get_db_diagnosis');
}

/// 手动选择数据库路径
///
/// @param path - 用户选择的数据库文件路径
/// @returns Promise<boolean> 验证成功返回 true
export async function selectDatabasePath(path: string): Promise<boolean> {
  return await invoke<boolean>('select_database_path', { path });
}

// ============== PDF 标注相关 ==============

/// 标注类型
export type AnnotationType =
  | 'highlight'
  | 'rectangle'
  | 'ellipse'
  | 'arrow'
  | 'free_draw'
  | 'text_note';

/// 标注颜色
export interface AnnotationColor {
  r: number;
  g: number;
  b: number;
  a: number;
}

/// PDF 坐标点
export interface PdfPoint {
  x: number;
  y: number;
}

/// PDF 矩形区域
export interface PdfRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/// 高亮标注数据
export interface HighlightData {
  rect: PdfRect;
  text: string;
}

/// 矩形标注数据
export interface RectangleData {
  rect: PdfRect;
  stroke_width: number;
}

/// 椭圆标注数据
export interface EllipseData {
  rect: PdfRect;
  stroke_width: number;
}

/// 箭头标注数据
export interface ArrowData {
  start: PdfPoint;
  end: PdfPoint;
  stroke_width: number;
}

/// 自由绘制标注数据
export interface FreeDrawData {
  points: PdfPoint[];
  stroke_width: number;
}

/// 文本笔记标注数据
export interface TextNoteData {
  position: PdfPoint;
  content: string;
  icon_size: number;
}

/// 标注数据联合类型
export type AnnotationData =
  | HighlightData
  | RectangleData
  | EllipseData
  | ArrowData
  | FreeDrawData
  | TextNoteData;

/// 单个标注结构
export interface Annotation {
  id: string;
  annotation_type: AnnotationType;
  color: AnnotationColor;
  page: number;
  data: AnnotationData;
  created_at: number;
  updated_at: number;
}

/// 标注统计信息
export interface AnnotationStats {
  total_count: number;
  highlight_count: number;
  rectangle_count: number;
  ellipse_count: number;
  arrow_count: number;
  free_draw_count: number;
  text_note_count: number;
  page_count: number;
}

/// 保存单条标注
///
/// @param pdfPath - PDF 文件路径
/// @param fileName - PDF 文件名
/// @param annotation - 标注数据
/// @throws 保存失败时抛出异常
export async function saveAnnotation(
  pdfPath: string,
  fileName: string,
  annotation: Annotation
): Promise<void> {
  return await invoke('save_annotation', {
    pdfPath,
    fileName,
    annotation,
  });
}

/// 批量保存标注
///
/// @param pdfPath - PDF 文件路径
/// @param fileName - PDF 文件名
/// @param annotations - 标注列表
/// @throws 保存失败时抛出异常
export async function saveAnnotations(
  pdfPath: string,
  fileName: string,
  annotations: Annotation[]
): Promise<void> {
  return await invoke('save_annotations', {
    pdfPath,
    fileName,
    annotations,
  });
}

/// 加载指定 PDF 的所有标注
///
/// @param pdfPath - PDF 文件路径
/// @returns Promise<Annotation[]> 标注列表
/// @throws 加载失败时抛出异常
export async function loadAnnotations(pdfPath: string): Promise<Annotation[]> {
  return await invoke<Annotation[]>('load_annotations', { pdfPath });
}

/// 加载指定 PDF 指定页面的标注
///
/// @param pdfPath - PDF 文件路径
/// @param page - 页码（从 1 开始）
/// @returns Promise<Annotation[]> 标注列表
/// @throws 加载失败时抛出异常
export async function loadAnnotationsByPage(
  pdfPath: string,
  page: number
): Promise<Annotation[]> {
  return await invoke<Annotation[]>('load_annotations_by_page', {
    pdfPath,
    page,
  });
}

/// 更新单条标注
///
/// @param pdfPath - PDF 文件路径
/// @param annotation - 更新后的标注数据
/// @returns Promise<boolean> 是否更新成功
/// @throws 更新失败时抛出异常
export async function updateAnnotation(
  pdfPath: string,
  annotation: Annotation
): Promise<boolean> {
  return await invoke<boolean>('update_annotation', {
    pdfPath,
    annotation,
  });
}

/// 删除单条标注
///
/// @param pdfPath - PDF 文件路径
/// @param annotationId - 标注 ID
/// @returns Promise<boolean> 是否删除成功
/// @throws 删除失败时抛出异常
export async function deleteAnnotation(
  pdfPath: string,
  annotationId: string
): Promise<boolean> {
  return await invoke<boolean>('delete_annotation', {
    pdfPath,
    annotationId,
  });
}

/// 删除指定 PDF 的所有标注
///
/// @param pdfPath - PDF 文件路径
/// @returns Promise<boolean> 是否删除成功
/// @throws 删除失败时抛出异常
export async function deleteAllAnnotations(pdfPath: string): Promise<boolean> {
  return await invoke<boolean>('delete_all_annotations', { pdfPath });
}

/// 检查指定 PDF 是否有标注
///
/// @param pdfPath - PDF 文件路径
/// @returns Promise<boolean> 是否有标注
export async function hasAnnotations(pdfPath: string): Promise<boolean> {
  return await invoke<boolean>('has_annotations', { pdfPath });
}

/// 获取标注文件路径
///
/// @param pdfPath - PDF 文件路径
/// @returns Promise<string | null> 标注文件路径，不存在时返回 null
export async function getAnnotationFilePath(
  pdfPath: string
): Promise<string | null> {
  return await invoke<string | null>('get_annotation_file_path', { pdfPath });
}

/// 获取标注统计信息
///
/// @param pdfPath - PDF 文件路径
/// @returns Promise<AnnotationStats> 标注统计信息
/// @throws 获取失败时抛出异常
export async function getAnnotationStats(
  pdfPath: string
): Promise<AnnotationStats> {
  return await invoke<AnnotationStats>('get_annotation_stats', { pdfPath });
}
