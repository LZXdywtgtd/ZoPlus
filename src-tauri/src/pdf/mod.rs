//! PDF 解析与标注模块
//!
//! 本模块负责 PDF 文件的解析和标注功能。
//!
//! # 支持的标注类型
//! - [x] 高亮 (Highlight)
//! - [x] 矩形 (Rectangle)
//! - [x] 椭圆 (Ellipse)
//! - [x] 箭头 (Arrow)
//! - [x] 自由绘制 (FreeDraw)
//! - [x] 文本笔记 (TextNote)
//!
//! # 功能说明
//! - [x] PDF.js 预览集成（前端实现）
//! - [x] Canvas 标注渲染（前端实现）
//! - [x] 标注数据序列化（JSON）
//! - [x] Rust 后端存储模块

pub mod annotations;
pub mod storage;
pub mod commands;

// 重新导出常用的类型和函数
pub use annotations::*;
pub use storage::AnnotationStorage;
pub use commands::*;
