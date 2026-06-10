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

// ============== 文献摘要相关 ==============

/// 文献摘要结构
export interface ArticleSummary {
  item_id: number;
  title: string;
  authors: string;
  year: string;
  core_problem: string;
  research_methods: string;
  key_conclusions: string;
  innovation: string;
  limitations: string;
  keywords: string[];
  generated_at: number;
  citation: string;
  highlighted_content: string[];
  version: number;
}

/// 生成文献摘要
///
/// @param itemId - 文献 ID
/// @param pdfKey - PDF 密钥（可选，用于提取用户标注）
/// @returns Promise<ArticleSummary> 摘要信息
/// @throws AI 未配置或生成失败时抛出异常
export async function getArticleSummary(
  itemId: number,
  pdfKey?: string
): Promise<ArticleSummary> {
  return await invoke<ArticleSummary>('get_article_summary', {
    item_id: itemId,
    pdf_key: pdfKey,
  });
}

/// 检查是否有缓存的摘要
///
/// @param itemId - 文献 ID
/// @returns Promise<boolean> 是否有缓存
export async function hasCachedSummary(itemId: number): Promise<boolean> {
  return await invoke<boolean>('has_cached_summary', { item_id: itemId });
}

/// 获取缓存的摘要
///
/// @param itemId - 文献 ID
/// @returns Promise<ArticleSummary | null> 缓存的摘要，不存在时返回 null
export async function getCachedSummary(itemId: number): Promise<ArticleSummary | null> {
  return await invoke<ArticleSummary | null>('get_cached_summary', { item_id: itemId });
}

/// 导出摘要为 Markdown 格式
///
/// @param itemId - 文献 ID
/// @returns Promise<string> Markdown 格式的摘要
/// @throws 没有缓存摘要时抛出异常
export async function exportSummaryAsMarkdown(itemId: number): Promise<string> {
  return await invoke<string>('export_summary_as_markdown', { item_id: itemId });
}

// ============== 智能笔记相关 ==============

/// 笔记模板类型
export type NoteTemplateType =
  | 'key_points'
  | 'methods'
  | 'conclusions'
  | 'critical'
  | 'general';

/// 笔记结构
export interface Note {
  note_id: string;
  item_id: number;
  item_title: string;
  title: string;
  content: string;
  template: NoteTemplateType;
  source_text: string | null;
  page: number | null;
  tags: string[];
  created_at: number;
  updated_at: number;
  version: number;
}

/// 生成单条笔记
///
/// @param itemId - 文献 ID
/// @param sourceText - 原文内容（选自高亮）
/// @param page - 页码
/// @param template - 笔记模板类型
/// @returns Promise<Note> 生成的笔记
/// @throws AI 未配置或生成失败时抛出异常
export async function generateNote(
  itemId: number,
  sourceText: string | null,
  page: number | null,
  template: NoteTemplateType
): Promise<Note> {
  return await invoke<Note>('generate_note', {
    item_id: itemId,
    source_text: sourceText,
    page: page,
    template: template,
  });
}

/// 批量生成笔记（基于多个高亮）
///
/// @param itemId - 文献 ID
/// @param pdfKey - PDF 密钥
/// @param template - 笔记模板类型
/// @returns Promise<Note[]> 生成的笔记列表
/// @throws AI 未配置或生成失败时抛出异常
export async function generateNotesBatch(
  itemId: number,
  pdfKey: string,
  template: NoteTemplateType
): Promise<Note[]> {
  return await invoke<Note[]>('generate_notes_batch', {
    item_id: itemId,
    pdf_key: pdfKey,
    template: template,
  });
}

/// 保存笔记到 Zotero itemNotes 表
///
/// @param itemId - 文献 ID
/// @param note - 笔记数据
/// @returns Promise<boolean> 保存成功返回 true
/// @throws 保存失败时抛出异常
export async function saveNoteToItem(itemId: number, note: Note): Promise<boolean> {
  return await invoke<boolean>('save_note_to_item', {
    item_id: itemId,
    note: note,
  });
}

/// 获取指定文献的所有笔记
///
/// @param itemId - 文献 ID
/// @returns Promise<Note[]> 笔记列表
/// @throws 获取失败时抛出异常
export async function getNotesForItem(itemId: number): Promise<Note[]> {
  return await invoke<Note[]>('get_notes_for_item', { item_id: itemId });
}

/// 删除笔记
///
/// @param noteId - 笔记 ID
/// @returns Promise<boolean> 删除成功返回 true
/// @throws 删除失败时抛出异常
export async function deleteNote(noteId: string): Promise<boolean> {
  return await invoke<boolean>('delete_note', { note_id: noteId });
}

