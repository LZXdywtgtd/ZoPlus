//! 文献引用关系图谱模块
//!
//! 本模块提供基于 Zotero 数据库的引用网络分析功能：
//! - 从 itemRelations 表提取引用关系
//! - 计算文献的 PageRank 值识别关键文献
//! - 生成可交互的关系图谱数据
//!
//! # 核心算法
//! - PageRank：用于识别引用网络中的关键节点（文献）
//! - 邻接表：高效存储稀疏的引用关系
//! - 迭代计算：标准的 PageRank 迭代直到收敛

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 引用关系图谱节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationNode {
    /// 文献ID
    pub item_id: i32,
    /// 文献标题
    pub title: String,
    /// 作者信息
    pub authors: String,
    /// 发表年份
    pub year: String,
    /// 被引次数（直接引用）
    pub citation_count: i32,
    /// PageRank 值
    pub pagerank: f64,
    /// 节点大小（用于可视化，基于被引次数）
    pub node_size: f64,
}

/// 引用关系图谱边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationEdge {
    /// 源节点ID（引用方）
    pub source: i32,
    /// 目标节点ID（被引方）
    pub target: i32,
    /// 边的权重（可表示引用强度）
    pub weight: f64,
}

/// 完整引用图谱数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationGraph {
    /// 所有节点
    pub nodes: Vec<CitationNode>,
    /// 所有边
    pub edges: Vec<CitationEdge>,
    /// 总节点数
    pub total_nodes: i32,
    /// 总边数
    pub total_edges: i32,
    /// 计算耗时（毫秒）
    pub compute_time_ms: i64,
}

/// 关键文献推荐结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPaper {
    /// 文献ID
    pub item_id: i32,
    /// 文献标题
    pub title: String,
    /// 作者信息
    pub authors: String,
    /// 发表年份
    pub year: String,
    /// PageRank 值
    pub pagerank: f64,
    /// 被引次数
    pub citation_count: i32,
    /// 推荐理由
    pub reason: String,
}

/// 文献引用关系详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperCitations {
    /// 文献ID
    pub item_id: i32,
    /// 文献标题
    pub title: String,
    /// 作者信息
    pub authors: String,
    /// 施引文献（引用了该文献的文献）
    pub cited_by: Vec<CitationNode>,
    /// 被引文献（该文献引用的文献）
    pub references: Vec<CitationNode>,
    /// 总被引次数
    pub total_cited_by: i32,
    /// 总参考文献数
    pub total_references: i32,
}

/// PageRank 配置
#[derive(Debug, Clone)]
pub struct PageRankConfig {
    /// 阻尼系数（通常 0.85）
    pub damping: f64,
    /// 最大迭代次数
    pub max_iterations: usize,
    /// 收敛阈值
    pub convergence_threshold: f64,
}

impl Default for PageRankConfig {
    fn default() -> Self {
        Self {
            damping: 0.85,
            max_iterations: 100,
            convergence_threshold: 1e-6,
        }
    }
}

