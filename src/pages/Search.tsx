//! 搜索页面
//!
//! 整合搜索框和搜索结果，提供完整的全文搜索功能。

import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Typography, Space, Alert, Card, Button, message } from 'antd';
import { BuildOutlined, DeleteOutlined } from '@ant-design/icons';
import SearchBox from '../components/SearchBox';
import SearchResults, { SearchResultItem } from '../components/SearchResults';

const { Title, Text } = Typography;

/// 搜索请求参数
interface SearchRequest {
    query: string;
    offset?: number;
    limit?: number;
    fuzzy?: boolean;
    fuzzy_distance?: number;
}

/// 搜索响应结构
interface SearchResponse {
    results: SearchResultItem[];
    total: number;
    offset: number;
    limit: number;
}

/// 索引状态结构
interface IndexStatus {
    index_path: string;
    document_count: number;
    exists: boolean;
}

/// 搜索页面组件
function Search() {
    // 搜索结果列表
    const [results, setResults] = useState<SearchResultItem[]>([]);
    // 总结果数
    const [total, setTotal] = useState<number>(0);
    // 当前偏移
    const [offset, setOffset] = useState<number>(0);
    // 每页数量
    const [pageSize, setPageSize] = useState<number>(20);
    // 加载状态
    const [loading, setLoading] = useState<boolean>(false);
    // 索引构建中状态
    const [buildingIndex, setBuildingIndex] = useState<boolean>(false);
    // 索引状态
    const [indexStatus, setIndexStatus] = useState<IndexStatus | null>(null);

    // 初始化搜索索引
    const initializeIndex = useCallback(async () => {
        try {
            setLoading(true);
            await invoke('init_search_index');
            // 获取索引状态
            const status = await invoke<IndexStatus>('get_index_status');
            setIndexStatus(status);
        } catch (error) {
            message.error(`初始化索引失败: ${error}`);
        } finally {
            setLoading(false);
        }
    }, []);

    // 执行搜索
    const handleSearch = useCallback(async (query: string, fuzzy: boolean) => {
        if (!query.trim()) {
            message.warning('请输入搜索关键词');
            return;
        }

        try {
            setLoading(true);
            const request: SearchRequest = {
                query,
                offset: 0,
                limit: pageSize,
                fuzzy,
                fuzzy_distance: 2,
            };
            const response = await invoke<SearchResponse>('search_papers', { request });
            setResults(response.results);
            setTotal(response.total);
            setOffset(response.offset);
        } catch (error) {
            message.error(`搜索失败: ${error}`);
        } finally {
            setLoading(false);
        }
    }, [pageSize]);

    // 处理分页变化
    const handlePageChange = useCallback(async (page: number, newPageSize: number) => {
        const newOffset = (page - 1) * newPageSize;
        setOffset(newOffset);
        setPageSize(newPageSize);

        try {
            setLoading(true);
            const request: SearchRequest = {
                query: '', // 保持当前搜索关键词
                offset: newOffset,
                limit: newPageSize,
            };
            const response = await invoke<SearchResponse>('search_papers', { request });
            setResults(response.results);
            setTotal(response.total);
        } catch (error) {
            message.error(`获取分页结果失败: ${error}`);
        } finally {
            setLoading(false);
        }
    }, []);

    // 处理清除索引
    const handleClearIndex = useCallback(async () => {
        try {
            setLoading(true);
            await invoke('clear_search_index');
            message.success('索引已清除');
            setResults([]);
            setTotal(0);
        } catch (error) {
            message.error(`清除索引失败: ${error}`);
        } finally {
            setLoading(false);
        }
    }, []);

    // 处理构建索引
    const handleBuildIndex = useCallback(async () => {
        try {
            setBuildingIndex(true);
            await invoke('build_search_index');
            message.success('索引构建完成');
            // 刷新索引状态
            const status = await invoke<IndexStatus>('get_index_status');
            setIndexStatus(status);
        } catch (error) {
            message.error(`构建索引失败: ${error}`);
        } finally {
            setBuildingIndex(false);
        }
    }, []);

    // 组件挂载时初始化索引
    useEffect(() => {
        initializeIndex();
    }, [initializeIndex]);

    return (
        <Space direction="vertical" style={{ width: '100%' }} size="large">
            {/* 页面标题 */}
            <Title level={3}>全文搜索</Title>

            {/* 索引状态提示 */}
            {indexStatus && (
                <Alert
                    message={
                        <Space>
                            <Text>索引状态：</Text>
                            <Text strong>已收录 {indexStatus.document_count} 篇文献</Text>
                        </Space>
                    }
                    type={indexStatus.document_count > 0 ? 'success' : 'warning'}
                    showIcon
                    action={
                        <Space>
                            <Button
                                icon={<BuildOutlined />}
                                onClick={handleBuildIndex}
                                loading={buildingIndex}
                                size="small"
                            >
                                {indexStatus.document_count > 0 ? '重建索引' : '构建索引'}
                            </Button>
                            {indexStatus.document_count > 0 && (
                                <Button
                                    icon={<DeleteOutlined />}
                                    onClick={handleClearIndex}
                                    danger
                                    size="small"
                                >
                                    清除索引
                                </Button>
                            )}
                        </Space>
                    }
                />
            )}

            {/* 搜索框 */}
            <Card>
                <SearchBox
                    onSearch={handleSearch}
                    onClear={handleClearIndex}
                    disabled={loading || buildingIndex}
                    loading={loading}
                    placeholder="输入标题、作者、关键词等搜索文献..."
                />
            </Card>

            {/* 搜索结果 */}
            <SearchResults
                results={results}
                total={total}
                offset={offset}
                limit={pageSize}
                loading={loading}
                onPageChange={handlePageChange}
                emptyText="输入关键词开始搜索"
            />
        </Space>
    );
}

export default Search;