/// 更新笔记
///
/// @param note - 更新后的笔记数据
/// @returns Promise<boolean> 更新成功返回 true
/// @throws 更新失败时抛出异常
export async function updateNote(note: Note): Promise<boolean> {
  return await invoke<boolean>('update_note', { note: note });
}

/// 导出单条笔记为 Markdown
///
/// @param note - 笔记数据
/// @returns string Markdown 格式
export async function exportNoteAsMarkdown(note: Note): Promise<string> {
  return await invoke<string>('export_note_as_markdown', { note: note });
}

/// 批量导出笔记为 Markdown
///
/// @param notes - 笔记列表
/// @param itemTitle - 文献标题
/// @returns string Markdown 格式
export async function exportAllNotesAsMarkdown(notes: Note[], itemTitle: string): Promise<string> {
  return await invoke<string>('export_all_notes_as_markdown', {
    notes: notes,
    item_title: itemTitle,
  });
}

// ============== RAG 跨文献问答相关 ==============

/// 文献上下文结构
export interface DocumentContext {
  item_id: number;
  title: string;
  authors: string;
  year: string;
  abstract_text: string;
  keywords: string;
  score: number;
  citation_key: string;
}

/// RAG 聊天消息结构
export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  citations: DocumentContext[];
  timestamp: number;
}

/// RAG 配置结构
export interface RagConfig {
  top_k: number;
  streaming: boolean;
  min_score: number;
}

/// 发送聊天消息（非流式）
///
/// @param message - 用户消息
/// @returns Promise<ChatMessage> 助手回复
/// @throws AI 未配置或生成失败时抛出异常
export async function aiChat(message: string): Promise<ChatMessage> {
  return await invoke<ChatMessage>('ai_chat', { message });
}

///发送聊天消息（流式）
///
/// @param message - 用户消息
/// @returns Promise<string[]> 流式输出的文本片段
/// @throws AI 未配置或生成失败时抛出异常
export async function aiChatStream(message: string): Promise<string[]> {
  return await invoke<string[]>('ai_chat_stream', { message });
}

/// 获取聊天历史
///
/// @returns Promise<ChatMessage[]> 聊天消息列表
export async function getChatHistory(): Promise<ChatMessage[]> {
  return await invoke<ChatMessage[]>('get_chat_history');
}

/// 清除聊天历史
///
/// @returns Promise<boolean> 清除成功返回 true
export async function clearChatHistory(): Promise<boolean> {
  return await invoke<boolean>('clear_chat_history');
}

/// 获取当前引用的文献上下文
///
/// @returns Promise<DocumentContext[]> 文献上下文列表
export async function getChatContext(): Promise<DocumentContext[]> {
  return await invoke<DocumentContext[]>('get_chat_context');
}

/// 更新 RAG 配置
///
/// @param config - 配置对象（可选字段）
/// @returns Promise<RagConfig> 更新后的完整配置
export async function updateRagConfig(config: Partial<RagConfig>): Promise<RagConfig> {
  return await invoke<RagConfig>('update_rag_config', {
    top_k: config.top_k,
    streaming: config.streaming,
    min_score: config.min_score,
  });
}

/// 获取 RAG 配置
///
/// @returns Promise<RagConfig> 当前配置
export async function getRagConfig(): Promise<RagConfig> {
  return await invoke<RagConfig>('get_rag_config');
}

// ============== 文献对比相关 ==============

/// 对比维度结构
export interface ComparisonDimensions {
  research_questions: string[];
  research_methods: string[];
  key_conclusions: string[];
  innovations: string[];
  limitations: string[];
  citations: string[];
}

/// 矛盾点分析
export interface Contradiction {
  description: string;
  involved_indices: number[];
  contradiction_type: string;
}

/// 共识点分析
export interface Consensus {
  description: string;
  involved_indices: number[];
}

/// 引用关系
export interface CitationRelation {
  from_index: number;
  to_index: number;
  description: string;
}

/// 文献对比结构
export interface ArticleComparison {
  comparison_id: string;
  item_ids: number[];
  titles: string[];
  authors: string[];
  years: string[];
  dimensions: ComparisonDimensions;
  contradictions: Contradiction[];
  consensus: Consensus[];
  citation_relations: CitationRelation[];
  generated_at: number;
  version: number;
}

/// 对比多篇文献
///
/// @param itemIds - 文献 ID 列表（2-5篇）
/// @returns Promise<ArticleComparison> 对比结果
/// @throws AI 未配置或对比失败时抛出异常
export async function compareArticles(itemIds: number[]): Promise<ArticleComparison> {
  return await invoke<ArticleComparison>('compare_articles', {
    item_ids: itemIds,
  });
}

