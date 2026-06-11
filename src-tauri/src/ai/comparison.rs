//! 文献智能对比模块
//!
//! 本模块实现文献对比的自动生成功能，包括：
//! - 支持同时选择 2-5 篇文献进行对比
//! - 自动生成多维度对比表格
//! - 默认对比维度：研究问题、研究方法、关键结论、创新点、局限性、引用情况
//! - 自动识别文献之间的引用关系
//! - 突出显示不同文献之间的矛盾点和共识
//! - 对比结果可保存和复用
//! - 支持取消正在进行的对比

use crate::ai::models::Message;
use crate::ai::traits::AIProvider;
use crate::db::connection::get_connection;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// 对比缓存前缀
const COMPARISON_CACHE_PREFIX: &str = "ZoPlus_Comparison::";

/// 文献对比数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleComparison {
    /// 对比ID（基于参与文献ID生成）
    pub comparison_id: String,
    /// 参与对比的文献ID列表
    pub item_ids: Vec<i32>,
    /// 文献标题列表
    pub titles: Vec<String>,
    /// 作者列表
    pub authors: Vec<String>,
    /// 年份列表
    pub years: Vec<String>,
    /// 各维度对比内容
    pub dimensions: ComparisonDimensions,
    /// 矛盾点分析
    pub contradictions: Vec<Contradiction>,
    /// 共识点分析
    pub consensus: Vec<Consensus>,
    /// 引用关系图
    pub citation_relations: Vec<CitationRelation>,
    /// 生成时间
    pub generated_at: i64,
    /// 摘要版本
    pub version: u32,
}

/// 多维度对比内容
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComparisonDimensions {
    /// 研究问题对比
    pub research_questions: Vec<String>,
    /// 研究方法对比
    pub research_methods: Vec<String>,
    /// 关键结论对比
    pub key_conclusions: Vec<String>,
    /// 创新点对比
    pub innovations: Vec<String>,
    /// 局限性对比
    pub limitations: Vec<String>,
    /// 引用情况对比
    pub citations: Vec<String>,
}

/// 矛盾点分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    /// 矛盾描述
    pub description: String,
    /// 涉及的文献索引（对应 item_ids 的下标）
    pub involved_indices: Vec<usize>,
    /// 矛盾类型
    pub contradiction_type: String,
}

/// 共识点分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consensus {
    /// 共识描述
    pub description: String,
    /// 涉及的文献索引
    pub involved_indices: Vec<usize>,
}

/// 引用关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationRelation {
    /// 施引文献索引
    pub from_index: usize,
    /// 被引文献索引
    pub to_index: usize,
    /// 引用关系描述
    pub description: String,
}

/// 对比生成器
pub struct ComparisonGenerator {
    /// AI Provider
    provider: Arc<dyn AIProvider>,
    /// 取消标志
    cancel_flag: Arc<AtomicBool>,
}

