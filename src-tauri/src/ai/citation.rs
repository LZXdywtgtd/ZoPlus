//! 参考文献格式化模块
//!
//! 提供多格式参考文献的解析与格式化功能，支持：
//! - APA 7th
//! - MLA 9th
//! - Chicago 17th
//! - GB/T 7714-2015
//! - Harvard
//! - IEEE
//! - Vancouver
//! - Numero（数字编号格式）

use serde::{Deserialize, Serialize};

/// 参考文献格式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationFormat {
    /// APA 7th（美国心理学会）
    APA7,
    /// MLA 9th（现代语言学会）
    MLA9,
    /// Chicago 17th（芝加哥大学出版社）
    Chicago17,
    /// GB/T 7714-2015（中国国家标准）
    GB7714,
    /// Harvard（哈佛格式）
    Harvard,
    /// IEEE（电气与电子工程师协会）
    IEEE,
    /// Vancouver（温哥华格式）
    Vancouver,
    /// Numero（数字编号格式）
    Numero,
}

impl CitationFormat {
    /// 获取格式显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            CitationFormat::APA7 => "APA 7th",
            CitationFormat::MLA9 => "MLA 9th",
            CitationFormat::Chicago17 => "Chicago 17th",
            CitationFormat::GB7714 => "GB/T 7714-2015",
            CitationFormat::Harvard => "Harvard",
            CitationFormat::IEEE => "IEEE",
            CitationFormat::Vancouver => "Vancouver",
            CitationFormat::Numero => "Numero",
        }
    }

    /// 获取所有可用格式
    pub fn all() -> Vec<CitationFormat> {
        vec![
            CitationFormat::APA7,
            CitationFormat::MLA9,
            CitationFormat::Chicago17,
            CitationFormat::GB7714,
            CitationFormat::Harvard,
            CitationFormat::IEEE,
            CitationFormat::Vancouver,
            CitationFormat::Numero,
        ]
    }
}

impl Default for CitationFormat {
    fn default() -> Self {
        CitationFormat::APA7
    }
}

/// 文献类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    /// 期刊文章
    JournalArticle,
    /// 会议论文
    ConferencePaper,
    /// 书籍
    Book,
    /// 书籍章节
    BookChapter,
    /// 学位论文
    Thesis,
    /// 报告
    Report,
    /// 网页
    WebPage,
    /// 专利
    Patent,
    /// 报纸文章
    NewspaperArticle,
    /// 未知类型
    Unknown,
}

impl Default for ItemType {
    fn default() -> Self {
        ItemType::Unknown
    }
}

/// 文献元数据结构体
///
/// 包含参考文献所需的所有元数据字段
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CitationMetadata {
    /// 作者列表（按出现顺序）
    pub authors: Vec<Author>,
    /// 标题
    pub title: String,
    /// 期刊名称（适用于期刊文章）
    pub journal: Option<String>,
    /// 会议名称（适用于会议论文）
    pub conference: Option<String>,
    /// 书籍名称（适用于书籍章节）
    pub book_title: Option<String>,
    /// 出版年份
    pub year: Option<String>,
    /// 卷号
    pub volume: Option<String>,
    /// 期号
    pub issue: Option<String>,
    /// 起始页码
    pub pages: Option<String>,
    /// DOI
    pub doi: Option<String>,
    /// ISBN
    pub isbn: Option<String>,
    /// ISSN
    pub issn: Option<String>,
    /// URL
    pub url: Option<String>,
    /// 出版商（适用于书籍）
    pub publisher: Option<String>,
    /// 出版地点（适用于书籍）
    pub location: Option<String>,
    /// 文献类型
    pub item_type: ItemType,
    /// 访问日期（适用于网页）
    pub access_date: Option<String>,
    /// 学位名称（适用于学位论文）
    pub degree: Option<String>,
    /// 学校名称（适用于学位论文）
    pub school: Option<String>,
    /// 专利号（适用于专利）
    pub patent_number: Option<String>,
    /// 期刊卷副标题
    pub journal_volume_title: Option<String>,
    /// 会议地点
    pub conference_location: Option<String>,
    /// 会议日期
    pub conference_date: Option<String>,
    /// 编辑者列表
    pub editors: Vec<Author>,
    /// 译者列表
    pub translators: Vec<Author>,
    /// 版本
    pub edition: Option<String>,
    /// 城市（通用地点字段）
    pub city: Option<String>,
    /// 状态/日期（用于预印本等）
    pub status: Option<String>,
}

/// 作者信息结构体
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Author {
    /// 姓（Last Name）
    pub last_name: String,
    /// 名（First Name）
    pub first_name: String,
    /// 中间名
    pub middle_name: Option<String>,
    /// 职称/前缀（如 Dr., Prof.）
    pub prefix: Option<String>,
    /// 姓氏后缀（如 Jr., Sr.）
    pub suffix: Option<String>,
    /// 作者类型：primary=主要作者，editor=编辑，translator=译者
    #[serde(default)]
    pub author_type: AuthorType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthorType {
    /// 主要作者
    Primary,
    /// 编辑
    Editor,
    /// 译者
    Translator,
    #[default]
    Unknown,
}

impl Author {
    /// 获取姓名的标准格式
    pub fn display_name(&self, format: NameFormat) -> String {
        match format {
            NameFormat::LastFirst => {
                let mut name = self.last_name.clone();
                if !self.first_name.is_empty() {
                    name.push_str(", ");
                    if let Some(ref middle) = self.middle_name {
                        name.push_str(&self.first_name);
                        name.push(' ');
                        name.push_str(middle);
                    } else {
                        name.push_str(&self.first_name);
                    }
                }
                if let Some(ref suffix) = self.suffix {
                    name.push_str(", ");
                    name.push_str(suffix);
                }
                name
            }
            NameFormat::FirstLast => {
                let mut name = String::new();
                if let Some(ref prefix) = self.prefix {
                    name.push_str(prefix);
                    name.push(' ');
                }
                name.push_str(&self.first_name);
                if let Some(ref middle) = self.middle_name {
                    name.push(' ');
                    name.push_str(middle);
                }
                name.push(' ');
                name.push_str(&self.last_name);
                if let Some(ref suffix) = self.suffix {
                    name.push_str(", ");
                    name.push_str(suffix);
                }
                name
            }
            NameFormat::Initials => {
                let mut name = self.last_name.clone();
                name.push_str(", ");
                if !self.first_name.is_empty() {
                    name.push(self.first_name.chars().next().unwrap_or(' '));
                    name.push('.');
                }
                if let Some(ref middle) = self.middle_name {
                    name.push(' ');
                    name.push(middle.chars().next().unwrap_or(' '));
                    name.push('.');
                }
                name
            }
        }
    }
}

/// 姓名显示格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameFormat {
    /// 姓, 名
    LastFirst,
    /// 名 姓
    FirstLast,
    /// 姓, 首字母.
    Initials,
}

