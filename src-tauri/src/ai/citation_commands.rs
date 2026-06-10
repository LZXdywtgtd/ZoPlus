//! 参考文献格式化 Tauri 命令接口
//!
//! 提供参考文献格式化的前端调用接口

use crate::ai::citation::{
    CitationFormat, CitationFormatter, CitationMetadata, FormattedCitation, FormatterConfig,
    FormatterLanguage, ParsedCitation,
};

/// Tauri 命令：解析参考文献文本
///
/// # 参数
/// * `input` - 参考文献原始文本
///
/// # 返回值
/// * `Result<ParsedCitation, String>` - 解析结果
#[tauri::command]
pub fn parse_citation_text(input: String) -> Result<ParsedCitation, String> {
    eprintln!("[命令] parse_citation_text 被调用: {}", &input[..input.len().min(100)]);

    let result = crate::ai::citation::parse_citation(&input);
    Ok(result)
}

/// Tauri 命令：格式化参考文献
///
/// # 参数
/// * `metadata` - 文献元数据
/// * `format` - 目标格式（snake_case 字符串）
///
/// # 返回值
/// * `Result<FormattedCitation, String>` - 格式化结果
#[tauri::command]
pub fn format_citation(metadata: CitationMetadata, format: String) -> Result<FormattedCitation, String> {
    eprintln!("[命令] format_citation 被调用: format={}", format);

    let citation_format = parse_citation_format(&format)?;
    let formatter = CitationFormatter::new(citation_format);
    let result = formatter.format(&metadata);

    eprintln!("[命令] format_citation 完成: formatted_len={}", result.formatted.len());
    Ok(result)
}

/// Tauri 命令：批量格式化参考文献
///
/// # 参数
/// * `metadata_list` - 文献元数据列表
/// * `format` - 目标格式（snake_case 字符串）
///
/// # 返回值
/// * `Result<Vec<FormattedCitation>, String>` - 格式化结果列表
#[tauri::command]
pub fn format_citations_batch(
    metadata_list: Vec<CitationMetadata>,
    format: String,
) -> Result<Vec<FormattedCitation>, String> {
    eprintln!(
        "[命令] format_citations_batch 被调用: count={}, format={}",
        metadata_list.len(),
        format
    );

    let citation_format = parse_citation_format(&format)?;
    let formatter = CitationFormatter::new(citation_format);
    let results = formatter.format_batch(&metadata_list);

    eprintln!("[命令] format_citations_batch 完成: {}", results.len());
    Ok(results)
}

/// Tauri 命令：从 Zotero 数据库补全参考文献元数据
///
/// # 参数
/// * `item_id` - Zotero 文献 ID
/// * `metadata` - 已有元数据（会被补全）
///
/// # 返回值
/// * `Result<CitationMetadata, String>` - 补全后的元数据
#[tauri::command]
pub fn enrich_citation_metadata(
    item_id: i32,
    metadata: CitationMetadata,
) -> Result<CitationMetadata, String> {
    eprintln!("[命令] enrich_citation_metadata 被调用: item_id={}", item_id);

    crate::ai::citation::enrich_metadata_from_zotero(item_id, metadata)
}

/// Tauri 命令：获取所有可用的引用格式
#[tauri::command]
pub fn get_citation_formats() -> Vec<CitationFormatInfo> {
    CitationFormat::all()
        .into_iter()
        .map(|f| CitationFormatInfo {
            id: format!("{:?}", f).to_lowercase(),
            name: f.display_name().to_string(),
        })
        .collect()
}

/// Tauri 命令：设置格式化配置
#[tauri::command]
pub fn create_formatter_with_config(
    format: String,
    use_doi_hyperlink: bool,
    use_url_hyperlink: bool,
    add_access_date: bool,
    use_chinese_punctuation: bool,
    language: String,
) -> Result<CitationFormatterState, String> {
    let citation_format = parse_citation_format(&format)?;
    let formatter_language = match language.to_lowercase().as_str() {
        "chinese" | "zh" => FormatterLanguage::Chinese,
        _ => FormatterLanguage::English,
    };

    let config = FormatterConfig {
        use_doi_hyperlink,
        use_url_hyperlink,
        add_access_date,
        use_chinese_punctuation,
        language: formatter_language,
    };

    let _formatter = CitationFormatter::new(citation_format).with_config(config.clone());

    Ok(CitationFormatterState {
        format: citation_format,
        config: config.clone(),
    })
}

/// 引用格式信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct CitationFormatInfo {
    /// 格式 ID
    pub id: String,
    /// 格式显示名称
    pub name: String,
}

/// 格式化器状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct CitationFormatterState {
    /// 当前格式
    pub format: CitationFormat,
    /// 当前配置
    pub config: FormatterConfig,
}

/// 解析格式字符串为 CitationFormat 枚举
fn parse_citation_format(format: &str) -> Result<CitationFormat, String> {
    match format.to_lowercase().as_str() {
        "apa7" | "apa_7" | "apa" => Ok(CitationFormat::APA7),
        "mla9" | "mla_9" | "mla" => Ok(CitationFormat::MLA9),
        "chicago17" | "chicago_17" | "chicago" => Ok(CitationFormat::Chicago17),
        "gb7714" | "gb_7714" | "gb" => Ok(CitationFormat::GB7714),
        "harvard" => Ok(CitationFormat::Harvard),
        "ieee" => Ok(CitationFormat::IEEE),
        "vancouver" => Ok(CitationFormat::Vancouver),
        "numero" => Ok(CitationFormat::Numero),
        _ => Err(format!("未知的引用格式: {}，可用格式: apa7, mla9, chicago17, gb7714, harvard, ieee, vancouver, numero", format)),
    }
}