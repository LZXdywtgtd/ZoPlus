//!搜索结果列表组件
//!
//! 展示搜索结果，支持高亮显示匹配关键词，支持分页。

import { List, Typography, Space, Tag, Badge, Empty, Spin } from 'antd';
import { FileTextOutlined, UserOutlined, CalendarOutlined } from '@ant-design/icons';

const { Text, Title } = Typography;

/// 搜索结果项结构
export interface SearchResultItem {
    /** 文献ID */
    item_id: number;
    /** 标题 */
    title: string;
    /** 作者 */
    authors: string;
    /** 年份 */
    year: string;
    /** 摘要 */
    abstract_text: string;
    /** 关键词 */
    keywords: string;
    /** 全文路径 */
    fulltext_path: string;
    /** 标签 */
    tags: string;
    /** 相关度得分 */
    score: number;
    /** 高亮后的标题 */
    title_highlighted: string;
    /** 高亮后的作者 */
    authors_highlighted: string;
    /** 高亮后的摘要 */
    abstract_highlighted: string;
}

export interface SearchResultsProps {
    /**搜索结果列表 */
    results: SearchResultItem[];
    /** 总结果数 */
    total: number;
    /** 当前偏移 */
    offset: number;
    /** 每页数量 */
    limit: number;
    /**加载状态 */
    loading?: boolean;
    /** 空状态文本 */
    emptyText?: string;
    /** 结果点击回调 */
    onItemClick?: (item: SearchResultItem) => void;
    /** 分页变化回调 */
    onPageChange?: (page: number, pageSize: number) => void;
}

/// 解析高亮文本（将 **包裹的文本转换为 React 元素）
const parseHighlightedText = (text: string): React.ReactNode[] => {
    if (!text) return [];

    const parts: React.ReactNode[] = [];
    const regex = /\*\*(.+?)\*\*/g;
    let lastIndex = 0;
    let match;
    let key = 0;

    while ((match = regex.exec(text)) !== null) {
        // 添加匹配前的普通文本
        if (match.index > lastIndex) {
            parts.push(text.slice(lastIndex, match.index));
        }
        // 添加高亮文本（使用 React 样式）
        parts.push(
            <span key={key++} style={{ backgroundColor: '#fff3b0', fontWeight: 'bold' }}>
                {match[1]}
            </span>
        );
        lastIndex = regex.lastIndex;
    }

    // 添加剩余文本
    if (lastIndex < text.length) {
        parts.push(text.slice(lastIndex));
    }

    return parts;
};

/// 搜索结果列表组件
function SearchResults({
    results,
    total,
    offset,
    limit,
    loading = false,
    emptyText = '暂无搜索结果',
    onItemClick,
    onPageChange,
}: SearchResultsProps) {
    // 计算当前页码
    const currentPage = Math.floor(offset / limit) + 1;
    const pageSize = limit;

    // 处理分页变化
    const handlePageChange = (page: number, newPageSize: number) => {
        if (onPageChange) {
            onPageChange(page, newPageSize);
        }
    };

    // 渲染搜索结果项
    const renderItem = (item: SearchResultItem) => (
        <List.Item
            key={item.item_id}
            style={{
                padding: '16px',
                cursor: onItemClick ? 'pointer' : 'default',
                transition: 'background-color 0.2s',
            }}
            onClick={() => onItemClick && onItemClick(item)}
            onMouseEnter={(e) => {
                if (onItemClick) {
                    (e.currentTarget as HTMLElement).style.backgroundColor = '#f5f5f5';
                }
            }}
            onMouseLeave={(e) => {
                if (onItemClick) {
                    (e.currentTarget as HTMLElement).style.backgroundColor = 'transparent';
                }
            }}
        >
            <List.Item.Meta
                title={
                    <Space>
                        <FileTextOutlined />
                        <Title level={5} style={{ margin: 0 }}>
                            {parseHighlightedText(item.title_highlighted || item.title)}
                        </Title>
                       <Badge
                            count={`${(item.score * 100).toFixed(1)}%`}
                            style={{ backgroundColor: '#52c41a' }}
                        />
                    </Space>
                }
                description={
                    <Space direction="vertical" style={{ width: '100%' }} size="small">
                        {/* 作者信息 */}
                        <Space>
                            <UserOutlined />
                            <Text type="secondary">
                                {parseHighlightedText(item.authors_highlighted || item.authors) || '未知作者'}
                            </Text>
                        </Space>

                        {/* 年份信息 */}
                        {item.year && (
                            <Space>
                                <CalendarOutlined />
                                <Text type="secondary">{item.year}</Text>
                            </Space>
                        )}

                        {/* 摘要信息 */}
                        {(item.abstract_text || item.abstract_highlighted) && (
                            <Text
                                type="secondary"
                                ellipsis={{ rows: 2, expandable: true, symbol: '展开' } as any}
                                style={{ display: 'block', marginTop: '8px' }}
                            >
                                {parseHighlightedText(item.abstract_highlighted || item.abstract_text)}
                            </Text>
                        )}

                        {/* 标签信息 */}
                        {item.tags && (
                            <Space style={{ marginTop: '8px' }} wrap>
                                {item.tags.split(/[;,]/).filter(Boolean).map((tag, idx) => (
                                    <Tag key={idx} color="blue">{tag.trim()}</Tag>
                                ))}
                            </Space>
                        )}
                    </Space>
                }
            />
        </List.Item>
    );

    // 加载状态
    if (loading) {
        return (
            <div style={{ textAlign: 'center', padding: '50px 0' }}>
                <Spin size="large" tip="搜索中..." />
            </div>
        );
    }

    // 空状态
    if (results.length === 0) {
        return <Empty description={emptyText} image={Empty.PRESENTED_IMAGE_SIMPLE} />;
    }

    return (
        <List
            dataSource={results}
            renderItem={renderItem}
            pagination={{
                current: currentPage,
                pageSize: pageSize,
                total: total,
                showSizeChanger: true,
                showQuickJumper: true,
                showTotal: (total) => `共 ${total} 条结果`,
                onChange: handlePageChange,
                pageSizeOptions: ['10', '20', '50', '100'],
            }}
            loading={loading}
            style={{
                background: '#fff',
                borderRadius: '8px',
                boxShadow: '0 2px 8px rgba(0,0,0,0.1)',
            }}
        />
    );
}

export default SearchResults;