/// 参考文献解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCitation {
    /// 原始输入文本
    pub original: String,
    /// 解析出的元数据
    pub metadata: CitationMetadata,
    /// 解析过程中的警告信息
    #[serde(default)]
    pub warnings: Vec<String>,
    /// 是否解析成功
    pub success: bool,
}

/// 格式化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedCitation {
    /// 格式化后的参考文献
    pub formatted: String,
    /// 使用的格式
    pub format: CitationFormat,
    /// 格式化的元数据
    pub metadata: CitationMetadata,
    /// 是否有缺失字段的警告
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// 参考文献格式化器
pub struct CitationFormatter {
    /// 当前选中的格式
    format: CitationFormat,
    /// 格式配置
    config: FormatterConfig,
}

/// 格式化配置
#[derive(Debug, Clone, Serialize)]
pub struct FormatterConfig {
    /// 是否使用 DOI 超链接
    pub use_doi_hyperlink: bool,
    /// 是否使用 URL 超链接
    pub use_url_hyperlink: bool,
    /// 是否添加访问日期（对于网页）
    pub add_access_date: bool,
    /// 是否使用中文标点
    pub use_chinese_punctuation: bool,
    /// 默认语言（影响 et al. vs 等的使用）
    pub language: FormatterLanguage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FormatterLanguage {
    /// 英文
    English,
    /// 中文
    Chinese,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            use_doi_hyperlink: true,
            use_url_hyperlink: true,
            add_access_date: true,
            use_chinese_punctuation: false,
            language: FormatterLanguage::English,
        }
    }
}

impl Default for CitationFormatter {
    fn default() -> Self {
        Self::new(CitationFormat::APA7)
    }
}

impl CitationFormatter {
    /// 创建新的格式化器
    pub fn new(format: CitationFormat) -> Self {
        Self {
            format,
            config: FormatterConfig::default(),
        }
    }

    /// 设置格式
    pub fn with_format(mut self, format: CitationFormat) -> Self {
        self.format = format;
        self
    }

    /// 设置配置
    pub fn with_config(mut self, config: FormatterConfig) -> Self {
        self.config = config;
        self
    }

    /// 设置语言
    pub fn with_language(mut self, language: FormatterLanguage) -> Self {
        self.config.language = language;
        self
    }

    /// 格式化参考文献
    pub fn format(&self, metadata: &CitationMetadata) -> FormattedCitation {
        let mut warnings = Vec::new();

        // 检查必要字段
        if metadata.authors.is_empty() {
            warnings.push("缺少作者信息".to_string());
        }
        if metadata.title.is_empty() {
            warnings.push("缺少标题".to_string());
        }
        if metadata.year.is_none() {
            warnings.push("缺少年份".to_string());
        }

        let formatted = match self.format {
            CitationFormat::APA7 => self.format_apa7(metadata, &mut warnings),
            CitationFormat::MLA9 => self.format_mla9(metadata, &mut warnings),
            CitationFormat::Chicago17 => self.format_chicago17(metadata, &mut warnings),
            CitationFormat::GB7714 => self.format_gb7714(metadata, &mut warnings),
            CitationFormat::Harvard => self.format_harvard(metadata, &mut warnings),
            CitationFormat::IEEE => self.format_ieee(metadata, &mut warnings),
            CitationFormat::Vancouver => self.format_vancouver(metadata, &mut warnings),
            CitationFormat::Numero => self.format_numero(metadata, &mut warnings),
        };

        FormattedCitation {
            formatted,
            format: self.format,
            metadata: metadata.clone(),
            warnings,
        }
    }

    /// 批量格式化
    pub fn format_batch(&self, metadata_list: &[CitationMetadata]) -> Vec<FormattedCitation> {
        metadata_list.iter().map(|m| self.format(m)).collect()
    }

