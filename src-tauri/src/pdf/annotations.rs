//! PDF 标注数据结构体定义模块
//!
//! 本模块定义了 PDF 标注的各种数据结构，包括高亮、矩形、椭圆等标注类型
//! 标注坐标使用 PDF 原始坐标系统

use serde::{Deserialize, Serialize};

/// 标注类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationType {
    /// 高亮标注
    Highlight,
    /// 矩形标注
    Rectangle,
    /// 椭圆标注
    Ellipse,
    /// 箭头标注
    Arrow,
    /// 自由绘制标注
    FreeDraw,
    /// 文本笔记标注
    TextNote,
}

/// 标注颜色（ARGB 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationColor {
    /// 红色通道 (0-255)
    pub r: u8,
    /// 绿色通道 (0-255)
    pub g: u8,
    /// 蓝色通道 (0-255)
    pub b: u8,
    /// 透明度 (0-255)，0 为完全透明，255 为完全不透明
    pub a: u8,
}

impl AnnotationColor {
    /// 创建默认高亮颜色（黄色）
    pub fn default_highlight() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 0,
            a: 128,
        }
    }

    /// 创建默认矩形颜色（红色）
    pub fn default_rectangle() -> Self {
        Self {
            r: 255,
            g: 0,
            b: 0,
            a: 180,
        }
    }

    /// 创建默认椭圆颜色（蓝色）
    pub fn default_ellipse() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 255,
            a: 180,
        }
    }

    /// 创建默认箭头颜色（绿色）
    pub fn default_arrow() -> Self {
        Self {
            r: 0,
            g: 255,
            b: 0,
            a: 200,
        }
    }

    /// 创建默认自由绘制颜色（黑色）
    pub fn default_free_draw() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    /// 创建默认文本笔记颜色（橙色）
    pub fn default_text_note() -> Self {
        Self {
            r: 255,
            g: 165,
            b: 0,
            a: 200,
        }
    }

    /// 转换为 CSS 颜色字符串
    pub fn to_css_string(&self) -> String {
        format!(
            "rgba({}, {}, {}, {})",
            self.r,
            self.g,
            self.b,
            self.a as f32 / 255.0
        )
    }
}

/// PDF 坐标点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPoint {
    /// X 坐标（PDF 原始坐标系）
    pub x: f64,
    /// Y 坐标（PDF 原始坐标系）
    pub y: f64,
}

/// PDF 矩形区域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRect {
    /// 左上角 X 坐标
    pub x: f64,
    /// 左上角 Y 坐标
    pub y: f64,
    /// 矩形宽度
    pub width: f64,
    /// 矩形高度
    pub height: f64,
}

/// 高亮标注数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightAnnotation {
    /// 高亮区域矩形
    pub rect: PdfRect,
    /// 选择的文本内容
    pub text: String,
}

/// 矩形标注数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RectangleAnnotation {
    /// 矩形区域
    pub rect: PdfRect,
    /// 边框宽度
    pub stroke_width: f64,
}

/// 椭圆标注数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EllipseAnnotation {
    /// 外接矩形区域
    pub rect: PdfRect,
    /// 边框宽度
    pub stroke_width: f64,
}

/// 箭头标注数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrowAnnotation {
    /// 箭头起点
    pub start: PdfPoint,
    /// 箭头终点
    pub end: PdfPoint,
    /// 箭头线宽
    pub stroke_width: f64,
}

/// 自由绘制标注数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeDrawAnnotation {
    /// 绘制路径上的所有点
    pub points: Vec<PdfPoint>,
    /// 绘制线宽
    pub stroke_width: f64,
}

/// 文本笔记标注数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextNoteAnnotation {
    /// 笔记位置（左上角）
    pub position: PdfPoint,
    /// 笔记内容
    pub content: String,
    /// 笔记图标大小
    pub icon_size: f64,
}

/// 单个标注的联合类型数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnnotationData {
    /// 高亮标注
    Highlight(HighlightAnnotation),
    /// 矩形标注
    Rectangle(RectangleAnnotation),
    /// 椭圆标注
    Ellipse(EllipseAnnotation),
    /// 箭头标注
    Arrow(ArrowAnnotation),
    /// 自由绘制
    FreeDraw(FreeDrawAnnotation),
    /// 文本笔记
    TextNote(TextNoteAnnotation),
}

/// 单个标注结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    /// 标注唯一ID
    pub id: String,
    /// 标注类型
    pub annotation_type: AnnotationType,
    /// 标注颜色
    pub color: AnnotationColor,
    /// 所属页面（从 1 开始）
    pub page: u32,
    /// 标注数据
    pub data: AnnotationData,
    /// 创建时间（Unix 时间戳，毫秒）
    pub created_at: i64,
    /// 更新时间（Unix 时间戳，毫秒）
    pub updated_at: i64,
}

impl Annotation {
    /// 创建高亮标注的便捷方法
    pub fn new_highlight(
        rect: PdfRect,
        text: String,
        color: Option<AnnotationColor>,
        page: u32,
    ) -> Self {
        let now = chrono_timestamp();
        Self {
            id: generate_uuid(),
            annotation_type: AnnotationType::Highlight,
            color: color.unwrap_or_else(AnnotationColor::default_highlight),
            page,
            data: AnnotationData::Highlight(HighlightAnnotation { rect, text }),
            created_at: now,
            updated_at: now,
        }
    }