/// 获取缓存的对比结果
///
/// @param itemIds - 文献 ID 列表
/// @returns Promise<ArticleComparison | null> 对比结果，不存在时返回 null
export async function getComparisonResult(itemIds: number[]): Promise<ArticleComparison | null> {
  return await invoke<ArticleComparison | null>('get_comparison_result', {
    item_ids: itemIds,
  });
}

/// 检查是否有缓存的对比结果
///
/// @param itemIds - 文献 ID 列表
/// @returns Promise<boolean> 是否有缓存
export async function hasComparisonResult(itemIds: number[]): Promise<boolean> {
  return await invoke<boolean>('has_comparison_result', {
    item_ids: itemIds,
  });
}

/// 导出对比结果
///
/// @param comparison - 对比结果
/// @param format - 导出格式（markdown 或 csv）
/// @returns Promise<string> 导出的内容
/// @throws 不支持的格式时抛出异常
export async function exportComparison(
  comparison: ArticleComparison,
  format: 'markdown' | 'csv'
): Promise<string> {
  return await invoke<string>('export_comparison', {
    comparison: comparison,
    format: format,
  });
}

/// 获取对比结果的 Markdown 格式
///
/// @param itemIds - 文献 ID 列表
/// @returns Promise<string> Markdown 格式的对比结果
/// @throws 没有缓存时抛出异常
export async function getComparisonAsMarkdown(itemIds: number[]): Promise<string> {
  return await invoke<string>('get_comparison_as_markdown', {
    item_ids: itemIds,
  });
}

/// 获取对比结果的 CSV 格式
///
/// @param itemIds - 文献 ID 列表
/// @returns Promise<string> CSV 格式的对比结果
/// @throws 没有缓存时抛出异常
export async function getComparisonAsCsv(itemIds: number[]): Promise<string> {
  return await invoke<string>('get_comparison_as_csv', {
    item_ids: itemIds,
  });
}

// ============== 引用图谱相关 ==============

/// 引用关系图谱节点
export interface CitationNode {
  /// 文献ID
  item_id: number;
  /// 文献标题
  title: string;
  /// 作者信息
  authors: string;
  /// 发表年份
  year: string;
  /// 被引次数
  citation_count: number;
  /// PageRank 值
  pagerank: number;
  /// 节点大小
  node_size: number;
}

/// 引用关系图谱边
export interface CitationEdge {
  /// 源节点ID
  source: number;
  /// 目标节点ID
  target: number;
  /// 权重
  weight: number;
}

/// 引用图谱数据
export interface CitationGraph {
  /// 所有节点
  nodes: CitationNode[];
  /// 所有边
  edges: CitationEdge[];
  /// 总节点数
  total_nodes: number;
  /// 总边数
  total_edges: number;
  /// 计算耗时（毫秒）
  compute_time_ms: number;
}

/// 关键文献
export interface KeyPaper {
  /// 文献ID
  item_id: number;
  /// 文献标题
  title: string;
  /// 作者信息
  authors: string;
  /// 发表年份
  year: string;
  /// PageRank 值
  pagerank: number;
  /// 被引次数
  citation_count: number;
  /// 推荐理由
  reason: string;
}

/// 文献引用关系详情
export interface PaperCitations {
  /// 文献ID
  item_id: number;
  /// 文献标题
  title: string;
  /// 作者信息
  authors: string;
  /// 施引文献
  cited_by: CitationNode[];
  /// 被引文献
  references: CitationNode[];
  /// 总被引次数
  total_cited_by: number;
  /// 总参考文献数
  total_references: number;
}

/// 获取引用图谱数据
///
/// @param minCitations - 最小被引次数（过滤条件）
/// @returns Promise<CitationGraph> 图谱数据
/// @throws 数据库访问失败时抛出异常
export async function getCitationGraph(minCitations: number = 0): Promise<CitationGraph> {
  return await invoke<CitationGraph>('get_citation_graph', {
    min_citations: minCitations,
  });
}

/// 获取关键文献推荐列表
///
/// @param limit - 返回数量限制
/// @returns Promise<KeyPaper[]> 关键文献列表
/// @throws 数据库访问失败时抛出异常
export async function getKeyPapers(limit: number = 20): Promise<KeyPaper[]> {
  return await invoke<KeyPaper[]>('get_key_papers', {
    limit: limit,
  });
}

/// 获取指定文献的引用关系
///
/// @param itemId - 文献ID
/// @returns Promise<PaperCitations> 引用关系详情
/// @throws 数据库访问失败或文献不存在时抛出异常
export async function getPaperCitations(itemId: number): Promise<PaperCitations> {
  return await invoke<PaperCitations>('get_paper_citations', {
    item_id: itemId,
  });
}