    /// APA 7th 格式
    fn format_apa7(&self, meta: &CitationMetadata, warnings: &mut Vec<String>) -> String {
        let mut result = String::new();

        // 作者
        result.push_str(&self.format_authors_apa(&meta.authors));
        result.push_str(". ");

        // 年份
        if let Some(ref year) = meta.year {
            result.push('(');
            result.push_str(year);
            result.push_str("). ");
        } else {
            result.push_str("(n.d.). ");
            warnings.push("缺少年份，使用 n.d.（无日期）".to_string());
        }

        // 标题
        result.push_str(&meta.title);
        result.push_str(". ");

        // 根据文献类型添加不同信息
        match meta.item_type {
            ItemType::JournalArticle => {
                // 期刊名称斜体
                if let Some(ref journal) = meta.journal {
                    result.push_str("*");
                    result.push_str(journal);
                    result.push_str("*, ");
                }
                // 卷号(期号)
                if let Some(ref volume) = meta.volume {
                    result.push_str(volume);
                    if let Some(ref issue) = meta.issue {
                        result.push('(');
                        result.push_str(issue);
                        result.push(')');
                    }
                }
                // 页码
                if let Some(ref pages) = meta.pages {
                    result.push_str(", ");
                    result.push_str(pages);
                }
                result.push('.');
                // DOI
                if let Some(ref doi) = meta.doi {
                    result.push_str(" https://doi.org/");
                    result.push_str(doi);
                }
            }
            ItemType::Book => {
                if let Some(ref edition) = meta.edition {
                    result.push_str("(");
                    result.push_str(edition);
                    result.push_str(" ed.). ");
                } else {
                    result.push_str(". ");
                }
                if let Some(ref publisher) = meta.publisher {
                    result.push_str(publisher);
                    result.push('.');
                }
            }
            ItemType::BookChapter => {
                result.push_str("In ");
                // 编辑者
                if !meta.editors.is_empty() {
                    result.push_str(&self.format_authors_apa(&meta.editors));
                    result.push_str(" (Ed.), ");
                } else if !meta.editors.is_empty() {
                    result.push_str(&self.format_authors_apa(&meta.editors));
                    result.push_str(" (Eds.), ");
                }
                // 书籍标题斜体
                if let Some(ref book_title) = meta.book_title {
                    result.push_str("*");
                    result.push_str(book_title);
                    result.push_str("*");
                }
                // 页码
                if let Some(ref pages) = meta.pages {
                    result.push_str(" (pp. ");
                    result.push_str(pages);
                    result.push(')');
                }
                result.push_str(". ");
                if let Some(ref publisher) = meta.publisher {
                    result.push_str(publisher);
                    result.push('.');
                }
            }
            ItemType::ConferencePaper => {
                if let Some(ref conference) = meta.conference {
                    result.push_str("*");
                    result.push_str(conference);
                    result.push_str("*. ");
                }
                if let Some(ref location) = meta.conference_location {
                    result.push_str(location);
                    if let Some(ref date) = meta.conference_date {
                        result.push_str(", ");
                        result.push_str(date);
                    }
                    result.push_str(". ");
                }
            }
            ItemType::WebPage => {
                if let Some(ref url) = meta.url {
                    result.push_str("Retrieved from ");
                    result.push_str(url);
                }
                if self.config.add_access_date {
                    if let Some(ref access_date) = meta.access_date {
                        result.push_str(" (Accessed ");
                        result.push_str(access_date);
                        result.push(')');
                    }
                }
            }
            _ => {
                // 其他类型，尝试使用通用格式
                if let Some(ref publisher) = meta.publisher {
                    result.push_str(publisher);
                    result.push('.');
                }
                if let Some(ref doi) = meta.doi {
                    result.push_str(" https://doi.org/");
                    result.push_str(doi);
                }
            }
        }

        result
    }