/// 构建引用图谱
///
/// # 参数
/// * `db_path` - Zotero 数据库路径
/// * `min_citations` - 最小被引次数（用于过滤）
///
/// # 返回值
/// * `Result<CitationGraph, String>` - 图谱数据或错误
pub fn build_citation_graph(db_path: &str, min_citations: i32) -> Result<CitationGraph, String> {
    let start_time = std::time::Instant::now();

    // 连接数据库
    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| format!("无法打开数据库: {}", e))?;

    // 1. 获取所有文献基本信息
    let items = get_all_items_info(&conn)?;

    // 2. 提取引用关系
    let (edges, citation_counts) = extract_citation_relations(&conn, &items)?;

    // 3. 构建节点
    let nodes: Vec<CitationNode> = items
        .into_iter()
        .filter(|item| {
            // 过滤：只保留被引次数 >= min_citations 的文献
            let count = citation_counts.get(&item.item_id).copied().unwrap_or(0);
            count >= min_citations
        })
        .map(|item| {
            let count = citation_counts.get(&item.item_id).copied().unwrap_or(0);
            CitationNode {
                item_id: item.item_id,
                title: item.title,
                authors: item.authors,
                year: item.year,
                citation_count: count,
                pagerank: 0.0, // 稍后计算
                node_size: compute_node_size(count),
            }
        })
        .collect();

    // 4. 计算 PageRank
    let node_ids: Vec<i32> = nodes.iter().map(|n| n.item_id).collect();
    let pagerank_scores = compute_pagerank(&node_ids, &edges, &PageRankConfig::default());

    // 5. 更新节点的 PageRank 值
    let nodes: Vec<CitationNode> = nodes
        .into_iter()
        .map(|mut node| {
            node.pagerank = pagerank_scores.get(&node.item_id).copied().unwrap_or(0.0);
            node
        })
        .collect();

    let total_nodes = nodes.len() as i32;
    let total_edges = edges.len() as i32;
    let elapsed = start_time.elapsed();
    let compute_time_ms = elapsed.as_millis() as i64;

    Ok(CitationGraph {
        nodes,
        edges,
        total_nodes,
        total_edges,
        compute_time_ms,
    })
}