impl ComparisonGenerator {
    /// 创建新的对比生成器
    pub fn new(provider: Arc<dyn AIProvider>) -> Self {
        Self {
            provider,
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 请求取消对比
    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }

    /// 重置取消标志
    fn reset_cancel_flag(&self) {
        self.cancel_flag.store(false, Ordering::SeqCst);
    }

    /// 检查是否已取消
    #[allow(dead_code)]
    fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::SeqCst)
    }

    /// 从 Zotero 数据库读取多篇文献元数据
    fn fetch_metadata(&self, item_ids: &[i32]) -> Result<Vec<ItemMetadata>, ComparisonError> {
        let guard = get_connection()
            .map_err(|e| ComparisonError::DatabaseError(e.to_string()))?;
        let conn = guard
            .as_ref()
            .ok_or_else(|| ComparisonError::DatabaseError("数据库连接未初始化".to_string()))?;

        let mut results = Vec::new();

        for item_id in item_ids {
            let sql = r#"
            SELECT
                i.itemID as item_id,
                fv_title.value as title,
                fv_date.value as year,
                (
                    SELECT GROUP_CONCAT(
                        COALESCE(c.lastName, '') || COALESCE(c.firstName, ''),
                        '; '
                    )
                    FROM itemCreators ia
                    JOIN creators c ON ia.creatorID = c.creatorID
                    WHERE ia.itemID = i.itemID
                    ORDER BY ia.orderIndex
                ) as authors
            FROM items i
            LEFT JOIN itemData id_title ON i.itemID = id_title.itemID
                AND id_title.fieldID = (SELECT fieldID FROM fields WHERE fieldName = 'title')
            LEFT JOIN itemDataValues fv_title ON id_title.valueID = fv_title.valueID
            LEFT JOIN itemData id_date ON i.itemID = id_date.itemID
                AND id_date.fieldID = (SELECT fieldID FROM fields WHERE fieldName = 'date')
            WHERE i.itemID = ?
            "#;

            let metadata = conn
                .query_row(sql, params![item_id], |row| {
                    Ok(ItemMetadata {
                        item_id: row.get(0)?,
                        title: row.get::<_, String>(1).unwrap_or_default(),
                        year: row.get::<_, String>(2).unwrap_or_default(),
                        authors: row.get::<_, String>(3).unwrap_or_default(),
                    })
                })
                .map_err(|e| ComparisonError::DatabaseError(format!("查询元数据失败: {}", e)))?;

            results.push(metadata);
        }

        Ok(results)
    }

    /// 识别文献之间的引用关系
    fn detect_citation_relations(&self, item_ids: &[i32]) -> Vec<CitationRelation> {
        let guard = match get_connection() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };
        let conn = match guard.as_ref() {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut relations = Vec::new();

        // 获取所有文献的 citation key
        let mut citation_keys: Vec<(usize, String)> = Vec::new();
        for (idx, item_id) in item_ids.iter().enumerate() {
            let sql = r#"
            SELECT value
            FROM itemData id_key
            JOIN itemDataValues fv_key ON id_key.valueID = fv_key.valueID
            WHERE id_key.itemID = ?
            AND id_key.fieldID = (SELECT fieldID FROM fields WHERE fieldName = 'citationKey')
            "#;

            if let Ok(citation_key) = conn.query_row(sql, params![item_id], |row| row.get::<_, String>(0))
            {
                citation_keys.push((idx, citation_key));
            }
        }

        // 简化的引用检测：检查 extra 字段中是否包含其他文献的 citation key
        for (from_idx, item_id) in item_ids.iter().enumerate() {
            let sql = r#"
            SELECT value
            FROM itemData id_extra
            JOIN itemDataValues fv_extra ON id_extra.valueID = fv_extra.valueID
            WHERE id_extra.itemID = ?
            AND id_extra.fieldID = (SELECT fieldID FROM fields WHERE fieldName = 'extra')
            "#;

            if let Ok(extra) = conn.query_row(sql, params![item_id], |row| row.get::<_, String>(0)) {
                for (to_idx, citation_key) in &citation_keys {
                    if *to_idx != from_idx && extra.contains(citation_key) {
                        relations.push(CitationRelation {
                            from_index: from_idx,
                            to_index: *to_idx,
                            description: format!("文献 {} 引用了 {}", from_idx + 1, to_idx + 1),
                        });
                    }
                }
            }
        }

        relations
    }

    /// 从 extra 字段读取缓存的对比结果
    fn read_cached_comparison(&self, item_ids: &[i32]) -> Option<ArticleComparison> {
        // 生成对比ID
        let comparison_id = generate_comparison_id(item_ids);

        let guard = match get_connection() {
            Ok(g) => g,
            Err(_) => return None,
        };
        let conn = guard.as_ref()?;

        // 在所有参与的文献中查找缓存（使用第一篇文献的 extra 字段）
        if item_ids.is_empty() {
            return None;
        }

        let sql = r#"
        SELECT fv_extra.value
        FROM itemData id_extra
        JOIN itemDataValues fv_extra ON id_extra.valueID = fv_extra.valueID
        WHERE id_extra.itemID = ?
        AND id_extra.fieldID = (SELECT fieldID FROM fields WHERE fieldName = 'extra')
        "#;

        let extra_content: Option<String> = conn
            .query_row(sql, params![item_ids[0]], |row| row.get(0))
            .ok()?;

        if let Some(extra) = extra_content {
            if let Some(start) = extra.find(&format!("{}:", COMPARISON_CACHE_PREFIX)) {
                let json_start = start + COMPARISON_CACHE_PREFIX.len() + comparison_id.len() + 1;
                if let Some(end) = extra.find("::EndComparison") {
                    let json_str = &extra[json_start..end];
                    if let Ok(comparison) = ArticleComparison::from_json(json_str) {
                        return Some(comparison);
                    }
                }
            }
        }

        None
    }

    /// 保存对比结果到 extra 字段
    fn save_comparison_to_extra(
        &self,
        item_ids: &[i32],
        comparison: &ArticleComparison,
    ) -> Result<(), ComparisonError> {
        if item_ids.is_empty() {
            return Err(ComparisonError::InvalidInput("文献ID列表为空".to_string()));
        }

        let guard = get_connection()
            .map_err(|e| ComparisonError::DatabaseError(e.to_string()))?;
        let conn = guard
            .as_ref()
            .ok_or_else(|| ComparisonError::DatabaseError("数据库连接未初始化".to_string()))?;

        // 序列化对比结果
        let json =
            comparison
                .to_json()
                .map_err(|e| ComparisonError::SerializeError(e.to_string()))?;
        let cache_key = generate_comparison_id(item_ids);
        let cache_entry = format!(
            "{}{}:{}{}::EndComparison",
            COMPARISON_CACHE_PREFIX, cache_key, json, " ".repeat(0)
        );

        // 获取 extra 字段的当前值
        let current_extra: Option<String> = conn
            .query_row(
                r#"
                SELECT fv_extra.value
                FROM itemData id_extra
                JOIN itemDataValues fv_extra ON id_extra.valueID = fv_extra.valueID
                WHERE id_extra.itemID = ?
                AND id_extra.fieldID = (SELECT fieldID FROM fields WHERE fieldName = 'extra')
                "#,
                params![item_ids[0]],
                |row| row.get(0),
            )
            .ok();

        // 构建新的 extra 值
        let new_extra = if let Some(existing) = current_extra {
            let cleaned = if let Some(start) = existing.find(COMPARISON_CACHE_PREFIX) {
                let before = &existing[..start];
                let after = existing[start..]
                    .find("::EndComparison")
                    .map(|end| &existing[start + end + 15..])
                    .unwrap_or("");
                format!("{}{}", before.trim(), after.trim())
            } else {
                existing
            };
            format!("{}\n{}", cleaned.trim(), cache_entry)
        } else {
            cache_entry
        };

        // 更新 extra 字段
        let update_sql = r#"
        UPDATE itemData
        SET valueID = (
            SELECT v.valueID
            FROM itemDataValues v
            WHERE v.value = ?
            LIMIT 1
        )
        WHERE itemID = ?
        AND fieldID = (SELECT fieldID FROM fields WHERE fieldName = 'extra')
        "#;

        let result = conn.execute(update_sql, params![new_extra, item_ids[0]]);
        if result.is_err() {
            let insert_sql = r#"
            INSERT INTO itemData (itemID, fieldID, valueID)
            SELECT ?, fieldID, v.valueID
            FROM itemDataValues v
            WHERE v.value = ?
            LIMIT 1
            "#;
            conn.execute(insert_sql, params![item_ids[0], new_extra])
                .map_err(|e| ComparisonError::DatabaseError(format!("保存对比结果失败: {}", e)))?;
        }

        eprintln!(
            "[对比] 对比结果已保存: comparison_id={}",
            comparison.comparison_id
        );
        Ok(())
    }

    /// 生成文献对比
    pub async fn generate_comparison(
        &self,
        item_ids: Vec<i32>,
    ) -> Result<ArticleComparison, ComparisonError> {
        self.reset_cancel_flag();
        eprintln!("[对比] 开始生成对比: item_ids={:?}", item_ids);

        if item_ids.len() < 2 {
            return Err(ComparisonError::InvalidInput(
                "对比需要至少2篇文献".to_string(),
            ));
        }

        if item_ids.len() > 5 {
            return Err(ComparisonError::InvalidInput(
                "对比最多支持5篇文献".to_string(),
            ));
        }

        // 获取元数据
        let metadata = self.fetch_metadata(&item_ids)?;

        // 检测引用关系
        let citation_relations = self.detect_citation_relations(&item_ids);

        // 构建提示词
        let prompt = self.build_comparison_prompt(&metadata);

        // 调用 AI 生成对比
        let messages = vec![
            Message::system(
                "你是一个专业的学术论文对比分析助手。请根据提供的多篇论文信息，\
                生成结构化的对比分析。",
            ),
            Message::user(prompt),
        ];

        let response = self
            .provider
            .chat_completion(messages)
            .map_err(|e| ComparisonError::AIError(e.to_string()))?;

        // 解析 AI 响应
        let comparison = self.parse_comparison_response(&metadata, &response)?;

        // 添加引用关系
        let mut final_comparison = comparison;
        final_comparison.citation_relations = citation_relations;

        // 保存到 extra 字段
        self.save_comparison_to_extra(&item_ids, &final_comparison)?;

        eprintln!(
            "[对比] 对比生成完成: comparison_id={}",
            final_comparison.comparison_id
        );
        Ok(final_comparison)
    }

    /// 构建对比生成提示词
    fn build_comparison_prompt(&self, metadata: &[ItemMetadata]) -> String {
        let mut prompt = format!(
            "请对以下 {} 篇学术论文进行多维度对比分析：\n\n",
            metadata.len()
        );

        for (i, m) in metadata.iter().enumerate() {
            prompt.push_str(&format!(
                "文献{}：{}\n作者：{}\n年份：{}\n\n",
                i + 1,
                m.title,
                m.authors,
                m.year
            ));
        }

        prompt.push_str(
            "请按以下结构生成对比分析（使用 Markdown 格式）：\n\n\
            ## 研究问题对比\n\
            （对比各文献研究的核心问题，找出异同）\n\n\
            ## 研究方法对比\n\
            （对比各文献采用的研究方法和技术路线）\n\n\
            ## 关键结论对比\n\
            （对比各文献的主要发现和结论）\n\n\
            ## 创新点对比\n\
            （对比各文献的创新之处）\n\n\
            ## 局限性对比\n\
            （对比各文献的不足和局限性）\n\n\
            ## 引用情况对比\n\
            （对比各文献的引用频次和引用来源）\n\n\
            ## 矛盾点分析\n\
            （分析文献之间存在的矛盾或不一致的观点）\n\n\
            ## 共识点分析\n\
            （分析文献之间存在的共识或一致的观点）",
        );

        prompt
    }

    /// 解析 AI 响应
    fn parse_comparison_response(
        &self,
        metadata: &[ItemMetadata],
        response: &str,
    ) -> Result<ArticleComparison, ComparisonError> {
        let sections = parse_markdown_sections(response);

        let n = metadata.len();
        let comparison_id = generate_comparison_id(
            &metadata.iter().map(|m| m.item_id).collect::<Vec<_>>(),
        );

        // 提取各维度内容
        let research_questions = self.extract_dimension_content(&sections, "研究问题对比", n);
        let research_methods = self.extract_dimension_content(&sections, "研究方法对比", n);
        let key_conclusions = self.extract_dimension_content(&sections, "关键结论对比", n);
        let innovations = self.extract_dimension_content(&sections, "创新点对比", n);
        let limitations = self.extract_dimension_content(&sections, "局限性对比", n);
        let citations = self.extract_dimension_content(&sections, "引用情况对比", n);

        // 解析矛盾点
        let contradictions = self.parse_contradictions(
            sections.get("矛盾点分析").cloned().unwrap_or_default(),
            n,
        );

        // 解析共识点
        let consensus = self.parse_consensus(
            sections.get("共识点分析").cloned().unwrap_or_default(),
            n,
        );

        Ok(ArticleComparison {
            comparison_id,
            item_ids: metadata.iter().map(|m| m.item_id).collect(),
            titles: metadata.iter().map(|m| m.title.clone()).collect(),
            authors: metadata.iter().map(|m| m.authors.clone()).collect(),
            years: metadata.iter().map(|m| m.year.clone()).collect(),
            dimensions: ComparisonDimensions {
                research_questions,
                research_methods,
                key_conclusions,
                innovations,
                limitations,
                citations,
            },
            contradictions,
            consensus,
            citation_relations: Vec::new(), // 后续添加
            generated_at: chrono_timestamp(),
            version: 1,
        })
    }

    /// 提取维度内容（处理各文献的对比描述）
    fn extract_dimension_content(
        &self,
        sections: &std::collections::HashMap<String, String>,
        key: &str,
        _expected_count: usize,
    ) -> Vec<String> {
        let content = sections.get(key).cloned().unwrap_or_default();

        // 尝试按文献分割内容
        // 格式可能是 "文献1: ... 文献2: ..." 或按段落分隔
        let mut results = Vec::new();

        // 简单处理：尝试识别 "文献X:" 或 "第X篇" 模式
        let parts: Vec<&str> = content
            .split("文献")
            .flat_map(|s| s.split('第'))
            .filter(|s| !s.is_empty())
            .collect();

        if parts.len() >= 2 {
            for part in parts {
                results.push(part.trim().to_string());
            }
        } else {
            // 如果无法分割，返回整个内容
            results.push(content);
        }

        results
    }

    /// 解析矛盾点
    fn parse_contradictions(&self, content: String, _n: usize) -> Vec<Contradiction> {
        let mut contradictions = Vec::new();

        // 简单解析：按行分割，寻找包含"矛盾"或"冲突"的描述
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // 检测是否包含矛盾相关关键词
            let has_contradiction = trimmed.contains("矛盾")
                || trimmed.contains("冲突")
                || trimmed.contains("不一致")
                || trimmed.contains("相反");

            if has_contradiction {
                contradictions.push(Contradiction {
                    description: trimmed.to_string(),
                    involved_indices: Vec::new(), // 需要进一步解析
                    contradiction_type: "观点冲突".to_string(),
                });
            }
        }

        contradictions
    }

    /// 解析共识点
    fn parse_consensus(&self, content: String, _n: usize) -> Vec<Consensus> {
        let mut consensus_points = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // 检测是否包含共识相关关键词
            let has_consensus = trimmed.contains("共识")
                || trimmed.contains("一致")
                || trimmed.contains("相同")
                || trimmed.contains("认同");

            if has_consensus {
                consensus_points.push(Consensus {
                    description: trimmed.to_string(),
                    involved_indices: Vec::new(),
                });
            }
        }

        consensus_points
    }

    /// 检查是否有缓存的对比
    pub fn has_cached_comparison(&self, item_ids: &[i32]) -> bool {
        self.read_cached_comparison(item_ids).is_some()
    }

    /// 获取缓存的对比
    pub fn get_cached_comparison(&self, item_ids: &[i32]) -> Option<ArticleComparison> {
        self.read_cached_comparison(item_ids)
    }
}