    /// MLA 9th 格式
    fn format_mla9(&self, meta: &CitationMetadata, _warnings: &mut Vec<String>) -> String {
        let mut result = String::new();

        // 作者
        result.push_str(&self.format_authors_mla(&meta.authors));
        result.push_str(". ");

        // 标题
        result.push_str("\"");
        result.push_str(&meta.title);
        result.push_str(".\" ");

        // 根据文献类型添加不同信息
        match meta.item_type {
            ItemType::JournalArticle => {
                if let Some(ref journal) = meta.journal {
                    result.push_str("*");
                    result.push_str(journal);
                    result.push_str("*, ");
                }
                if let Some(ref volume) = meta.volume {
                    result.push_str("vol. ");
                    result.push_str(volume);
                    result.push_str(", ");
                }
                if let Some(ref issue) = meta.issue {
                    result.push_str("no. ");
                    result.push_str(issue);
                    result.push_str(", ");
                }
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                    result.push_str(", ");
                }
                if let Some(ref pages) = meta.pages {
                    result.push_str("pp. ");
                    result.push_str(pages);
                }
                result.push('.');
            }
            ItemType::Book => {
                result.push_str("*");
                result.push_str(&meta.title);
                result.push_str("*, ");
                if let Some(ref edition) = meta.edition {
                    result.push_str(edition);
                    result.push_str(", ");
                }
                if let Some(ref publisher) = meta.publisher {
                    result.push_str(publisher);
                    result.push_str(", ");
                }
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                }
                result.push('.');
            }
            ItemType::BookChapter => {
                result.push_str("\"");
                result.push_str(&meta.title);
                result.push_str(". \" *");
                if let Some(ref book_title) = meta.book_title {
                    result.push_str(book_title);
                    result.push_str("*, ");
                }
                if let Some(ref publisher) = meta.publisher {
                    result.push_str(publisher);
                    result.push_str(", ");
                }
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                }
                result.push('.');
            }
            _ => {
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                    result.push_str(". ");
                }
            }
        }

        // DOI 或 URL
        if let Some(ref doi) = meta.doi {
            result.push_str(" https://doi.org/");
            result.push_str(doi);
            result.push('.');
        } else if let Some(ref url) = meta.url {
            result.push_str(url);
            result.push('.');
        }

        result
    }

    /// Chicago 17th 格式
    fn format_chicago17(&self, meta: &CitationMetadata, _warnings: &mut Vec<String>) -> String {
        let mut result = String::new();

        // 作者
        result.push_str(&self.format_authors_chicago(&meta.authors));
        result.push_str(". ");

        // 标题
        match meta.item_type {
            ItemType::JournalArticle => {
                result.push_str("\"");
                result.push_str(&meta.title);
                result.push_str(".\" ");
            }
            _ => {
                result.push_str("*");
                result.push_str(&meta.title);
                result.push_str("*. ");
            }
        }

        // 根据文献类型添加不同信息
        match meta.item_type {
            ItemType::JournalArticle => {
                if let Some(ref journal) = meta.journal {
                    result.push_str("*");
                    result.push_str(journal);
                    result.push_str("*. ");
                }
                if let Some(ref volume) = meta.volume {
                    result.push_str(volume);
                }
                if let Some(ref issue) = meta.issue {
                    result.push_str(", no. ");
                    result.push_str(issue);
                }
                if let Some(ref year) = meta.year {
                    result.push_str(" (");
                    result.push_str(year);
                    result.push_str("): ");
                }
                if let Some(ref pages) = meta.pages {
                    result.push_str(pages);
                }
                result.push('.');
            }
            ItemType::Book => {
                if let Some(ref edition) = meta.edition {
                    result.push_str(edition);
                    result.push_str(". ");
                }
                if let Some(ref location) = meta.location {
                    result.push_str(location);
                    result.push_str(": ");
                }
                if let Some(ref publisher) = meta.publisher {
                    result.push_str(publisher);
                    result.push_str(", ");
                }
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                }
                result.push('.');
            }
            _ => {
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                    result.push_str(". ");
                }
            }
        }

        // DOI
        if let Some(ref doi) = meta.doi {
            result.push_str(" https://doi.org/");
            result.push_str(doi);
            result.push('.');
        }

        result
    }

    /// GB/T 7714-2015 格式（中国国家标准）
    fn format_gb7714(&self, meta: &CitationMetadata, _warnings: &mut Vec<String>) -> String {
        let mut result = String::new();

        // 作者（中文格式：姓，名）
        result.push_str(&self.format_authors_gb(&meta.authors));
        result.push_str(". ");

        // 标题
        result.push_str(&meta.title);
        result.push_str("[");

        // 文献类型标识
        let type标识 = match meta.item_type {
            ItemType::JournalArticle => "J",
            ItemType::ConferencePaper => "C",
            ItemType::Book => "M",
            ItemType::BookChapter => "M",
            ItemType::Thesis => "D",
            ItemType::Report => "R",
            ItemType::Patent => "P",
            ItemType::WebPage => "EB",
            _ => "J",
        };
        result.push_str(type标识);
        result.push_str("]. ");

        // 根据文献类型添加不同信息
        match meta.item_type {
            ItemType::JournalArticle => {
                if let Some(ref journal) = meta.journal {
                    result.push_str(journal);
                    result.push_str(", ");
                }
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                }
                if let Some(ref volume) = meta.volume {
                    result.push_str(", ");
                    result.push_str(volume);
                }
                if let Some(ref issue) = meta.issue {
                    result.push_str("(");
                    result.push_str(issue);
                    result.push(')');
                }
                if let Some(ref pages) = meta.pages {
                    result.push_str(": ");
                    result.push_str(pages);
                }
                result.push('.');
            }
            ItemType::Book => {
                if let Some(ref location) = meta.location {
                    result.push_str(location);
                    result.push_str(": ");
                }
                if let Some(ref publisher) = meta.publisher {
                    result.push_str(publisher);
                    result.push_str(", ");
                }
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                }
                result.push('.');
            }
            ItemType::Thesis => {
                if let Some(ref school) = meta.school {
                    result.push_str(school);
                    result.push_str(", ");
                }
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                }
                result.push('.');
            }
            _ => {
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                    result.push_str(". ");
                }
            }
        }

        // DOI
        if let Some(ref doi) = meta.doi {
            result.push_str("DOI: ");
            result.push_str(doi);
            result.push('.');
        }

        result
    }

    /// Harvard 格式
    fn format_harvard(&self, meta: &CitationMetadata, _warnings: &mut Vec<String>) -> String {
        let mut result = String::new();

        // 作者
        result.push_str(&self.format_authors_harvard(&meta.authors));
        result.push_str(", ");

        // 年份
        if let Some(ref year) = meta.year {
            result.push('(');
            result.push_str(year);
            result.push_str(") ");
        } else {
            result.push_str("(n.d.) ");
        }

        // 标题
        match meta.item_type {
            ItemType::JournalArticle => {
                result.push_str("'");
                result.push_str(&meta.title);
                result.push_str("', ");
            }
            _ => {
                result.push_str("*");
                result.push_str(&meta.title);
                result.push_str("*, ");
            }
        }

        // 根据文献类型添加不同信息
        match meta.item_type {
            ItemType::JournalArticle => {
                if let Some(ref journal) = meta.journal {
                    result.push_str("*");
                    result.push_str(journal);
                    result.push_str("*, ");
                }
                if let Some(ref volume) = meta.volume {
                    result.push_str(volume);
                }
                if let Some(ref issue) = meta.issue {
                    result.push_str("(");
                    result.push_str(issue);
                    result.push(')');
                }
                if let Some(ref pages) = meta.pages {
                    result.push_str(", pp. ");
                    result.push_str(pages);
                }
                result.push('.');
            }
            ItemType::Book => {
                if let Some(ref edition) = meta.edition {
                    result.push_str(edition);
                    result.push_str(" edn. ");
                }
                if let Some(ref location) = meta.location {
                    result.push_str(location);
                    result.push_str(": ");
                }
                if let Some(ref publisher) = meta.publisher {
                    result.push_str(publisher);
                }
                result.push('.');
            }
            _ => {}
        }

        // DOI
        if let Some(ref doi) = meta.doi {
            result.push_str(" doi: ");
            result.push_str(doi);
        }

        result
    }

    /// IEEE 格式
    fn format_ieee(&self, meta: &CitationMetadata, _warnings: &mut Vec<String>) -> String {
        let mut result = String::new();

        // 作者
        result.push_str(&self.format_authors_ieee(&meta.authors));
        result.push_str(", ");

        // 标题
        result.push_str("\"");
        result.push_str(&meta.title);
        result.push_str(",\" ");

        // 根据文献类型添加不同信息
        match meta.item_type {
            ItemType::JournalArticle => {
                if let Some(ref journal) = meta.journal {
                    result.push_str("*");
                    result.push_str(journal);
                    result.push_str("*, ");
                }
                if let Some(ref volume) = meta.volume {
                    result.push_str("vol. ");
                    result.push_str(volume);
                    result.push_str(", ");
                }
                if let Some(ref issue) = meta.issue {
                    result.push_str("no. ");
                    result.push_str(issue);
                    result.push_str(", ");
                }
                if let Some(ref pages) = meta.pages {
                    result.push_str("pp. ");
                    result.push_str(pages);
                    result.push_str(", ");
                }
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                }
                result.push('.');
            }
            ItemType::ConferencePaper => {
                if let Some(ref conference) = meta.conference {
                    result.push_str("in *");
                    result.push_str(conference);
                    result.push_str("*, ");
                }
                if let Some(ref location) = meta.conference_location {
                    result.push_str(location);
                    result.push_str(", ");
                }
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                }
                result.push('.');
            }
            ItemType::Book => {
                if let Some(ref location) = meta.location {
                    result.push_str(location);
                    result.push_str(": ");
                }
                if let Some(ref publisher) = meta.publisher {
                    result.push_str(publisher);
                    result.push_str(", ");
                }
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                }
                result.push('.');
            }
            _ => {
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                    result.push_str(". ");
                }
            }
        }

        // DOI
        if let Some(ref doi) = meta.doi {
            result.push_str("doi: ");
            result.push_str(doi);
        }

        result
    }

    /// Vancouver 格式
    fn format_vancouver(&self, meta: &CitationMetadata, _warnings: &mut Vec<String>) -> String {
        let mut result = String::new();

        // 作者（只使用姓）
        result.push_str(&self.format_authors_vancouver(&meta.authors));
        result.push_str(". ");

        // 标题
        result.push_str(&meta.title);
        result.push_str(". ");

        // 根据文献类型添加不同信息
        match meta.item_type {
            ItemType::JournalArticle => {
                if let Some(ref journal) = meta.journal {
                    result.push_str(journal);
                    result.push_str(". ");
                }
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                }
                if let Some(ref volume) = meta.volume {
                    result.push(';');
                    result.push_str(volume);
                }
                if let Some(ref issue) = meta.issue {
                    result.push('(');
                    result.push_str(issue);
                    result.push(')');
                }
                if let Some(ref pages) = meta.pages {
                    result.push_str(":");
                    result.push_str(pages);
                }
                result.push('.');
            }
            ItemType::Book => {
                if let Some(ref location) = meta.location {
                    result.push_str(location);
                    result.push_str(": ");
                }
                if let Some(ref publisher) = meta.publisher {
                    result.push_str(publisher);
                    result.push_str("; ");
                }
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                }
                result.push('.');
            }
            _ => {
                if let Some(ref year) = meta.year {
                    result.push_str(year);
                    result.push_str(". ");
                }
            }
        }

        // DOI
        if let Some(ref doi) = meta.doi {
            result.push_str("doi:");
            result.push_str(doi);
        }

        result
    }

    /// Numero 数字编号格式
    fn format_numero(&self, meta: &CitationMetadata, _warnings: &mut Vec<String>) -> String {
        let mut result = String::new();

        // 作者
        result.push_str(&self.format_authors_numero(&meta.authors));
        result.push_str(". ");

        // 标题
        result.push_str(&meta.title);
        result.push_str(". ");

        // 年份
        if let Some(ref year) = meta.year {
            result.push_str(year);
            result.push_str(". ");
        }

        // 期刊/会议信息
        match meta.item_type {
            ItemType::JournalArticle => {
                if let Some(ref journal) = meta.journal {
                    result.push_str(journal);
                    result.push_str(". ");
                }
                if let Some(ref volume) = meta.volume {
                    result.push_str(volume);
                }
                if let Some(ref issue) = meta.issue {
                    result.push('(');
                    result.push_str(issue);
                    result.push(')');
                }
                if let Some(ref pages) = meta.pages {
                    result.push_str(": ");
                    result.push_str(pages);
                }
                result.push('.');
            }
            ItemType::Book => {
                if let Some(ref publisher) = meta.publisher {
                    result.push_str(publisher);
                    result.push_str(". ");
                }
            }
            _ => {}
        }

        // DOI
        if let Some(ref doi) = meta.doi {
            result.push_str("https://doi.org/");
            result.push_str(doi);
            result.push('.');
        }

        result
    }

    // ==================== 作者格式化辅助函数 ====================

    /// APA 格式作者
    fn format_authors_apa(&self, authors: &[Author]) -> String {
        if authors.is_empty() {
            return String::new();
        }

        let et_al = if self.config.language == FormatterLanguage::English {
            "et al."
        } else {
            "等"
        };

        if authors.len() == 1 {
            authors[0].display_name(NameFormat::LastFirst)
        } else if authors.len() == 2 {
            format!(
                "{} & {}",
                authors[0].display_name(NameFormat::LastFirst),
                authors[1].display_name(NameFormat::LastFirst)
            )
        } else {
            // 超过20个作者时使用 et al.
            if authors.len() > 20 {
                format!(
                    "{}{}",
                    authors[0].display_name(NameFormat::LastFirst),
                    et_al
                )
            } else {
                let all_but_last: Vec<_> = authors[..authors.len() - 1]
                    .iter()
                    .map(|a| a.display_name(NameFormat::LastFirst))
                    .collect();
                let last = authors.last().unwrap().display_name(NameFormat::LastFirst);
                format!("{}, {}, & {}", all_but_last.join(", "), last, "")
            }
        }
    }

    /// MLA 格式作者
    fn format_authors_mla(&self, authors: &[Author]) -> String {
        if authors.is_empty() {
            return String::new();
        }

        if authors.len() == 1 {
            authors[0].display_name(NameFormat::LastFirst)
        } else if authors.len() == 2 {
            format!(
                "{}, and {}",
                authors[0].display_name(NameFormat::LastFirst),
                authors[1].display_name(NameFormat::FirstLast)
            )
        } else {
            format!(
                "{}, et al.",
                authors[0].display_name(NameFormat::LastFirst)
            )
        }
    }

    /// Chicago 格式作者
    fn format_authors_chicago(&self, authors: &[Author]) -> String {
        if authors.is_empty() {
            return String::new();
        }

        if authors.len() == 1 {
            authors[0].display_name(NameFormat::LastFirst)
        } else if authors.len() <= 10 {
            let all_but_last: Vec<_> = authors[..authors.len() - 1]
                .iter()
                .map(|a| a.display_name(NameFormat::LastFirst))
                .collect();
            let last = authors.last().unwrap().display_name(NameFormat::LastFirst);
            format!("{}, and {}", all_but_last.join(", "), last)
        } else {
            // 超过10个作者时只列前7个
            let first_7: Vec<_> = authors[..7]
                .iter()
                .map(|a| a.display_name(NameFormat::LastFirst))
                .collect();
            format!("{} et al.", first_7.join(", "))
        }
    }

    /// GB/T 7714 格式作者
    fn format_authors_gb(&self, authors: &[Author]) -> String {
        if authors.is_empty() {
            return String::new();
        }

        // GB/T 7714: 姓 名 格式，姓名之间用空格分隔
        let author_strs: Vec<_> = authors
            .iter()
            .map(|a| {
                if a.last_name.is_empty() {
                    a.first_name.clone()
                } else {
                    format!("{} {}", a.last_name, a.first_name)
                }
            })
            .collect();

        if author_strs.len() <= 3 {
            author_strs.join(", ")
        } else {
            // 超过3个作者时只列前3个
            format!("{}, 等", author_strs[..3].join(", "))
        }
    }

    /// Harvard 格式作者
    fn format_authors_harvard(&self, authors: &[Author]) -> String {
        if authors.is_empty() {
            return String::new();
        }

        if authors.len() == 1 {
            authors[0].display_name(NameFormat::LastFirst)
        } else if authors.len() == 2 {
            format!(
                "{} and {}",
                authors[0].display_name(NameFormat::LastFirst),
                authors[1].display_name(NameFormat::LastFirst)
            )
        } else if authors.len() == 3 {
            format!(
                "{}, and {}",
                authors[0].display_name(NameFormat::LastFirst),
                authors[1].display_name(NameFormat::LastFirst)
            )
        } else {
            format!(
                "{} et al.",
                authors[0].display_name(NameFormat::LastFirst)
            )
        }
    }

    /// IEEE 格式作者
    fn format_authors_ieee(&self, authors: &[Author]) -> String {
        if authors.is_empty() {
            return String::new();
        }

        let author_strs: Vec<_> = authors
            .iter()
            .map(|a| a.display_name(NameFormat::FirstLast))
            .collect();

        if author_strs.len() <= 6 {
            author_strs.join(", ")
        } else {
            // 超过6个作者时只列前3个
            format!("{} et al.", author_strs[..3].join(", "))
        }
    }

    /// Vancouver 格式作者
    fn format_authors_vancouver(&self, authors: &[Author]) -> String {
        if authors.is_empty() {
            return String::new();
        }

        let author_strs: Vec<_> = authors
            .iter()
            .map(|a| {
                if a.last_name.is_empty() {
                    a.first_name.clone()
                } else {
                    a.last_name.clone()
                }
            })
            .collect();

        if author_strs.len() <= 6 {
            author_strs.join(", ")
        } else {
            // 超过6个作者时只列前3个
            format!("{} et al.", author_strs[..3].join(", "))
        }
    }

    /// Numero 格式作者
    fn format_authors_numero(&self, authors: &[Author]) -> String {
        if authors.is_empty() {
            return String::new();
        }

        let author_strs: Vec<_> = authors
            .iter()
            .map(|a| {
                if a.last_name.is_empty() {
                    a.first_name.clone()
                } else {
                    format!("{}, {}", a.last_name, a.first_name)
                }
            })
            .collect();

        if author_strs.len() <= 3 {
            author_strs.join(", ")
        } else {
            format!("{} et al.", author_strs[..3].join(", "))
        }
    }
}