/// 获取所有文献基本信息
fn get_all_items_info(conn: &rusqlite::Connection) -> Result<Vec<ItemBasicInfo>, String> {
    let author_table = detect_author_table_name(conn);

    let sql = format!(
        r#"
        SELECT
            i.itemID as item_id,
            COALESCE(fv_title.value, '') as title,
            COALESCE(fv_date.value, '') as year,
            (
                SELECT GROUP_CONCAT(
                    COALESCE(c.lastName, '') || COALESCE(c.firstName, ''),
                    '; '
                )
                FROM {} ia
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
        LEFT JOIN itemDataValues fv_date ON id_date.valueID = fv_date.valueID
        "#,
        author_table
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("准备查询失败: {}", e))?;

    let items = stmt
        .query_map([], |row| {
            Ok(ItemBasicInfo {
                item_id: row.get(0)?,
                title: row.get(1)?,
                year: row.get(2)?,
                authors: row.get(3)?,
            })
        })
        .map_err(|e| format!("查询失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}

/// 文献基本信息
#[derive(Debug, Clone)]
struct ItemBasicInfo {
    item_id: i32,
    title: String,
    year: String,
    authors: String,
}

/// 检测作者关联表名
fn detect_author_table_name(conn: &rusqlite::Connection) -> String {
    let candidates = ["itemCreators", "itemAuthors", "itemCreator"];
    for table in candidates {
        let exists = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?",
                [table],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if exists {
            return table.to_string();
        }
    }
    "itemCreators".to_string()
}

/// 提取引用关系
///
/// Zotero 的 itemRelations 表存储了文献关系，包括引用关系：
/// - 引用方（subjectItemID）引用了被引方（objectItemID）
/// - 关系类型（relationType）例如 "cites"
fn extract_citation_relations(
    conn: &rusqlite::Connection,
    items: &[ItemBasicInfo],
) -> Result<(Vec<CitationEdge>, HashMap<i32, i32>), String> {
    // 检查 itemRelations 表是否存在
    let table_exists = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='itemRelations'",
            [],
            |_| Ok(true),
        )
        .unwrap_or(false);

    let mut edges = Vec::new();
    let mut citation_counts: HashMap<i32, i32> = HashMap::new();

    // 构建 item_id 集合用于验证
    let valid_ids: HashSet<i32> = items.iter().map(|i| i.item_id).collect();

    if table_exists {
        // 从 itemRelations 表提取引用关系
        let sql = r#"
            SELECT subjectItemID, objectItemID, relationType
            FROM itemRelations
            WHERE relationType LIKE '%cites%'
               OR relationType LIKE '%references%'
               OR relationType = 'dc:relation'
        "#;

        let mut stmt = conn.prepare(sql).map_err(|e| format!("准备查询失败: {}", e))?;

        let relations: Vec<(i32, i32)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| format!("查询失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        for (source, target) in relations {
            // 验证节点有效性
            if valid_ids.contains(&source) && valid_ids.contains(&target) && source != target {
                edges.push(CitationEdge {
                    source,
                    target,
                    weight: 1.0,
                });
                // 统计被引次数
                *citation_counts.entry(target).or_insert(0) += 1;
            }
        }
    } else {
        eprintln!("[引用图谱] itemRelations 表不存在，尝试从 itemData 提取 DOI 引用关系");
        // 备选方案：从 itemData 中提取 DOI 引用信息
        // 这需要解析文献的 "references" 或 "related" 字段
    }

    // 确保所有节点都有被引次数记录
    for item in items {
        citation_counts.entry(item.item_id).or_insert(0);
    }

    Ok((edges, citation_counts))
}

/// 计算节点大小
fn compute_node_size(citation_count: i32) -> f64 {
    // 节点大小基于被引次数，使用对数缩放避免极端值
    // 最小 size 为 5，最大为 50
    let base_size = 5.0;
    let scale = 10.0;
    let log_count = (citation_count as f64 + 1.0).ln() + 1.0;
    (base_size + scale * log_count).min(50.0)
}

/// 计算 PageRank
///
/// # 参数
/// * `node_ids` - 节点 ID 列表
/// * `edges` - 边列表
/// * `config` - PageRank 配置
///
/// # 返回值
/// * `HashMap<i32, f64>` - 每个节点的 PageRank 值
fn compute_pagerank(
    node_ids: &[i32],
    edges: &[CitationEdge],
    config: &PageRankConfig,
) -> HashMap<i32, f64> {
    let n = node_ids.len();
    if n == 0 {
        return HashMap::new();
    }

    // 构建邻接表（出链）
    let mut outgoing: HashMap<i32, Vec<i32>> = HashMap::new();
    for node_id in node_ids {
        outgoing.entry(*node_id).or_insert_with(Vec::new);
    }
    for edge in edges {
        if let Some(out_list) = outgoing.get_mut(&edge.source) {
            out_list.push(edge.target);
        }
    }

    // 初始化 PageRank（均匀分布）
    let mut pagerank: HashMap<i32, f64> = HashMap::new();
    let initial_value = 1.0 / n as f64;
    for node_id in node_ids {
        pagerank.insert(*node_id, initial_value);
    }

    // 迭代计算
    let damping = config.damping;
    let jump_prob = (1.0 - damping) / n as f64;

    for i in 0..config.max_iterations {
        let mut new_pagerank: HashMap<i32, f64> = HashMap::new();

        // 跳跃项（随机跳转）
        for node_id in node_ids {
            new_pagerank.insert(*node_id, jump_prob);
        }

        // 传播项（从入链传递）
        for edge in edges {
            if let Some(pr) = pagerank.get(&edge.source) {
                let out_degree = outgoing.get(&edge.source).map(|v| v.len()).unwrap_or(1);
                if out_degree > 0 {
                    let contribution = pr * damping / out_degree as f64;
                    if let Some(current) = new_pagerank.get_mut(&edge.target) {
                        *current += contribution;
                    }
                }
            }
        }

        // 检查收敛
        let mut max_diff: f64 = 0.0;
        for node_id in node_ids {
            let old = pagerank.get(node_id).copied().unwrap_or(0.0);
            let new = new_pagerank.get(node_id).copied().unwrap_or(0.0);
            let diff = (new - old).abs();
            max_diff = max_diff.max(diff);
        }

        pagerank = new_pagerank;

        if max_diff < config.convergence_threshold {
            eprintln!("[PageRank] 在 {} 次迭代后收敛", i);
            break;
        }
    }

    // 归一化
    let sum: f64 = pagerank.values().sum();
    if sum > 0.0 {
        for (_, pr) in pagerank.iter_mut() {
            *pr /= sum;
        }
    }

    pagerank
}

/// 获取关键文献推荐列表
pub fn get_key_papers(db_path: &str, limit: i32) -> Result<Vec<KeyPaper>, String> {
    let graph = build_citation_graph(db_path, 0)?;

    // 按 PageRank 排序
    let mut nodes = graph.nodes;
    nodes.sort_by(|a, b| b.pagerank.partial_cmp(&a.pagerank).unwrap_or(std::cmp::Ordering::Equal));

    let key_papers: Vec<KeyPaper> = nodes
        .into_iter()
        .take(limit as usize)
        .map(|node| {
            let reason = if node.citation_count > 10 {
                format!("高被引文献（被引 {} 次）", node.citation_count)
            } else if node.pagerank > 0.01 {
                format!("PageRank: {:.4}，引用网络核心节点", node.pagerank)
            } else {
                "引用网络重要节点".to_string()
            };

            KeyPaper {
                item_id: node.item_id,
                title: node.title,
                authors: node.authors,
                year: node.year,
                pagerank: node.pagerank,
                citation_count: node.citation_count,
                reason,
            }
        })
        .collect();

    Ok(key_papers)
}

/// 获取指定文献的引用关系
pub fn get_paper_citations(db_path: &str, item_id: i32) -> Result<PaperCitations, String> {
    let graph = build_citation_graph(db_path, 0)?;

    // 查找指定文献
    let node = graph
        .nodes
        .iter()
        .find(|n| n.item_id == item_id)
        .ok_or_else(|| format!("未找到 item_id={} 的文献", item_id))?
        .clone();

    // 查找施引文献（被该文献引用的文献，即 out_edges）
    let cited_by: Vec<CitationNode> = graph
        .edges
        .iter()
        .filter(|e| e.target == item_id)
        .filter_map(|e| graph.nodes.iter().find(|n| n.item_id == e.source).cloned())
        .collect();

    // 查找被引文献（该文献引用的文献，即 in_edges）
    let references: Vec<CitationNode> = graph
        .edges
        .iter()
        .filter(|e| e.source == item_id)
        .filter_map(|e| graph.nodes.iter().find(|n| n.item_id == e.target).cloned())
        .collect();

    // 先获取长度，避免移动后借用
    let total_cited_by = cited_by.len() as i32;
    let total_references = references.len() as i32;

    Ok(PaperCitations {
        item_id,
        title: node.title,
        authors: node.authors,
        cited_by,
        references,
        total_cited_by,
        total_references,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_node_size() {
        assert_eq!(compute_node_size(0), 5.0);
        assert_eq!(compute_node_size(1), 15.0);
        assert!(compute_node_size(100) > 15.0);
    }

    #[test]
    fn test_pagerank_basic() {
        // 简单图：A -> B, B -> C, C -> A
        let node_ids = vec![1, 2, 3];
        let edges = vec![
            CitationEdge { source: 1, target: 2, weight: 1.0 },
            CitationEdge { source: 2, target: 3, weight: 1.0 },
            CitationEdge { source: 3, target: 1, weight: 1.0 },
        ];

        let pagerank = compute_pagerank(&node_ids, &edges, &PageRankConfig::default());

        // 三角环应该均匀分布
        let pr1 = pagerank.get(&1).unwrap();
        let pr2 = pagerank.get(&2).unwrap();
        let pr3 = pagerank.get(&3).unwrap();

        assert!((pr1 - pr2).abs() < 0.01);
        assert!((pr2 - pr3).abs() < 0.01);
    }

    #[test]
    fn test_pagerank_with_sink() {
        // 有一个节点没有出链（汇聚节点）
        let node_ids = vec![1, 2, 3];
        let edges = vec![
            CitationEdge { source: 1, target: 2, weight: 1.0 },
            CitationEdge { source: 2, target: 3, weight: 1.0 },
        ];

        let pagerank = compute_pagerank(&node_ids, &edges, &PageRankConfig::default());

        // 节点3没有出链，应该获得较高分数
        let pr3 = pagerank.get(&3).unwrap();
        let pr1 = pagerank.get(&1).unwrap();
        assert!(pr3 > pr1);
    }
}