    /// 创建矩形标注的便捷方法
    pub fn new_rectangle(
        rect: PdfRect,
        stroke_width: f64,
        color: Option<AnnotationColor>,
        page: u32,
    ) -> Self {
        let now = chrono_timestamp();
        Self {
            id: generate_uuid(),
            annotation_type: AnnotationType::Rectangle,
            color: color.unwrap_or_else(AnnotationColor::default_rectangle),
            page,
            data: AnnotationData::Rectangle(RectangleAnnotation {
                rect,
                stroke_width,
            }),
            created_at: now,
            updated_at: now,
        }
    }

    /// 创建椭圆标注的便捷方法
    pub fn new_ellipse(
        rect: PdfRect,
        stroke_width: f64,
        color: Option<AnnotationColor>,
        page: u32,
    ) -> Self {
        let now = chrono_timestamp();
        Self {
            id: generate_uuid(),
            annotation_type: AnnotationType::Ellipse,
            color: color.unwrap_or_else(AnnotationColor::default_ellipse),
            page,
            data: AnnotationData::Ellipse(EllipseAnnotation {
                rect,
                stroke_width,
            }),
            created_at: now,
            updated_at: now,
        }
    }

    /// 创建箭头标注的便捷方法
    pub fn new_arrow(
        start: PdfPoint,
        end: PdfPoint,
        stroke_width: f64,
        color: Option<AnnotationColor>,
        page: u32,
    ) -> Self {
        let now = chrono_timestamp();
        Self {
            id: generate_uuid(),
            annotation_type: AnnotationType::Arrow,
            color: color.unwrap_or_else(AnnotationColor::default_arrow),
            page,
            data: AnnotationData::Arrow(ArrowAnnotation {
                start,
                end,
                stroke_width,
            }),
            created_at: now,
            updated_at: now,
        }
    }

    /// 创建自由绘制标注的便捷方法
    pub fn new_free_draw(
        points: Vec<PdfPoint>,
        stroke_width: f64,
        color: Option<AnnotationColor>,
        page: u32,
    ) -> Self {
        let now = chrono_timestamp();
        Self {
            id: generate_uuid(),
            annotation_type: AnnotationType::FreeDraw,
            color: color.unwrap_or_else(AnnotationColor::default_free_draw),
            page,
            data: AnnotationData::FreeDraw(FreeDrawAnnotation {
                points,
                stroke_width,
            }),
            created_at: now,
            updated_at: now,
        }
    }

    /// 创建文本笔记标注的便捷方法
    pub fn new_text_note(
        position: PdfPoint,
        content: String,
        icon_size: f64,
        color: Option<AnnotationColor>,
        page: u32,
    ) -> Self {
        let now = chrono_timestamp();
        Self {
            id: generate_uuid(),
            annotation_type: AnnotationType::TextNote,
            color: color.unwrap_or_else(AnnotationColor::default_text_note),
            page,
            data: AnnotationData::TextNote(TextNoteAnnotation {
                position,
                content,
                icon_size,
            }),
            created_at: now,
            updated_at: now,
        }
    }
}

/// PDF 文件标注集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfAnnotations {
    /// PDF 文件路径（MD5 哈希值，用于标识）
    pub pdf_key: String,
    /// PDF 文件名
    pub file_name: String,
    /// 标注列表
    pub annotations: Vec<Annotation>,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
}

impl PdfAnnotations {
    /// 创建新的标注集合
    pub fn new(pdf_key: String, file_name: String) -> Self {
        let now = chrono_timestamp();
        Self {
            pdf_key,
            file_name,
            annotations: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 添加标注
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.updated_at = chrono_timestamp();
        self.annotations.push(annotation);
    }

    /// 删除标注
    pub fn remove_annotation(&mut self, annotation_id: &str) -> bool {
        let original_len = self.annotations.len();
        self.annotations.retain(|a| a.id != annotation_id);
        if self.annotations.len() != original_len {
            self.updated_at = chrono_timestamp();
            true
        } else {
            false
        }
    }

    /// 更新标注
    pub fn update_annotation(&mut self, annotation: Annotation) -> bool {
        if let Some(idx) = self.annotations.iter().position(|a| a.id == annotation.id) {
            self.annotations[idx] = annotation;
            self.updated_at = chrono_timestamp();
            true
        } else {
            false
        }
    }

    /// 获取指定页面的所有标注
    pub fn get_annotations_by_page(&self, page: u32) -> Vec<&Annotation> {
        self.annotations.iter().filter(|a| a.page == page).collect()
    }
}

// ============== 工具函数 ==============

/// 获取当前时间戳（毫秒）
fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// 生成简单的 UUID（基于时间戳和随机数）
fn generate_uuid() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let random: u32 = rand_simple();
    format!("{:x}-{:x}", now, random)
}

/// 简单的随机数生成器
fn rand_simple() -> u32 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let state = RandomState::new();
    let mut hasher = state.build_hasher();
    hasher.write_u64(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64);
    hasher.finish() as u32
}
