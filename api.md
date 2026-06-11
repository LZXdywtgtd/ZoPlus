# ZoPlus API 文档
**版本**：v2.0
**最后更新**：2026年6月12日

---

## 目录
1. [数据库相关命令](#1-数据库相关命令)
2. [PDF 处理相关命令](#2-pdf-处理相关命令)
3. [全文搜索相关命令](#3-全文搜索相关命令)
4. [AI 相关命令](#4-ai-相关命令)
5. [文件导入相关命令](#5-文件导入相关命令)
6. [云同步相关命令](#6-云同步相关命令)
7. [动态数据库自省系统](#7-动态数据库自省系统)

---

## 1. 数据库相关命令

### 1.1 get_items
获取所有文献列表（异步）

**返回值**：`Result<Vec<ItemInfo>, String>`
```typescript
interface ItemInfo {
  item_id: number;
  title: string;
  authors: string[];
  year: Option<i32>;
  abstract: Option<String>;
  item_type: Option<String>;
  key: String;
}
```

### 1.2 get_items_paginated
分页获取文献列表

**参数**：
- `offset: i32` - 跳过记录数
- `limit: i32` - 返回记录数上限

**返回值**：`Result<Vec<ItemInfo>, String>`

### 1.3 get_item
根据 ID 获取单条文献

**参数**：
- `item_id: i32` - 文献 ID

**返回值**：`Result<Option<ItemInfo>, String>`

### 1.4 check_db_status
检查数据库状态

**返回值**：`bool` - 数据库是否存在

### 1.5 get_db_diagnosis
获取数据库诊断信息

**返回值**：`Result<DatabaseDiagnosis, String>`
```typescript
interface DatabaseDiagnosis {
  total_tables: number;
  required_present: string[];    // 存在的必需表
  required_missing: string[];    // 缺失的必需表
  optional_present: string[];    // 存在的可选表
  optional_missing: string[];    // 缺失的可选表
  all_tables: string[];          // 所有表名
}
```

### 1.6 explore_database_structure
探索数据库完整结构

**返回值**：`Result<DatabaseStructure, String>`
```typescript
interface DatabaseStructure {
  db_path: string;
  file_size: number;
  total_tables: number;
  all_tables: string[];
  table_structures: TableStructure[];
}

interface TableStructure {
  name: string;
  columns: ColumnInfo[];
  row_count: number;
}

interface ColumnInfo {
  cid: number;
  name: string;
  column_type: string;
  notnull: boolean;
  dflt_value: Option<string>;
  pk: boolean;
}
```

### 1.7 select_database_path
手动选择数据库路径

**参数**：
- `path: String` - 用户选择的数据库文件路径

**返回值**：`Result<bool, String>`

### 1.8 get_current_database_path
获取当前数据库路径

**返回值**：`Option<String>`

### 1.9 reset_db_connection
重置数据库连接

**返回值**：`bool`

### 1.10 delete_item
删除单条文献

**参数**：
- `item_id: i32` - 文献 ID

**返回值**：`Result<DeleteResult, String>`

### 1.11 delete_items
批量删除文献

**参数**：
- `item_ids: Vec<i32>` - 文献 ID 列表

**返回值**：`Result<DeleteResult, String>`

---

## 2. PDF 处理相关命令

### 2.1 save_annotation
保存 PDF 标注

**参数**：
- `item_id: i32` - 文献 ID
- `annotation: Annotation` - 标注数据

**返回值**：`Result<(), String>`

### 2.2 save_annotations
批量保存 PDF 标注

**参数**：
- `item_id: i32` - 文献 ID
- `annotations: Vec<Annotation>` - 标注列表

**返回值**：`Result<(), String>`

### 2.3 load_annotations
加载文献的所有标注

**参数**：
- `item_id: i32` - 文献 ID

**返回值**：`Result<Vec<Annotation>, String>`

### 2.4 load_annotations_by_page
按页码加载标注

**参数**：
- `item_id: i32` - 文献 ID
- `page: i32` - 页码（从 1 开始）

**返回值**：`Result<Vec<Annotation>, String>`

### 2.5 update_annotation
更新标注

**参数**：
- `annotation: Annotation` - 标注数据

**返回值**：`Result<(), String>`

### 2.6 delete_annotation
删除标注

**参数**：
- `annotation_id: String` - 标注 ID

**返回值**：`Result<(), String>`

### 2.7 delete_all_annotations
删除文献的所有标注

**参数**：
- `item_id: i32` - 文献 ID

**返回值**：`Result<(), String>`

### 2.8 has_annotations
检查文献是否有标注

**参数**：
- `item_id: i32` - 文献 ID

**返回值**：`Result<bool, String>`

### 2.9 get_annotation_file_path
获取标注文件路径

**参数**：
- `item_id: i32` - 文献 ID

**返回值**：`Result<String, String>`

### 2.10 get_annotation_stats
获取标注统计信息

**返回值**：`Result<AnnotationStats, String>`

### 2.11 extract_pdf_text
提取 PDF 全文文本

**参数**：
- `item_id: i32` - 文献 ID

**返回值**：`Result<String, String>`

### 2.12 extract_pdf_text_range
提取 PDF 指定页面范围的文本

**参数**：
- `item_id: i32` - 文献 ID
- `start_page: i32` - 起始页码
- `end_page: i32` - 结束页码

**返回值**：`Result<String, String>`

---

## 3. 全文搜索相关命令

### 3.1 init_search_index
初始化搜索索引

**返回值**：`Result<SearchIndexStatus, String>`

### 3.2 build_search_index
构建搜索索引

**返回值**：`Result<IndexingResult, String>`

### 3.3 search_papers
搜索文献

**参数**：
- `query: String` - 搜索关键词
- `limit: Option<i32>` - 返回结果数量限制

**返回值**：`Result<Vec<SearchResult>, String>`

### 3.4 clear_search_index
清空搜索索引

**返回值**：`Result<(), String>`

### 3.5 get_index_status
获取索引状态

**返回值**：`Result<SearchIndexStatus, String>`

### 3.6 update_paper_index
更新单篇文献的索引

**参数**：
- `item_id: i32` - 文献 ID

**返回值**：`Result<(), String>`

### 3.7 delete_from_index
从索引中删除文献

**参数**：
- `item_id: i32` - 文献 ID

**返回值**：`Result<(), String>`

---

## 4. AI 相关命令

### 4.1 get_ai_config
获取 AI 配置

**返回值**：`Result<AIConfig, String>`

### 4.2 update_ai_config
更新 AI 配置

**参数**：
- `config: AIConfig` - AI 配置

**返回值**：`Result<(), String>`

### 4.3 update_ai_api_key
更新 API Key

**参数**：
- `api_key: String` - 新的 API Key

**返回值**：`Result<(), String>`

### 4.4 update_ai_model
更新 AI 模型

**参数**：
- `model: String` - 模型名称

**返回值**：`Result<(), String>`

### 4.5 update_ai_provider
更新 AI 服务提供商

**参数**：
- `provider: String` - 提供商名称

**返回值**：`Result<(), String>`

### 4.6 set_ai_enabled
设置 AI 功能启用状态

**参数**：
- `enabled: bool` - 是否启用

**返回值**：`Result<(), String>`

### 4.7 is_ai_configured
检查 AI 是否已配置

**返回值**：`bool`

### 4.8 chat_completion
通用聊天完成

**参数**：
- `messages: Vec<ChatMessage>` - 消息列表
- `model: Option<String>` - 模型名称

**返回值**：`Result<String, String>`

### 4.9 test_ai_connection
测试 AI 连接

**返回值**：`Result<AIConnectionTest, String>`

### 4.10 get_all_ai_models
获取所有可用模型

**返回值**：`Result<Vec<AIProviderModels>, String>`

### 4.11 get_ai_models_by_provider
获取指定提供商的模型列表

**参数**：
- `provider: String` - 提供商名称

**返回值**：`Result<Vec<ModelInfo>, String>`

### 4.12 get_model_price
获取模型价格信息

**参数**：
- `provider: String` - 提供商
- `model: String` - 模型名称

**返回值**：`Result<ModelPrice, String>`

### 4.13 get_article_summary
生成文献摘要

**参数**：
- `item_id: i32` - 文献 ID
- `max_length: Option<i32>` - 最大长度

**返回值**：`Result<String, String>`

### 4.14 has_cached_summary
检查是否有缓存的摘要

**参数**：
- `item_id: i32` - 文献 ID

**返回值**：`bool`

### 4.15 get_cached_summary
获取缓存的摘要

**参数**：
- `item_id: i32` - 文献 ID

**返回值**：`Result<Option<String>, String>`

### 4.16 export_summary_as_markdown
导出摘要为 Markdown 格式

**参数**：
- `item_id: i32` - 文献 ID

**返回值**：`Result<String, String>`

### 4.17 generate_note
生成智能笔记

**参数**：
- `item_id: i32` - 文献 ID
- `note_type: String` - 笔记类型

**返回值**：`Result<String, String>`

### 4.18 generate_notes_batch
批量生成笔记

**参数**：
- `item_ids: Vec<i32>` - 文献 ID 列表
- `note_type: String` - 笔记类型

**返回值**：`Result<Vec<NoteResult>, String>`

### 4.19 save_note_to_item
保存笔记到文献

**参数**：
- `item_id: i32` - 文献 ID
- `note_content: String` - 笔记内容

**返回值**：`Result<i32, String>` - 返回 note ID

### 4.20 get_notes_for_item
获取文献的所有笔记

**参数**：
- `item_id: i32` - 文献 ID

**返回值**：`Result<Vec<Note>, String>`

### 4.21 delete_note
删除笔记

**参数**：
- `note_id: i32` - 笔记 ID

**返回值**：`Result<(), String>`

### 4.22 update_note
更新笔记

**参数**：
- `note_id: i32` - 笔记 ID
- `content: String` - 新内容

**返回值**：`Result<(), String>`

### 4.23 export_note_as_markdown
导出笔记为 Markdown

**参数**：
- `note_id: i32` - 笔记 ID

**返回值**：`Result<String, String>`

### 4.24 export_all_notes_as_markdown
导出所有笔记为 Markdown

**返回值**：`Result<String, String>`

### 4.25 parse_citation_text
解析引用文本

**参数**：
- `text: String` - 引用文本

**返回值**：`Result<ParsedCitation, String>`

### 4.26 format_citation
格式化引用

**参数**：
- `item_id: i32` - 文献 ID
- `style: String` - 引用样式

**返回值**：`Result<String, String>`

### 4.27 format_citations_batch
批量格式化引用

**参数**：
- `item_ids: Vec<i32>` - 文献 ID 列表
- `style: String` - 引用样式

**返回值**：`Result<Vec<String>, String>`

### 4.28 enrich_citation_metadata
丰富引用元数据

**参数**：
- `citation: ParsedCitation` - 解析后的引用

**返回值**：`Result<EnrichedCitation, String>`

### 4.29 get_citation_formats
获取所有可用的引用格式

**返回值**：`Result<Vec<CitationFormat>, String>`

### 4.30 create_formatter_with_config
创建自定义格式化的引用

**参数**：
- `config: FormatterConfig` - 格式化配置

**返回值**：`Result<String, String>`

### 4.31 ai_chat
跨文献智能问答

**参数**：
- `question: String` - 问题
- `item_ids: Option<Vec<i32>>` - 涉及的文献 ID 列表

**返回值**：`Result<ChatResponse, String>`

### 4.32 ai_chat_stream
跨文献智能问答（流式）

**参数**：
- `question: String` - 问题
- `item_ids: Option<Vec<i32>>` - 涉及的文献 ID 列表

**返回值**：流式返回 `String`

### 4.33 get_chat_history
获取聊天历史

**参数**：
- `limit: Option<i32>` - 返回数量限制

**返回值**：`Result<Vec<ChatMessage>, String>`

### 4.34 clear_chat_history
清空聊天历史

**返回值**：`Result<(), String>`

### 4.35 get_chat_context
获取聊天上下文

**返回值**：`Result<ChatContext, String>`

### 4.36 update_rag_config
更新 RAG 配置

**参数**：
- `config: RagConfig` - RAG 配置

**返回值**：`Result<(), String>`

### 4.37 get_rag_config
获取 RAG 配置

**返回值**：`Result<RagConfig, String>`

### 4.38 compare_articles
对比多篇文献

**参数**：
- `item_ids: Vec<i32>` - 文献 ID 列表
- `aspect: Option<String>` - 对比维度

**返回值**：`Result<String, String>` - 对比结果 ID

### 4.39 get_comparison_result
获取对比结果

**参数**：
- `result_id: String` - 结果 ID

**返回值**：`Result<ComparisonResult, String>`

### 4.40 has_comparison_result
检查是否有对比结果

**参数**：
- `result_id: String` - 结果 ID

**返回值**：`bool`

### 4.41 export_comparison
导出对比结果

**参数**：
- `result_id: String` - 结果 ID
- `format: String` - 格式（markdown/csv）

**返回值**：`Result<String, String>`

### 4.42 get_comparison_as_markdown
获取 Markdown 格式的对比结果

**参数**：
- `result_id: String` - 结果 ID

**返回值**：`Result<String, String>`

### 4.43 get_comparison_as_csv
获取 CSV 格式的对比结果

**参数**：
- `result_id: String` - 结果 ID

**返回值**：`Result<String, String>`

### 4.44 get_citation_graph
获取引用图谱

**参数**：
- `item_id: i32` - 文献 ID
- `depth: Option<i32>` - 图谱深度

**返回值**：`Result<CitationGraph, String>`

### 4.45 get_key_papers
获取关键论文

**参数**：
- `item_ids: Vec<i32>` - 文献 ID 列表
- `limit: Option<i32>` - 返回数量

**返回值**：`Result<Vec<KeyPaper>, String>`

### 4.46 get_paper_citations
获取论文的引用关系

**参数**：
- `item_id: i32` - 文献 ID

**返回值**：`Result<PaperCitations, String>`

### 4.47 answer_paper_question
单篇文献智能问答

**参数**：
- `item_id: i32` - 文献 ID
- `question: String` - 问题

**返回值**：`Result<String, String>`

---

## 5. 文件导入相关命令

### 5.1 import_file
导入本地 PDF 文件

**参数**：
- `file_path: String` - PDF 文件完整路径
- `max_file_size: Option<u64>` - 最大文件大小（字节），默认 100MB

**返回值**：`Result<ImportResult, String>`

### 5.2 import_folder
导入文件夹中的所有 PDF 文件

**参数**：
- `folder_path: String` - 文件夹完整路径

**返回值**：`Result<FolderImportResult, String>`

---

## 6. 云同步相关命令

### 6.1 sync_now
立即执行同步

**返回值**：`Result<SyncResult, String>`

### 6.2 get_sync_status
获取同步状态

**返回值**：`Result<SyncStatus, String>`

### 6.3 configure_sync
配置同步服务

**参数**：
- `config: SyncConfig` - 同步配置

**返回值**：`Result<(), String>`

### 6.4 get_sync_config
获取同步配置

**返回值**：`Result<SyncConfig, String>`

### 6.5 start_background_sync
启动后台同步

**返回值**：`Result<(), String>`

### 6.6 stop_background_sync
停止后台同步

**返回值**：`Result<(), String>`

---

## 7. 动态数据库自省系统

### 7.1 系统概述
ZoPlus v2.0 实现了完全动态数据库自省系统，通过运行时扫描 Zotero 数据库的实际表结构，动态构建 SQL 语句，支持不同版本的 Zotero 数据库。

### 7.2 核心数据结构

#### DatabaseMetadata
```rust
pub struct DatabaseMetadata {
    tables: HashMap<String, TableMetadata>,  // 表名(小写) -> 表元数据
}
```

#### TableMetadata
```rust
pub struct TableMetadata {
    pub name: String,                    // 表名
    pub columns: Vec<ColumnMetadata>,    // 字段列表
}
```

#### ColumnMetadata
```rust
pub struct ColumnMetadata {
    pub name: String,        // 字段名
    pub data_type: String,   // 数据类型
    pub is_primary_key: bool,// 是否主键
    pub is_nullable: bool,   // 是否可为空
}
```

### 7.3 核心方法

#### 表查询
- `table_exists(name: &str) -> bool` - 检查表是否存在（大小写不敏感）
- `get_table(name: &str) -> Option<&TableMetadata>` - 获取表元数据
- `find_table(candidates: &[&str]) -> Option<&str>` - 智能查找表（支持多候选表名）

#### 字段查询
- `column_exists(table_name: &str, column_name: &str) -> bool` - 检查字段是否存在
- `get_column(table_name: &str, column_name: &str) -> Option<&ColumnMetadata>` - 获取字段元数据

#### 动态 SQL 构建
- `build_insert(table: &str, data: &HashMap<&str, &str>) -> Option<(String, Vec<&str>)>`
- `build_select(table: &str, columns: &[&str], where_clause: &str) -> Option<String>`
- `build_update(table: &str, data: &HashMap<&str, &str>, where_clause: &str) -> Option<(String, Vec<&str>)>`
- `build_delete(table: &str, where_clause: &str) -> Option<String>`

#### 缓存管理
- `get_cached_metadata(conn: &Connection) -> SqliteResult<DatabaseMetadata>` - 获取缓存的元数据
- `invalidate_metadata_cache()` - 清除元数据缓存

### 7.4 Zotero 表名候选列表

```rust
pub struct ZoteroTableCandidates;

impl ZoteroTableCandidates {
    /// 作者关联表（兼容不同 Zotero 版本）
    pub const CREATORS: &'static [&'static str] = &["itemCreators", "itemAuthors", "itemCreator"];

    /// 作者表
    pub const CREATOR: &'static [&'static str] = &["creators"];

    /// 文献数据表
    pub const ITEM_DATA: &'static [&'static str] = &["itemData"];

    /// 文献数据值表
    pub const ITEM_DATA_VALUES: &'static [&'static str] = &["itemDataValues"];

    /// 文献表
    pub const ITEMS: &'static [&'static str] = &["items"];

    /// 文献标签表
    pub const ITEM_TAGS: &'static [&'static str] = &["itemTags", "tags"];

    /// 集合表
    pub const COLLECTIONS: &'static [&'static str] = &["collections"];

    /// 文献附件表
    pub const ITEM_ATTACHMENTS: &'static [&'static str] = &["itemAttachments", "attachments"];

    /// 字段表
    pub const FIELDS: &'static [&'static str] = &["fields"];

    /// 文献类型表
    pub const ITEM_TYPES: &'static [&'static str] = &["itemTypes"];

    /// 删除文献表
    pub const DELETED_ITEMS: &'static [&'static str] = &["deletedItems"];
}
```

### 7.5 动态列名检测

#### find_items_key_column
动态检测 items 表中的 key 列名。Zotero 可能使用 `key`、`itemKey`、`keyString`、`itemKeyString` 等列名。

```rust
pub fn find_items_key_column(metadata: &DatabaseMetadata) -> Option<&str>
```

### 7.6 使用示例

```typescript
// 前端调用示例
const diagnosis = await invoke<DatabaseDiagnosis>("get_db_diagnosis");
console.log(`检测到 ${diagnosis.total_tables} 个表`);

// 探索数据库结构
const structure = await invoke<DatabaseStructure>("explore_database_structure");
structure.table_structures.forEach(table => {
    console.log(`表: ${table.name}, 行数: ${table.row_count}`);
    table.columns.forEach(col => {
        console.log(`  - ${col.name}: ${col.column_type}`);
    });
});
```

---

## 附录：类型定义

```typescript
// 标注类型
type AnnotationType = "highlight" | "underline" | "strikeout" | "note" | "image" | "freehand";

interface Annotation {
  id: string;
  item_id: number;
  page: number;
  type: AnnotationType;
  content: string;
  rects: Rect[];
  color?: string;
  created_at: string;
  updated_at: string;
}

interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface AnnotationStats {
  total: number;
  by_type: Record<AnnotationType, number>;
  by_page: Record<number, number>;
}

// 搜索相关
interface SearchResult {
  item_id: number;
  title: string;
  snippet: string;
  score: number;
}

interface SearchIndexStatus {
  initialized: boolean;
  document_count: number;
  last_updated: string;
}

// AI 相关
interface AIConfig {
  provider: string;
  model: string;
  api_key: string;
  enabled: boolean;
}

interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
}

interface ChatResponse {
  answer: string;
  sources: number[];
}

// 同步相关
interface SyncConfig {
  server_url: string;
  token: string;
  enabled: boolean;
  auto_sync: boolean;
  sync_interval: number;
}

interface SyncStatus {
  last_sync: string;
  status: "idle" | "syncing" | "error";
  error?: string;
}
```