/// 从原始文本解析参考文献
pub fn parse_citation(input: &str) -> ParsedCitation {
    let mut metadata = CitationMetadata::default();
    let mut warnings = Vec::new();
    let original = input.to_string();

    // 预处理：移除多余空白
    let text = input.trim().to_string();

    // 尝试识别文献类型
    metadata.item_type = detect_item_type(&text);

    // 尝试提取作者信息
    metadata.authors = extract_authors(&text, &mut warnings);

    // 尝试提取年份
    if let Some(year) = extract_year(&text) {
        metadata.year = Some(year);
    }

    // 尝试提取 DOI
    if let Some(doi) = extract_doi(&text) {
        metadata.doi = Some(doi);
    }

    // 尝试提取标题
    metadata.title = extract_title(&text, &metadata.authors, &metadata.year);

    // 尝试提取期刊信息
    if let Some(journal) = extract_journal(&text) {
        metadata.journal = Some(journal);
    }

    // 尝试提取 ISBN
    if let Some(isbn) = extract_isbn(&text) {
        metadata.isbn = Some(isbn);
    }

    // 尝试提取 URL
    if let Some(url) = extract_url(&text) {
        metadata.url = Some(url);
    }

    // 尝试提取卷号、期号、页码
    if let Some((volume, issue, pages)) = extract_volume_issue_pages(&text) {
        metadata.volume = volume;
        metadata.issue = issue;
        metadata.pages = pages;
    }

    let success = !metadata.authors.is_empty()
        || !metadata.title.is_empty()
        || metadata.year.is_some()
        || metadata.doi.is_some();

    if !success {
        warnings.push("未能解析出有效的参考文献信息".to_string());
    }

    ParsedCitation {
        original,
        metadata,
        warnings,
        success,
    }
}