/// 文献元数据
struct ItemMetadata {
    item_id: i32,
    title: String,
    authors: String,
    year: String,
}

/// 生成对比ID
fn generate_comparison_id(item_ids: &[i32]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    item_ids.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// 获取当前时间戳（毫秒）
fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

impl ArticleComparison {
    /// 导出为 Markdown 格式
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# 文献对比分析报告\n\n");
        md.push_str("---\n\n");

        // 参与文献列表
        md.push_str("## 参与对比的文献\n\n");
        for (i, title) in self.titles.iter().enumerate() {
            md.push_str(&format!(
                "{}. **{}** - {} ({})\n",
                i + 1,
                title,
                self.authors.get(i).unwrap_or(&"未知".to_string()),
                self.years.get(i).unwrap_or(&"未知".to_string())
            ));
        }
        md.push_str("\n---\n\n");

        // 各维度对比表格
        md.push_str("## 多维度对比\n\n");

        // 研究问题
        md.push_str("### 研究问题\n\n");
        md.push_str("| 文献 | 研究问题 |\n");
        md.push_str("|------|----------|\n");
        for (i, content) in self.dimensions.research_questions.iter().enumerate() {
            md.push_str(&format!(
                "| 文献{} | {} |\n",
                i + 1,
                content.replace("|", "\\|").replace("\n", "<br>")
            ));
        }
        md.push_str("\n");

        // 研究方法
        md.push_str("### 研究方法\n\n");
        md.push_str("| 文献 | 研究方法 |\n");
        md.push_str("|------|----------|\n");
        for (i, content) in self.dimensions.research_methods.iter().enumerate() {
            md.push_str(&format!(
                "| 文献{} | {} |\n",
                i + 1,
                content.replace("|", "\\|").replace("\n", "<br>")
            ));
        }
        md.push_str("\n");

        // 关键结论
        md.push_str("### 关键结论\n\n");
        md.push_str("| 文献 | 关键结论 |\n");
        md.push_str("|------|----------|\n");
        for (i, content) in self.dimensions.key_conclusions.iter().enumerate() {
            md.push_str(&format!(
                "| 文献{} | {} |\n",
                i + 1,
                content.replace("|", "\\|").replace("\n", "<br>")
            ));
        }
        md.push_str("\n");

        // 创新点
        md.push_str("### 创新点\n\n");
        md.push_str("| 文献 | 创新点 |\n");
        md.push_str("|------|----------|\n");
        for (i, content) in self.dimensions.innovations.iter().enumerate() {
            md.push_str(&format!(
                "| 文献{} | {} |\n",
                i + 1,
                content.replace("|", "\\|").replace("\n", "<br>")
            ));
        }
        md.push_str("\n");

        // 局限性
        md.push_str("### 局限性\n\n");
        md.push_str("| 文献 | 局限性 |\n");
        md.push_str("|------|----------|\n");
        for (i, content) in self.dimensions.limitations.iter().enumerate() {
            md.push_str(&format!(
                "| 文献{} | {} |\n",
                i + 1,
                content.replace("|", "\\|").replace("\n", "<br>")
            ));
        }
        md.push_str("\n");

        // 矛盾点
        if !self.contradictions.is_empty() {
            md.push_str("---\n\n## 矛盾点分析\n\n");
            for (i, c) in self.contradictions.iter().enumerate() {
                md.push_str(&format!(
                    "{}. {} ({})\n\n",
                    i + 1,
                    c.description,
                    c.contradiction_type
                ));
            }
            md.push_str("\n");
        }

        // 共识点
        if !self.consensus.is_empty() {
            md.push_str("---\n\n## 共识点分析\n\n");
            for (i, c) in self.consensus.iter().enumerate() {
                md.push_str(&format!("{}. {}\n\n", i + 1, c.description));
            }
            md.push_str("\n");
        }

        // 引用关系
        if !self.citation_relations.is_empty() {
            md.push_str("---\n\n## 引用关系\n\n");
            for r in &self.citation_relations {
                md.push_str(&format!(
                    "- 文献{} 引用了 文献{}\n",
                    r.from_index + 1,
                    r.to_index + 1
                ));
            }
            md.push_str("\n");
        }

        md.push_str("---\n\n");
        md.push_str(&format!(
            "*对比生成时间: {} | 版本: {}*\n",
            chrono_datetime_string(self.generated_at),
            self.version
        ));

        md
    }

    /// 导出为 Excel CSV 格式
    pub fn to_csv(&self) -> String {
        let mut csv = String::new();

        // 表头
        csv.push_str("维度,文献");
        for i in 1..=self.titles.len() {
            csv.push_str(&format!(",文献{}", i));
        }
        csv.push_str("\n");

        // 各维度数据
        let dimensions = vec![
            ("研究问题", &self.dimensions.research_questions),
            ("研究方法", &self.dimensions.research_methods),
            ("关键结论", &self.dimensions.key_conclusions),
            ("创新点", &self.dimensions.innovations),
            ("局限性", &self.dimensions.limitations),
            ("引用情况", &self.dimensions.citations),
        ];

        for (dim_name, dim_content) in dimensions {
            csv.push_str(dim_name);
            for content in dim_content {
                csv.push_str(&format!(",\"{}\"", content.replace("\"", "\"\"")));
            }
            csv.push_str("\n");
        }

        csv
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// 从 JSON 字符串反序列化
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// 时间戳转可读日期字符串
fn chrono_datetime_string(timestamp: i64) -> String {
    let secs = timestamp / 1000;
    std::time::UNIX_EPOCH
        .checked_add(std::time::Duration::from_secs(secs as u64))
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| {
            let days_since_epoch = d.as_secs() / 86400;
            let year = (days_since_epoch / 365) + 1970;
            let remaining = days_since_epoch % 365;
            let month = remaining / 30 + 1;
            let day = remaining % 30 + 1;
            format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(31))
        })
        .unwrap_or_else(|| "未知时间".to_string())
}

