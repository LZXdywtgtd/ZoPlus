//! PDF 标注数据持久化模块
//!
//! 本模块负责标注数据的读写操作，将标注序列化为 JSON 格式存储到本地文件
//! 文件按 PDF 文件名命名，存储在应用数据目录下

use crate::pdf::annotations::{Annotation, PdfAnnotations};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// 标注存储错误类型
#[derive(Debug)]
pub enum StorageError {
    /// IO 错误
    IoError(std::io::Error),
    /// JSON 序列化错误
    JsonError(serde_json::Error),
    /// 目录不存在
    DirectoryNotFound,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::IoError(e) => write!(f, "IO 错误: {}", e),
            StorageError::JsonError(e) => write!(f, "JSON 序列化错误: {}", e),
            StorageError::DirectoryNotFound => write!(f, "目录不存在"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::IoError(err)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::JsonError(err)
    }
}

/// 标注存储管理器
pub struct AnnotationStorage {
    /// 存储目录路径
    storage_dir: PathBuf,
}

impl AnnotationStorage {
    /// 创建新的存储管理器
    ///
    /// # 参数
    /// * `storage_dir` - 存储目录路径
    ///
    /// # 返回值
    /// * `Result<Self, StorageError>` - 成功时返回存储管理器实例
    pub fn new(storage_dir: PathBuf) -> Result<Self, StorageError> {
        // 确保目录存在
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)?;
        }
        Ok(Self { storage_dir })
    }

    /// 根据 PDF 文件路径生成存储文件名（MD5 哈希）
    ///
    /// # 参数
    /// * `pdf_path` - PDF 文件路径
    ///
    /// # 返回值
    /// * `String` - MD5 哈希字符串
    pub fn generate_pdf_key(pdf_path: &str) -> String {
        let mut hasher = DefaultHasher::new();
        pdf_path.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// 获取标注文件的完整路径
    ///
    /// # 参数
    /// * `pdf_key` - PDF 文件密钥
    ///
    /// # 返回值
    /// * `PathBuf` - 标注文件路径
    pub fn get_annotation_file_path(&self, pdf_key: &str) -> PathBuf {
        self.storage_dir.join(format!("{}.json", pdf_key))
    }

    /// 保存 PDF 标注集合到文件
    ///
    /// # 参数
    /// * `annotations` - PDF 标注集合
    ///
    /// # 返回值
    /// * `Result<(), StorageError>` - 成功时返回 ()
    pub fn save(&self, annotations: &PdfAnnotations) -> Result<(), StorageError> {
        let file_path = self.get_annotation_file_path(&annotations.pdf_key);
        let json_content = serde_json::to_string_pretty(annotations)?;
        fs::write(file_path, json_content)?;
        Ok(())
    }

    /// 从文件加载 PDF 标注集合
    ///
    /// # 参数
    /// * `pdf_key` - PDF 文件密钥
    ///
    /// # 返回值
    /// * `Result<Option<PdfAnnotations>, StorageError>` - 成功时返回标注集合，不存在时返回 None
    pub fn load(&self, pdf_key: &str) -> Result<Option<PdfAnnotations>, StorageError> {
        let file_path = self.get_annotation_file_path(pdf_key);
        if !file_path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(file_path)?;
        let annotations = serde_json::from_str::<PdfAnnotations>(&content)?;
        Ok(Some(annotations))
    }

    /// 删除指定 PDF 的标注文件
    ///
    /// # 参数
    /// * `pdf_key` - PDF 文件密钥
    ///
    /// # 返回值
    /// * `Result<bool, StorageError>` - 成功时返回是否删除了文件
    pub fn delete(&self, pdf_key: &str) -> Result<bool, StorageError> {
        let file_path = self.get_annotation_file_path(pdf_key);
        if file_path.exists() {
            fs::remove_file(file_path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 检查指定 PDF 是否有标注文件
    ///
    /// # 参数
    /// * `pdf_key` - PDF 文件密钥
    ///
    /// # 返回值
    /// * `bool` - 标注文件是否存在
    pub fn exists(&self, pdf_key: &str) -> bool {
        self.get_annotation_file_path(pdf_key).exists()
    }

    /// 获取所有标注文件列表
    ///
    /// # 返回值
    /// * `Result<Vec<String>, StorageError>` - PDF 密钥列表
    pub fn list(&self) -> Result<Vec<String>, StorageError> {
        let mut keys = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.storage_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "json" {
                        if let Some(stem) = entry.path().file_stem() {
                            if let Some(key) = stem.to_str() {
                                keys.push(key.to_string());
                            }
                        }
                    }
                }
            }
        }
        Ok(keys)
    }

    /// 保存单条标注到文件
    ///
    /// # 参数
    /// * `pdf_key` - PDF 文件密钥
    /// * `file_name` - PDF 文件名
    /// * `annotation` - 标注数据
    ///
    /// # 返回值
    /// * `Result<(), StorageError>` - 成功时返回 ()
    pub fn save_annotation(
        &self,
        pdf_key: &str,
        file_name: &str,
        annotation: Annotation,
    ) -> Result<(), StorageError> {
        let mut annotations = match self.load(pdf_key)? {
            Some(mut existing) => {
                existing.add_annotation(annotation);
                existing
            }
            None => {
                let mut new_annotations =
                    PdfAnnotations::new(pdf_key.to_string(), file_name.to_string());
                new_annotations.add_annotation(annotation);
                new_annotations
            }
        };
        self.save(&annotations)
    }

    /// 更新单条标注
    ///
    /// # 参数
    /// * `pdf_key` - PDF 文件密钥
    /// * `annotation` - 更新后的标注数据
    ///
    /// # 返回值
    /// * `Result<bool, StorageError>` - 成功时返回是否更新了标注
    pub fn update_annotation(
        &self,
        pdf_key: &str,
        annotation: Annotation,
    ) -> Result<bool, StorageError> {
        if let Some(mut annotations) = self.load(pdf_key)? {
            let updated = annotations.update_annotation(annotation);
            if updated {
                self.save(&annotations)?;
            }
            return Ok(updated);
        }
        Ok(false)
    }

    /// 删除单条标注
    ///
    /// # 参数
    /// * `pdf_key` - PDF 文件密钥
    /// * `annotation_id` - 标注 ID
    ///
    /// # 返回值
    /// * `Result<bool, StorageError>` - 成功时返回是否删除了标注
    pub fn delete_annotation(
        &self,
        pdf_key: &str,
        annotation_id: &str,
    ) -> Result<bool, StorageError> {
        if let Some(mut annotations) = self.load(pdf_key)? {
            let deleted = annotations.remove_annotation(annotation_id);
            if deleted {
                self.save(&annotations)?;
            }
            return Ok(deleted);
        }
        Ok(false)
    }

    /// 获取指定 PDF 的所有标注
    ///
    /// # 参数
    /// * `pdf_key` - PDF 文件密钥
    ///
    /// # 返回值
    /// * `Result<Vec<Annotation>, StorageError>` - 标注列表
    pub fn get_annotations(&self, pdf_key: &str) -> Result<Vec<Annotation>, StorageError> {
        match self.load(pdf_key)? {
            Some(annotations) => Ok(annotations.annotations),
            None => Ok(Vec::new()),
        }
    }

    /// 获取指定 PDF 指定页面的所有标注
    ///
    /// # 参数
    /// * `pdf_key` - PDF 文件密钥
    /// * `page` - 页码（从 1 开始）
    ///
    /// # 返回值
    /// * `Result<Vec<Annotation>, StorageError>` - 标注列表
    pub fn get_annotations_by_page(
        &self,
        pdf_key: &str,
        page: u32,
    ) -> Result<Vec<Annotation>, StorageError> {
        match self.load(pdf_key)? {
            Some(annotations) => Ok(annotations
                .get_annotations_by_page(page)
                .into_iter()
                .cloned()
                .collect()),
            None => Ok(Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::annotations::{
        Annotation, AnnotationData, AnnotationType, HighlightAnnotation, PdfRect,
    };
    use std::env;

    #[test]
    fn test_storage() {
        let temp_dir = env::temp_dir().join("zoplus_test_annotations");
        let storage = AnnotationStorage::new(temp_dir.clone()).unwrap();

        let pdf_key = "test_pdf_key";
        let file_name = "test.pdf";

        let annotation = Annotation {
            id: "test_annotation".to_string(),
            annotation_type: AnnotationType::Highlight,
            color: crate::pdf::annotations::AnnotationColor::default_highlight(),
            page: 1,
            data: AnnotationData::Highlight(HighlightAnnotation {
                rect: PdfRect {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 20.0,
                },
                text: "Test highlight".to_string(),
            }),
            created_at: 0,
            updated_at: 0,
        };

        // 保存标注
        storage
            .save_annotation(pdf_key, file_name, annotation.clone())
            .unwrap();

        // 验证存在
        assert!(storage.exists(pdf_key));

        // 加载标注
        let annotations = storage.get_annotations(pdf_key).unwrap();
        assert_eq!(annotations.len(), 1);

        // 删除标注
        storage.delete(pdf_key).unwrap();
        assert!(!storage.exists(pdf_key));

        // 清理
        let _ = fs::remove_dir_all(temp_dir);
    }
}