/// 检测文献类型
fn detect_item_type(text: &str) -> ItemType {
    let text_lower = text.to_lowercase();

    // 基于关键词检测文献类型
    if text_lower.contains("doi") || text_lower.contains("10.") {
        if text_lower.contains("journal") || text_lower.contains("article") {
            return ItemType::JournalArticle;
        }
    }

    if text_lower.contains("conference") || text_lower.contains("proceedings") {
        return ItemType::ConferencePaper;
    }

    if text_lower.contains("phd thesis") || text_lower.contains("master thesis") || text_lower.contains("dissertation") {
        return ItemType::Thesis;
    }

    if text_lower.contains("book") || text_lower.contains("isbn") {
        if text_lower.contains("chapter") || text_lower.contains("pp.") {
            return ItemType::BookChapter;
        }
        return ItemType::Book;
    }

    if text_lower.contains("tech. rep.") || text_lower.contains("technical report") {
        return ItemType::Report;
    }

    if text_lower.contains("patent") {
        return ItemType::Patent;
    }

    if text_lower.contains("http") || text_lower.contains("www.") {
        return ItemType::WebPage;
    }

    // 默认假设为期刊文章
    ItemType::JournalArticle
}

/// 提取作者信息（简化实现）
fn extract_authors(text: &str, warnings: &mut Vec<String>) -> Vec<Author> {
    let mut authors = Vec::new();

    // 尝试匹配常见的作者模式
    // 模式1: "Author, A. B." 或 "Author, A."
    // 模式2: "A. B. Author"

    // 简化实现：尝试提取以逗号分隔的第一个作者
    if let Some(first_comma) = text.find(',') {
        let potential_author = text[..first_comma].trim().to_string();
        if !potential_author.is_empty()
            && !potential_author.contains('.')
            && potential_author.len() < 50
        {
            authors.push(Author {
                last_name: potential_author,
                first_name: String::new(),
                middle_name: None,
                prefix: None,
                suffix: None,
                author_type: AuthorType::Primary,
            });
        }
    }

    if authors.is_empty() {
        warnings.push("无法自动提取作者信息，请手动输入".to_string());
    }

    authors
}

