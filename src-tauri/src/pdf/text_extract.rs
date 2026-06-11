//! PDF 文本提取模块
//!
//! 本模块负责从 PDF 文件中提取文本内容，支持全文提取和分页提取

use std::path::Path;
use std::process::Command;

/// PDF 文本提取错误类型
#[derive(Debug, thiserror::Error)]
pub enum PdfTextError {
    /// 文件不存在
    #[error("PDF 文件不存在: {0}")]
    FileNotFound(String),
    /// 文件读取失败
    #[error("文件读取失败: {0}")]
    IoError(#[from] std::io::Error),
    /// PDF 文本提取失败
    #[error("文本提取失败: {0}")]
    ExtractionFailed(String),
    /// 未找到 pdftotext 工具
    #[error("未找到 pdftotext 工具，请安装 poppler-utils")]
    PdftotextNotFound,
}

/// 从 PDF 文件提取全文文本
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
///
/// # 返回值
/// * `Result<String, PdfTextError>` - 提取的文本内容
pub fn extract_text_from_pdf(pdf_path: &Path) -> Result<String, PdfTextError> {
    // 检查文件是否存在
    if !pdf_path.exists() {
        return Err(PdfTextError::FileNotFound(
            pdf_path.to_string_lossy().to_string(),
        ));
    }

    // 检查文件扩展名
    if pdf_path.extension().map(|e| e.to_str()) != Some(Some("pdf")) {
        return Err(PdfTextError::ExtractionFailed(
            "文件不是 PDF 格式".to_string(),
        ));
    }

    // 尝试使用 pdftotext 提取文本
    extract_with_pdftotext(pdf_path)
}

/// 使用 pdftotext 工具提取 PDF 文本
fn extract_with_pdftotext(pdf_path: &Path) -> Result<String, PdfTextError> {
    // 检查 pdftotext 是否可用
    let pdftotext_check = Command::new("pdftotext")
        .arg("-v")
        .output();

    if pdftotext_check.is_err() {
        return Err(PdfTextError::PdftotextNotFound);
    }

    // 执行 pdftotext
    let output = Command::new("pdftotext")
        .arg("-layout")  // 保持布局
        .arg("-enc")
        .arg("UTF-8")    // 使用 UTF-8 编码
        .arg(pdf_path)
        .arg("-")        // 输出到 stdout
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PdfTextError::ExtractionFailed(format!(
            "pdftotext 执行失败: {}",
            stderr
        )));
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(text)
}

/// 从 PDF 提取指定页码范围的文本
///
/// # 参数
/// * `pdf_path` - PDF 文件路径
/// * `start_page` - 起始页码（从 1 开始）
/// * `end_page` - 结束页码
///
/// # 返回值
/// * `Result<String, PdfTextError>` - 提取的文本内容
pub fn extract_text_from_pdf_range(
    pdf_path: &Path,
    start_page: u32,
    end_page: u32,
) -> Result<String, PdfTextError> {
    // 检查文件是否存在
    if !pdf_path.exists() {
        return Err(PdfTextError::FileNotFound(
            pdf_path.to_string_lossy().to_string(),
        ));
    }

    // 检查 pdftotext 是否可用
    let pdftotext_check = Command::new("pdftotext")
        .arg("-v")
        .output();

    if pdftotext_check.is_err() {
        return Err(PdfTextError::PdftotextNotFound);
    }

    // 执行 pdftotext，指定页码范围
    // pdftotext 支持使用 -f 和 -l 指定页码范围
    let output = Command::new("pdftotext")
        .arg("-layout")
        .arg("-enc")
        .arg("UTF-8")
        .arg("-f")
        .arg(start_page.to_string())
        .arg("-l")
        .arg(end_page.to_string())
        .arg(pdf_path)
        .arg("-")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PdfTextError::ExtractionFailed(format!(
            "pdftotext 执行失败: {}",
            stderr
        )));
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_error_file_not_found() {
        let result = extract_text_from_pdf(Path::new("/nonexistent/file.pdf"));
        assert!(result.is_err());
        match result {
            Err(PdfTextError::FileNotFound(_)) => {}
            _ => panic!("期望 FileNotFound 错误"),
        }
    }
}