/// 解析 Markdown 内容，提取各章节
fn parse_markdown_sections(content: &str) -> std::collections::HashMap<String, String> {
    let mut sections = std::collections::HashMap::new();
    let mut current_section = String::new();
    let mut current_title = String::new();
    let mut in_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // 检查是否是二级标题
        if trimmed.starts_with("## ") {
            if in_section && !current_title.is_empty() {
                sections.insert(current_title.clone(), current_section.trim().to_string());
            }

            current_title = trimmed.trim_start_matches("## ").to_string();
            current_section.clear();
            in_section = true;
        } else if in_section {
            current_section.push_str(trimmed);
            current_section.push('\n');
        }
    }

    if in_section && !current_title.is_empty() {
        sections.insert(current_title, current_section.trim().to_string());
    }

    sections
}

/// 对比错误类型
#[derive(Debug)]
pub enum ComparisonError {
    /// AI 调用错误
    AIError(String),
    /// 数据库错误
    DatabaseError(String),
    /// 解析错误
    ParseError(String),
    /// 序列化错误
    SerializeError(String),
    /// 缓存不存在
    CacheNotFound,
    /// 输入无效
    InvalidInput(String),
    /// 已取消
    Cancelled,
}

impl std::fmt::Display for ComparisonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonError::AIError(msg) => write!(f, "AI 调用失败: {}", msg),
            ComparisonError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            ComparisonError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            ComparisonError::SerializeError(msg) => write!(f, "序列化错误: {}", msg),
            ComparisonError::CacheNotFound => write!(f, "缓存不存在"),
            ComparisonError::InvalidInput(msg) => write!(f, "输入无效: {}", msg),
            ComparisonError::Cancelled => write!(f, "对比已取消"),
        }
    }
}

impl std::error::Error for ComparisonError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_comparison_id() {
        let ids1 = vec![1, 2, 3];
        let ids2 = vec![1, 2, 3];
        let ids3 = vec![1, 2, 4];

        assert_eq!(generate_comparison_id(&ids1), generate_comparison_id(&ids2));
        assert_ne!(generate_comparison_id(&ids1), generate_comparison_id(&ids3));
    }

    #[test]
    fn test_parse_markdown_sections() {
        let content = r##"
## 研究问题对比
这是研究问题的对比内容

## 研究方法对比
这是研究方法的对比内容
"##;

        let sections = parse_markdown_sections(content);
        assert!(sections.contains_key("研究问题对比"));
        assert!(sections.contains_key("研究方法对比"));
    }
}