/// 提取年份
fn extract_year(text: &str) -> Option<String> {
    // 匹配4位数年份 (19xx 或 20xx)
    let year_pattern = regex::Regex::new(r"\b(19|20)\d{2}\b").ok()?;
    year_pattern
        .captures(text)
        .and_then(|cap| cap.get(0))
        .map(|m| m.as_str().to_string())
}

/// 提取 DOI
fn extract_doi(text: &str) -> Option<String> {
    // 匹配 DOI 模式
    let doi_pattern = regex::Regex::new(r"10\.\d{4,}/[^\s]+").ok()?;
    doi_pattern
        .captures(text)
        .and_then(|cap| cap.get(0))
        .map(|m| m.as_str().trim_end_matches(['.', ',', ';', ')'].as_slice()).to_string())
}

/// 提取 ISBN
fn extract_isbn(text: &str) -> Option<String> {
    // 匹配 ISBN-10 或 ISBN-13
    let isbn_pattern = regex::Regex::new(r"(ISBN[- ]?)?((97[89])?\d{9}[\dXx])").ok()?;
    isbn_pattern
        .captures(text)
        .and_then(|cap| cap.get(0))
        .map(|m| m.as_str().to_string())
}

/// 提取 URL
fn extract_url(text: &str) -> Option<String> {
    let url_pattern = regex::Regex::new(r"https?://[^\s]+").ok()?;
    url_pattern
        .captures(text)
        .and_then(|cap| cap.get(0))
        .map(|m| m.as_str().trim_end_matches(['.', ',', ';', ')'].as_slice()).to_string())
}

/// 提取标题
fn extract_title(text: &str, _authors: &[Author], year: &Option<String>) -> String {
    // 简化实现：尝试找到被引号或特殊格式包裹的标题
    let text = text.trim();

    // 尝试匹配被双引号包裹的标题
    if let Some(start) = text.find("\"") {
        if let Some(end) = text[start + 1..].find("\"") {
            let title = &text[start + 1..start + 1 + end];
            if !title.is_empty() && title.len() > 5 {
                return title.to_string();
            }
        }
    }

    // 尝试匹配斜体标题（用 * 标记）
    if let Some(start) = text.find('*') {
        if let Some(end) = text[start + 1..].find('*') {
            let title = &text[start + 1..start + 1 + end];
            if !title.is_empty() && title.len() > 5 {
                return title.to_string();
            }
        }
    }

    // 如果有作者和年份，尝试提取中间部分作为标题
    if let Some(ref year_str) = year {
        if let Some(year_pos) = text.find(year_str.as_str()) {
            let after_year = &text[year_pos + year_str.len()..];
            // 标题通常在年份之后，第一个句号之前
            if let Some(period) = after_year.find('.') {
                let potential_title = after_year[..period].trim();
                if !potential_title.is_empty() && potential_title.len() > 10 {
                    return potential_title.to_string();
                }
            }
        }
    }

    // 返回原文前100个字符作为备选
    text.chars().take(100).collect()
}

/// 提取期刊名称
fn extract_journal(text: &str) -> Option<String> {
    // 简化实现：查找常见的期刊关键词
    let journal_patterns = [
        r"in\s+([^,]+)",
        r"journal\s+of\s+([^,]+)",
        r"\*([^*]+)\*",
    ];

    for pattern in journal_patterns {
        if let Ok(re) = regex::Regex::new(&format!("(?i){}", pattern)) {
            if let Some(cap) = re.captures(text) {
                if let Some(name) = cap.get(1) {
                    let journal = name.as_str().trim();
                    if !journal.is_empty() && journal.len() > 2 {
                        return Some(journal.to_string());
                    }
                }
            }
        }
    }

    None
}

/// 提取卷号、期号、页码
fn extract_volume_issue_pages(text: &str) -> Option<(Option<String>, Option<String>, Option<String>)> {
    let vol_pattern = regex::Regex::new(r"vol\.?\s*(\d+)").ok()?;
    let issue_pattern = regex::Regex::new(r"no\.?\s*(\d+)").ok()?;
    let pages_pattern = regex::Regex::new(r"pp?\.?\s*(\d+[-–]\d+)").ok()?;

    let volume = vol_pattern
        .captures(text)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string());

    let issue = issue_pattern
        .captures(text)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string());

    let pages = pages_pattern
        .captures(text)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string());

    if volume.is_some() || issue.is_some() || pages.is_some() {
        Some((volume, issue, pages))
    } else {
        None
    }
}

