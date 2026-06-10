//! 文献对比页面
//!
//! 提供多篇文献的多维度对比分析功能

import { useState, useEffect } from 'react';
import {
  Card,
  Button,
  Space,
  Typography,
  message,
  Table,
  Tag,
  Modal,
  List,
  Spin,
  Alert,
  Select,
} from 'antd';
import {
  SwapOutlined,
  DownloadOutlined,
  CopyOutlined,
  DeleteOutlined,
  EyeOutlined,
  CloseOutlined,
  LoadingOutlined,
} from '@ant-design/icons';
import {
  compareArticles,
  getComparisonResult,
  hasComparisonResult,
  exportComparison,
  type ArticleComparison,
  type Contradiction,
  type Consensus,
} from '../utils/tauriCommands';
import useAppStore from '../store/appStore';

const { Title, Text } = Typography;
const { Option } = Select;

/// 文献对比页面组件
function ArticleComparison() {
  // 从 store 获取选中的文献ID
  const { selectedItemIds, setSelectedItemIds, items } = useAppStore();
  // 选中的文献ID列表（本地状态，用于页面内调整）
  const [localSelectedIds, setLocalSelectedIds] = useState<number[]>([]);
  // 选中的文献信息（标题列表）
  const [selectedTitles, setSelectedTitles] = useState<string[]>([]);
  // 当前对比结果
  const [comparison, setComparison] = useState<ArticleComparison | null>(null);
  // 加载状态
  const [loading, setLoading] = useState<boolean>(false);
  // 导出格式
  const [exportFormat, setExportFormat] = useState<'markdown' | 'csv'>('markdown');
  // 查看详细弹窗
  const [detailModalVisible, setDetailModalVisible] = useState<boolean>(false);
  // 当前查看的维度
  const [currentDimension, setCurrentDimension] = useState<string>('');
  // 错误信息
  const [error, setError] = useState<string | null>(null);

  // 初始化：从 store 或 localStorage 加载选中的文献
  useEffect(() => {
    let ids: number[] = [];

    // 优先从 store 读取
    if (selectedItemIds.length > 0) {
      ids = selectedItemIds;
    } else {
      // 尝试从 localStorage 读取（从 ItemList 跳转过来时）
      const storedIds = localStorage.getItem('zoplus_comparison_ids');
      if (storedIds) {
        try {
          ids = JSON.parse(storedIds);
          setSelectedItemIds(ids);
          localStorage.removeItem('zoplus_comparison_ids');
        } catch (e) {
          console.error('Failed to parse stored comparison IDs:', e);
        }
      }
    }

    if (ids.length > 0) {
      setLocalSelectedIds(ids);
      // 根据 ID 获取标题
      const titles = ids.map((id) => {
        const item = items.find((i) => i.item_id === id);
        return item?.title || `文献 ${id}`;
      });
      setSelectedTitles(titles);
    }
  }, [selectedItemIds, items, setSelectedItemIds]);

  // 维度名称映射
  const dimensionNames: Record<string, string> = {
    research_questions: '研究问题',
    research_methods: '研究方法',
    key_conclusions: '关键结论',
    innovations: '创新点',
    limitations: '局限性',
    citations: '引用情况',
  };

  // 维度顺序
  const dimensionOrder = [
    'research_questions',
    'research_methods',
    'key_conclusions',
    'innovations',
    'limitations',
    'citations',
  ];

  /// 执行对比
  const handleCompare = async () => {
    if (localSelectedIds.length < 2) {
      message.warning('请至少选择2篇文献进行对比');
      return;
    }

    if (localSelectedIds.length > 5) {
      message.warning('最多支持5篇文献进行对比');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const result = await compareArticles(localSelectedIds);
      setComparison(result);
      message.success('对比完成');
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      setError(errMsg);
      message.error(`对比失败: ${errMsg}`);
    } finally {
      setLoading(false);
    }
  };

  /// 加载缓存的对比结果
  const handleLoadCached = async () => {
    if (localSelectedIds.length < 2) {
      message.warning('请先选择文献');
      return;
    }

    setLoading(true);
    try {
      const cached = await hasComparisonResult(localSelectedIds);
      if (cached) {
        const result = await getComparisonResult(localSelectedIds);
        if (result) {
          setComparison(result);
          message.success('已加载缓存的对比结果');
        }
      } else {
        message.info('没有缓存的对比结果');
      }
    } catch (err) {
      console.error('加载缓存失败:', err);
    } finally {
      setLoading(false);
    }
  };

  /// 导出对比结果
  const handleExport = async () => {
    if (!comparison) {
      message.warning('没有可导出的对比结果');
      return;
    }

    setLoading(true);
    try {
      const content = await exportComparison(comparison, exportFormat);

      // 创建下载
      const blob = new Blob([content], { type: exportFormat === 'markdown' ? 'text/markdown' : 'text/csv' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `文献对比_${comparison.comparison_id.slice(0, 8)}.${exportFormat === 'markdown' ? 'md' : 'csv'}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      message.success('导出成功');
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      message.error(`导出失败: ${errMsg}`);
    } finally {
      setLoading(false);
    }
  };

  /// 复制到剪贴板
  const handleCopy = async () => {
    if (!comparison) {
      message.warning('没有可复制的内容');
      return;
    }

    try {
      const content = await exportComparison(comparison, 'markdown');
      await navigator.clipboard.writeText(content);
      message.success('已复制到剪贴板');
    } catch (err) {
      message.error('复制失败');
    }
  };

  /// 查看维度详情
  const handleViewDimension = (dimension: string) => {
    setCurrentDimension(dimension);
    setDetailModalVisible(true);
  };

  /// 获取维度内容
  const getDimensionContent = (dimension: string): string[] => {
    if (!comparison) return [];
    const dims = comparison.dimensions as unknown as Record<string, string[]>;
    return dims[dimension] || [];
  };

  /// 渲染对比表格
  const renderComparisonTable = () => {
    if (!comparison) return null;

    return (
      <Card title="对比结果" extra={
        <Space>
          <Select value={exportFormat} onChange={setExportFormat} style={{ width: 120 }}>
            <Option value="markdown">Markdown</Option>
            <Option value="csv">CSV/Excel</Option>
          </Select>
          <Button icon={<DownloadOutlined />} onClick={handleExport}>
            导出
          </Button>
          <Button icon={<CopyOutlined />} onClick={handleCopy}>
            复制
          </Button>
        </Space>
      }>
        {/* 文献列表 */}
        <Alert
          message="参与对比的文献"
          description={
            <List
              size="small"
              dataSource={comparison.titles.map((title, i) => ({
                key: i,
                title,
                author: comparison.authors[i] || '未知',
                year: comparison.years[i] || '未知',
              }))}
              renderItem={(item) => (
                <List.Item>
                  <Text>
                    <Tag color="blue">{comparison.item_ids.indexOf(comparison.item_ids[item.key])}</Tag>
                    <Text strong>{item.title}</Text>
                    <Text type="secondary"> - {item.author} ({item.year})</Text>
                  </Text>
                </List.Item>
              )}
            />
          }
          type="info"
          style={{ marginBottom: 16 }}
        />

        {/* 维度表格 */}
        {dimensionOrder.map((dimKey) => {
          const dimContent = getDimensionContent(dimKey);
          if (dimContent.length === 0) return null;

          return (
            <Card
              key={dimKey}
              size="small"
              title={<Tag color="green">{dimensionNames[dimKey]}</Tag>}
              extra={
                <Button
                  type="link"
                  icon={<EyeOutlined />}
                  onClick={() => handleViewDimension(dimKey)}
                >
                  详细
                </Button>
              }
              style={{ marginBottom: 16 }}
            >
              <Table
                size="small"
                pagination={false}
                dataSource={dimContent.map((content, i) => ({
                  key: i,
                  articleIndex: `文献${i + 1}`,
                  content: content,
                }))}
                columns={[
                  {
                    title: '文献',
                    dataIndex: 'articleIndex',
                    width: 100,
                  },
                  {
                    title: dimensionNames[dimKey],
                    dataIndex: 'content',
                  },
                ]}
              />
            </Card>
          );
        })}

        {/* 矛盾点 */}
        {comparison.contradictions && comparison.contradictions.length > 0 && (
          <Card
            size="small"
            title={<Tag color="red">矛盾点分析</Tag>}
            style={{ marginBottom: 16 }}
          >
            <List
              size="small"
              dataSource={comparison.contradictions}
              renderItem={(item: Contradiction, index) => (
                <List.Item>
                  <Tag color="red">{index + 1}</Tag>
                  <Text>{item.description}</Text>
                  <Text type="secondary"> ({item.contradiction_type})</Text>
                </List.Item>
              )}
            />
          </Card>
        )}

        {/* 共识点 */}
        {comparison.consensus && comparison.consensus.length > 0 && (
          <Card
            size="small"
            title={<Tag color="green">共识点分析</Tag>}
            style={{ marginBottom: 16 }}
          >
            <List
              size="small"
              dataSource={comparison.consensus}
              renderItem={(item: Consensus, index) => (
                <List.Item>
                  <Tag color="green">{index + 1}</Tag>
                  <Text>{item.description}</Text>
                </List.Item>
              )}
            />
          </Card>
        )}

        {/* 引用关系 */}
        {comparison.citation_relations && comparison.citation_relations.length > 0 && (
          <Card
            size="small"
            title={<Tag color="blue">引用关系</Tag>}
          >
            <List
              size="small"
              dataSource={comparison.citation_relations}
              renderItem={(item) => (
                <List.Item>
                  <Text>
                    文献{item.from_index + 1} 引用了 文献{item.to_index + 1}
                  </Text>
                </List.Item>
              )}
            />
          </Card>
        )}
      </Card>
    );
  };

  /// 渲染维度详情弹窗
  const renderDimensionDetail = () => {
    if (!currentDimension || !comparison) return null;

    const dimContent = getDimensionContent(currentDimension);

    return (
      <Modal
        title={dimensionNames[currentDimension]}
        open={detailModalVisible}
        onCancel={() => setDetailModalVisible(false)}
        footer={[
          <Button key="close" onClick={() => setDetailModalVisible(false)}>
            关闭
          </Button>,
        ]}
        width={800}
      >
        {dimContent.map((content, i) => (
          <Card key={i} size="small" title={`文献${i + 1}: ${comparison.titles[i]}`} style={{ marginBottom: 16 }}>
            <Text style={{ whiteSpace: 'pre-wrap' }}>{content}</Text>
          </Card>
        ))}
      </Modal>
    );
  };

  return (
    <Space direction="vertical" style={{ width: '100%' }} size="large">
      <Card>
        <Title level={4}>智能文献对比</Title>
        <Text type="secondary">
          选择2-5篇文献，自动生成多维度对比表格。支持研究问题、研究方法、关键结论、创新点、局限性、引用情况等维度的对比分析。
        </Text>
      </Card>

      {/* 文献选择 */}
      <Card title="选择文献进行对比">
        <Space wrap style={{ marginBottom: 16 }}>
          <Text>已选择 {localSelectedIds.length} 篇文献（2-5篇）</Text>
        </Space>

        {error && (
          <Alert
            message="操作失败"
            description={error}
            type="error"
            closable
            onClose={() => setError(null)}
            style={{ marginBottom: 16 }}
          />
        )}

        <Space wrap>
          <Button
            type="primary"
            icon={<SwapOutlined />}
            onClick={handleCompare}
            loading={loading}
            disabled={localSelectedIds.length < 2 || localSelectedIds.length > 5}
          >
            开始对比
          </Button>
          <Button
            icon={<LoadingOutlined />}
            onClick={handleLoadCached}
            disabled={localSelectedIds.length < 2}
          >
            加载缓存
          </Button>
          {comparison && (
            <Button
              icon={<DeleteOutlined />}
              onClick={() => {
                setComparison(null);
                message.info('已清除对比结果');
              }}
            >
              清除结果
            </Button>
          )}
        </Space>

        {localSelectedIds.length > 0 && (
          <List
            size="small"
            bordered
            dataSource={selectedTitles.map((title, i) => ({
              key: localSelectedIds[i],
              title,
            }))}
            style={{ marginTop: 16 }}
            header={<Text strong>已选择的文献：</Text>}
            renderItem={(item) => (
              <List.Item
                actions={[
                  <Button
                    type="text"
                    danger
                    icon={<CloseOutlined />}
                    onClick={() => {
                      const newIds = localSelectedIds.filter((id) => id !== item.key);
                      const newTitles = selectedTitles.filter((_, i) => localSelectedIds[i] !== item.key);
                      setSelectedItemIds(newIds);
                      setSelectedTitles(newTitles);
                    }}
                  />
                ]}
              >
                {item.title}
              </List.Item>
            )}
          />
        )}
      </Card>

      {/* 加载状态 */}
      {loading && (
        <Card>
          <Space direction="vertical" style={{ width: '100%', textAlign: 'center' }}>
            <Spin size="large" indicator={<LoadingOutlined spin />} />
            <Text>正在生成对比，请稍候...</Text>
          </Space>
        </Card>
      )}

      {/* 对比结果 */}
      {!loading && comparison && renderComparisonTable()}

      {/* 维度详情弹窗 */}
      {renderDimensionDetail()}
    </Space>
  );
}

export default ArticleComparison;