# ZoPlus v0.1.0 发布说明

## 新功能
- 文献管理（导入、排序、删除）
- PDF 预览与标注（高亮、矩形、椭圆、箭头、自由绘制）
- 全文搜索（Tantivy）
- AI 功能（摘要、笔记、格式化、问答）
- 云同步（阿里云）
- 暗色/亮色主题

## 技术栈
- 前端：React 18 + TypeScript + Ant Design
- 后端：Rust + Tauri v2
- 数据库：SQLite (Zotero 原生)
- 搜索引擎：Tantivy

## 系统要求
- Windows 10/11, macOS 12+, Linux
- Zotero 6.0+
- 内存 4GB+

## 构建产物
- `src-tauri/target/release/zoplus.exe` - 可执行文件 (22.4 MB)
- `src-tauri/target/release/bundle/msi/ZoPlus_0.1.0_x64_en-US.msi` - Windows 安装包 (8.5 MB)

## 版本历史
### v0.1.0 (2026-06-11)
- 初始版本发布
- 实现文献管理基本功能
- 实现 PDF 阅读与标注功能
- 实现 Tantivy 全文搜索
- 实现 AI 摘要、笔记、格式化、问答功能
- 实现阿里云云同步功能
- 支持暗色/亮色主题切换