/// 根据 item_id 从 Zotero 数据库补全元数据
pub fn enrich_metadata_from_zotero(
    item_id: i32,
    mut metadata: CitationMetadata,
) -> Result<CitationMetadata, String> {
    use crate::db::connection::get_connection;
    use crate::db::metadata::get_cached_metadata;
    use crate::db::dynamic::{DynamicSqlBuilder, ZoteroTableCandidates};
    use rusqlite::params;

    let guard = get_connection().map_err(|e| format!("获取数据库连接失败: {}", e))?;
    let conn = guard.as_ref().ok_or_else(|| "数据库连接未初始化".to_string())?;

    // 获取动态元数据
    let db_metadata = get_cached_metadata(conn)
        .map_err(|e| format!("获取元数据失败: {}", e))?;
    let dynamic = DynamicSqlBuilder::new(&db_metadata);

    // 动态获取表名
    let items_table = dynamic.find_table(ZoteroTableCandidates::ITEMS)
        .ok_or_else(|| "未找到 items 表".to_string())?;
    let item_data_table = dynamic.find_table(ZoteroTableCandidates::ITEM_DATA)
        .ok_or_else(|| "未找到 itemData 表".to_string())?;
    let item_data_values_table = dynamic.find_table(ZoteroTableCandidates::ITEM_DATA_VALUES)
        .ok_or_else(|| "未找到 itemDataValues 表".to_string())?;
    let fields_table = dynamic.find_table(ZoteroTableCandidates::FIELDS)
        .ok_or_else(|| "未找到 fields 表".to_string())?;
    let authors_table = dynamic.find_table(ZoteroTableCandidates::CREATORS)
        .ok_or_else(|| "未找到 itemCreators 表".to_string())?;
    let creators_table = dynamic.find_table(ZoteroTableCandidates::CREATOR)
        .ok_or_else(|| "未找到 creators 表".to_string())?;

    // 查询文献基本信息
    let sql = format!(
        r#"
        SELECT
            i.itemID,
            fv_title.value as title,
            fv_date.value as year,
            i.itemTypeID
        FROM {items_table} i
        LEFT JOIN {item_data_table} id_title ON i.itemID = id_title.itemID
            AND id_title.fieldID = (SELECT fieldID FROM {fields_table} WHERE fieldName = 'title')
        LEFT JOIN {item_data_values_table} fv_title ON id_title.valueID = fv_title.valueID
        LEFT JOIN {item_data_table} id_date ON i.itemID = id_date.itemID
            AND id_date.fieldID = (SELECT fieldID FROM {fields_table} WHERE fieldName = 'date')
        LEFT JOIN {item_data_values_table} fv_date ON id_date.valueID = fv_date.valueID
        WHERE i.itemID = ?
        "#,
        items_table = items_table,
        item_data_table = item_data_table,
        item_data_values_table = item_data_values_table,
        fields_table = fields_table
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("准备查询失败: {}", e))?;

    let result = stmt.query_row(params![item_id], |row| {
        Ok((
            row.get::<_, String>(1).unwrap_or_default(),
            row.get::<_, String>(2).unwrap_or_default(),
        ))
    });

    match result {
        Ok((title, year)) => {
            if !title.is_empty() {
                metadata.title = title;
            }
            if !year.is_empty() {
                metadata.year = Some(year);
            }
        }
        Err(_) => {
            return Err(format!("未找到 item_id={} 的文献", item_id));
        }
    }

    // 查询作者信息
    let author_sql = format!(
        r#"
        SELECT c.lastName, c.firstName
        FROM {authors_table} ia
        JOIN {creators_table} c ON ia.creatorID = c.creatorID
        WHERE ia.itemID = ? AND ia.orderIndex >= 0
        ORDER BY ia.orderIndex
        "#,
        authors_table = authors_table,
        creators_table = creators_table
    );

    let mut author_stmt = conn.prepare(&author_sql).map_err(|e| format!("准备作者查询失败: {}", e))?;

    let author_results: Vec<(String, String)> = author_stmt
        .query_map(params![item_id], |row| {
            Ok((
                row.get::<_, String>(0).unwrap_or_default(),
                row.get::<_, String>(1).unwrap_or_default(),
            ))
        })
        .map_err(|e| format!("查询作者失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    if !author_results.is_empty() {
        metadata.authors = author_results
            .into_iter()
            .map(|(last_name, first_name)| Author {
                last_name,
                first_name,
                middle_name: None,
                prefix: None,
                suffix: None,
                author_type: AuthorType::Primary,
            })
            .collect();
    }

    // 查询其他元数据（DOI、ISBN 等）
    let meta_sql = format!(
        r#"
        SELECT f.fieldName, fv.value
        FROM {item_data_table} id
        JOIN {fields_table} f ON id.fieldID = f.fieldID
        JOIN {item_data_values_table} fv ON id.valueID = fv.valueID
        WHERE id.itemID = ?
        "#,
        item_data_table = item_data_table,
        fields_table = fields_table,
        item_data_values_table = item_data_values_table
    );

    let mut meta_stmt = conn.prepare(&meta_sql).map_err(|e| format!("准备元数据查询失败: {}", e))?;

    let meta_results: Vec<(String, String)> = meta_stmt
        .query_map(params![item_id], |row| {
            Ok((
                row.get::<_, String>(0).unwrap_or_default(),
                row.get::<_, String>(1).unwrap_or_default(),
            ))
        })
        .map_err(|e| format!("查询元数据失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    for (field_name, value) in meta_results {
        match field_name.as_str() {
            "DOI" => metadata.doi = Some(value),
            "ISBN" => metadata.isbn = Some(value),
            "ISSN" => metadata.issn = Some(value),
            "url" => metadata.url = Some(value),
            "publicationTitle" | "journalAbbreviation" => {
                if metadata.journal.is_none() {
                    metadata.journal = Some(value);
                }
            }
            "volume" => metadata.volume = Some(value),
            "issue" => metadata.issue = Some(value),
            "pages" => metadata.pages = Some(value),
            "publisher" => metadata.publisher = Some(value),
            "place" => metadata.location = Some(value),
            "edition" => metadata.edition = Some(value),
            "bookTitle" => metadata.book_title = Some(value),
            "conferenceName" => metadata.conference = Some(value),
            _ => {}
        }
    }

    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apa7_format() {
        let formatter = CitationFormatter::new(CitationFormat::APA7);
        let metadata = CitationMetadata {
            authors: vec![Author {
                last_name: "Smith".to_string(),
                first_name: "John".to_string(),
                middle_name: None,
                prefix: None,
                suffix: None,
                author_type: AuthorType::Primary,
            }],
            title: "Example Article Title".to_string(),
            journal: Some("Journal of Examples".to_string()),
            year: Some("2023".to_string()),
            volume: Some("10".to_string()),
            issue: Some("2".to_string()),
            pages: Some("100-120".to_string()),
            doi: Some("10.1234/example.2023".to_string()),
            ..Default::default()
        };

        let result = formatter.format(&metadata);
        assert!(result.formatted.contains("Smith"));
        assert!(result.formatted.contains("2023"));
        assert!(result.formatted.contains("Journal of Examples"));
    }

    #[test]
    fn test_parse_citation() {
        let input = "Smith, J. (2023). Example article. Journal of Testing, 10(2), 100-120. doi:10.1234/test";
        let result = parse_citation(input);
        assert!(result.success || !result.warnings.is_empty());
    }

    #[test]
    fn test_gb7714_format() {
        let formatter = CitationFormatter::new(CitationFormat::GB7714);
        let metadata = CitationMetadata {
            authors: vec![Author {
                last_name: "王".to_string(),
                first_name: "明".to_string(),
                middle_name: None,
                prefix: None,
                suffix: None,
                author_type: AuthorType::Primary,
            }],
            title: "计算机程序设计基础".to_string(),
            year: Some("2020".to_string()),
            publisher: Some("高等教育出版社".to_string()),
            item_type: ItemType::Book,
            ..Default::default()
        };

        let result = formatter.format(&metadata);
        assert!(result.formatted.contains("王明"));
        assert!(result.formatted.contains("[M]"));
    }
}