//! 文献引用关系图谱页面
//!
//! 提供可视化的文献引用关系图谱，支持：
//! - 基于 ECharts 的交互式关系图谱
//! - 节点大小代表被引次数
//! - 边代表引用关系
//! - 按关键词筛选
//! - 点击节点查看文献详情
//! - 支持导出图谱为 PNG 格式

import { useState, useEffect, useRef } from 'react';
import {
  Card,
  Button,
  Space,
  Typography,
  message,
  Spin,
  Alert,
  Input,
  Slider,
  Table,
  Tag,
  Modal,
  Tooltip,
  Select,
} from 'antd';
import {
  NodeIndexOutlined,
  DownloadOutlined,
  ReloadOutlined,
  SearchOutlined,
  EyeOutlined,
  InfoCircleOutlined,
} from '@ant-design/icons';
import * as echarts from 'echarts';
import {
  getCitationGraph,
  getKeyPapers,
  getPaperCitations,
  type CitationGraph,
  type KeyPaper,
  type PaperCitations,
} from '../utils/tauriCommands';

const { Title, Text } = Typography;
const { Search } = Input;
const { Option } = Select;

/// 文献引用关系图谱页面组件
function CitationGraphPage() {
  // 图谱数据
  const [graph, setGraph] = useState<CitationGraph | null>(null);
  // 关键文献列表
  const [keyPapers, setKeyPapers] = useState<KeyPaper[]>([]);
  // 当前选中节点的文献详情
  const [paperCitations, setPaperCitations] = useState<PaperCitations | null>(null);
  // 加载状态
  const [loading, setLoading] = useState<boolean>(false);
  // 错误信息
  const [error, setError] = useState<string | null>(null);
  // 最小被引次数过滤
  const [minCitations, setMinCitations] = useState<number>(0);
  // 关键词筛选
  const [keyword, setKeyword] = useState<string>('');
  // 布局类型
  const [layoutType, setLayoutType] = useState<string>('force');
  // 详情弹窗
  const [detailModalVisible, setDetailModalVisible] = useState<boolean>(false);
  // ECharts 容器引用
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);

  // 加载图谱数据
  const loadCitationGraph = async () => {
    setLoading(true);
    setError(null);

    try {
      const result = await getCitationGraph(minCitations);
      setGraph(result);
      message.success(`加载完成：${result.total_nodes} 个节点，${result.total_edges} 条边（耗时 ${result.compute_time_ms}ms）`);
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err);
      setError(errMsg);
      message.error(`加载失败: ${errMsg}`);
    } finally {
      setLoading(false);
    }
  };

  // 加载关键文献
  const loadKeyPapers = async () => {
    try {
      const result = await getKeyPapers(20);
      setKeyPapers(result);
    } catch (err) {
      console.error('加载关键文献失败:', err);
    }
  };

  // 初始化加载
  useEffect(() => {
    loadCitationGraph();
    loadKeyPapers();
  }, []);

  // 渲染图谱
  useEffect(() => {
    if (!graph || !chartRef.current) return;

    // 初始化图表
    if (!chartInstanceRef.current) {
      chartInstanceRef.current = echarts.init(chartRef.current);
    }

    const chart = chartInstanceRef.current;

    // 过滤节点（根据关键词）
    let filteredNodes = graph.nodes;
    if (keyword.trim()) {
      const kw = keyword.toLowerCase();
      filteredNodes = graph.nodes.filter(
        (n) =>
          n.title.toLowerCase().includes(kw) ||
          n.authors.toLowerCase().includes(kw) ||
          n.year.includes(kw)
      );
    }

    // 获取过滤后的节点ID集合
    const filteredNodeIds = new Set(filteredNodes.map((n) => n.item_id));

    // 过滤边（只保留两端节点都在过滤结果中的边）
    const filteredEdges = graph.edges.filter(
      (e) => filteredNodeIds.has(e.source) && filteredNodeIds.has(e.target)
    );

    // 转换为 ECharts 格式
    const echartsNodes = filteredNodes.map((node) => ({
      id: String(node.item_id),
      name: node.title.length > 30 ? node.title.substring(0, 30) + '...' : node.title,
      fullTitle: node.title,
      authors: node.authors,
      year: node.year,
      citationCount: node.citation_count,
      pagerank: node.pagerank,
      symbolSize: node.node_size,
      category: node.citation_count > 10 ? 1 : 0,
    }));

    const echartsEdges = filteredEdges.map((edge) => ({
      source: String(edge.source),
      target: String(edge.target),
      weight: edge.weight,
    }));

    // 配置
    const option: echarts.EChartsOption = {
      title: {
        text: `文献引用关系图谱（${filteredNodes.length} 节点 / ${filteredEdges.length} 边）`,
        left: 'center',
        top: 10,
        textStyle: {
          fontSize: 16,
          fontWeight: 'normal',
        },
      },
      tooltip: {
        trigger: 'item',
        formatter: (params: any) => {
          if (params.dataType === 'node') {
            const node = params.data;
            return `
              <div style="font-size: 12px;">
                <strong>${node.fullTitle}</strong><br/>
                作者: ${node.authors}<br/>
                年份: ${node.year}<br/>
                被引: ${node.citationCount} 次<br/>
                PageRank: ${node.pagerank.toFixed(4)}
              </div>
            `;
          } else if (params.dataType === 'edge') {
            return `引用关系`;
          }
          return '';
        },
      },
      legend: {
        data: ['普通文献', '高被引文献'],
        top: 50,
        left: 'center',
      },
      series: [
        {
          type: 'graph',
          layout: layoutType as any,
          roam: true,
          draggable: true,
          label: {
            show: true,
            fontSize: 10,
            formatter: (params: any) => {
              const name = params.data.name || '';
              return name.length > 15 ? name.substring(0, 15) + '...' : name;
            },
          },
          lineStyle: {
            width: 1,
            color: '#999',
            curveness: 0.3,
          },
          emphasis: {
            focus: 'adjacency',
            lineStyle: {
              width: 3,
              color: '#5470c6',
            },
          },
          data: echartsNodes,
          links: echartsEdges,
          categories: [
            { name: '普通文献', itemStyle: { color: '#91cc75' } },
            { name: '高被引文献', itemStyle: { color: '#ee6666' } },
          ],
          force: {
            repulsion: 200,
            gravity: 0.1,
            edgeLength: [50, 200],
            layoutAnimation: true,
          },
        },
      ],
    };

    chart.setOption(option, true);

    // 点击节点事件
    chart.off('click');
    chart.on('click', (params: any) => {
      if (params.dataType === 'node') {
        const itemId = parseInt(params.data.id);
        handleNodeClick(itemId);
      }
    });

    // 窗口大小变化时重新调整
    const handleResize = () => {
      chart.resize();
    };
    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
    };
  }, [graph, keyword, layoutType]);

  // 处理节点点击
  const handleNodeClick = async (itemId: number) => {
    try {
      const result = await getPaperCitations(itemId);
      setPaperCitations(result);
      setDetailModalVisible(true);
    } catch (err) {
      message.error('获取文献详情失败');
    }
  };

  // 导出图谱为 PNG
  const handleExportPng = () => {
    if (!chartInstanceRef.current) {
      message.warning('图表未初始化');
      return;
    }

    const url = chartInstanceRef.current.getDataURL({
      type: 'png',
      pixelRatio: 2,
      backgroundColor: '#fff',
    });

    const link = document.createElement('a');
    link.download = `citation_graph_${Date.now()}.png`;
    link.href = url;
    link.click();

    message.success('图谱已导出为 PNG');
  };

  // 关键文献表格列
  const keyPapersColumns = [
    {
      title: '排名',
      dataIndex: 'index',
      key: 'index',
      width: 60,
      render: (_: any, __: any, index: number) => index + 1,
    },
    {
      title: '标题',
      dataIndex: 'title',
      key: 'title',
      ellipsis: true,
      render: (title: string) => (
        <Tooltip title={title}>
          <Text ellipsis={{ tooltip: title }} style={{ maxWidth: 300 }}>
            {title}
          </Text>
        </Tooltip>
      ),
    },
    {
      title: '作者',
      dataIndex: 'authors',
      key: 'authors',
      ellipsis: true,
      width: 150,
    },
    {
      title: '年份',
      dataIndex: 'year',
      key: 'year',
      width: 80,
    },
    {
      title: '被引次数',
      dataIndex: 'citation_count',
      key: 'citation_count',
      width: 80,
      sorter: (a: KeyPaper, b: KeyPaper) => a.citation_count - b.citation_count,
    },
    {
      title: 'PageRank',
      dataIndex: 'pagerank',
      key: 'pagerank',
      width: 100,
      render: (pr: number) => pr.toFixed(4),
      sorter: (a: KeyPaper, b: KeyPaper) => a.pagerank - b.pagerank,
    },
    {
      title: '操作',
      key: 'action',
      width: 80,
      render: (_: any, record: KeyPaper) => (
        <Button
          type="link"
          size="small"
          icon={<EyeOutlined />}
          onClick={() => handleNodeClick(record.item_id)}
        >
          详情
        </Button>
      ),
    },
  ];

  // 渲染详情弹窗
  const renderDetailModal = () => {
    if (!paperCitations) return null;

    return (
      <Modal
        title={
          <Space>
            <InfoCircleOutlined />
            <span>文献详情</span>
          </Space>
        }
        open={detailModalVisible}
        onCancel={() => setDetailModalVisible(false)}
        footer={[
          <Button key="close" onClick={() => setDetailModalVisible(false)}>
            关闭
          </Button>,
        ]}
        width={900}
      >
        <Card size="small" style={{ marginBottom: 16 }}>
          <Text strong style={{ fontSize: 16 }}>{paperCitations.title}</Text>
          <br />
          <Text type="secondary">{paperCitations.authors}</Text>
        </Card>

        <Space direction="vertical" style={{ width: '100%' }} size="large">
          {/* 施引文献 */}
          <Card size="small" title={`被引用（${paperCitations.total_cited_by} 篇）`}>
            {paperCitations.cited_by.length > 0 ? (
              <Table
                size="small"
                pagination={{ pageSize: 5 }}
                dataSource={paperCitations.cited_by}
                columns={[
                  { title: '标题', dataIndex: 'title', ellipsis: true },
                  { title: '作者', dataIndex: 'authors', width: 150, ellipsis: true },
                  { title: '年份', dataIndex: 'year', width: 80 },
                ]}
                rowKey="item_id"
              />
            ) : (
              <Text type="secondary">暂无被引数据</Text>
            )}
          </Card>

          {/* 被引文献 */}
          <Card size="small" title={`参考文献（${paperCitations.total_references} 篇）`}>
            {paperCitations.references.length > 0 ? (
              <Table
                size="small"
                pagination={{ pageSize: 5 }}
                dataSource={paperCitations.references}
                columns={[
                  { title: '标题', dataIndex: 'title', ellipsis: true },
                  { title: '作者', dataIndex: 'authors', width: 150, ellipsis: true },
                  { title: '年份', dataIndex: 'year', width: 80 },
                ]}
                rowKey="item_id"
              />
            ) : (
              <Text type="secondary">暂无参考文献数据</Text>
            )}
          </Card>
        </Space>
      </Modal>
    );
  };

  return (
    <Space direction="vertical" style={{ width: '100%' }} size="large">
      <Card>
        <Title level={4}>文献引用关系图谱</Title>
        <Text type="secondary">
          可视化展示文献间的引用关系，节点大小代表被引次数。支持拖拽、缩放、筛选等功能。
          计算耗时：{graph?.compute_time_ms ?? 0}ms（处理 {graph?.total_nodes ?? 0} 个节点）
        </Text>
      </Card>

      {/* 控制面板 */}
      <Card>
        <Space wrap style={{ marginBottom: 16 }}>
          <Text>最小被引次数：</Text>
          <Slider
            min={0}
            max={50}
            value={minCitations}
            onChange={(value) => setMinCitations(value)}
            style={{ width: 200 }}
          />
          <Text type="secondary">（当前: {minCitations}）</Text>

          <Button
            icon={<ReloadOutlined />}
            onClick={loadCitationGraph}
            loading={loading}
          >
            重新加载
          </Button>

          <Button
            icon={<DownloadOutlined />}
            onClick={handleExportPng}
            disabled={!graph}
          >
            导出 PNG
          </Button>
        </Space>

        <Space wrap style={{ marginBottom: 16 }}>
          <Search
            placeholder="按标题/作者/年份筛选"
            allowClear
            style={{ width: 300 }}
            prefix={<SearchOutlined />}
            value={keyword}
            onChange={(e) => setKeyword(e.target.value)}
          />

          <Text>布局：</Text>
          <Select value={layoutType} onChange={setLayoutType} style={{ width: 120 }}>
            <Option value="force">力导向图</Option>
            <Option value="circular">环形图</Option>
            <Option value="none">无布局</Option>
          </Select>
        </Space>

        {error && (
          <Alert
            message="加载失败"
            description={error}
            type="error"
            closable
            onClose={() => setError(null)}
            style={{ marginBottom: 16 }}
          />
        )}
      </Card>

      {/* 加载状态 */}
      {loading && (
        <Card>
          <Space direction="vertical" style={{ width: '100%', textAlign: 'center' }}>
            <Spin size="large" />
            <Text>正在加载引用图谱数据...</Text>
          </Space>
        </Card>
      )}

      {/* 图谱展示 */}
      {!loading && graph && (
        <>
          <Card
            title="引用关系图谱"
            extra={
              <Tag color="blue">
                {graph.total_nodes} 节点 / {graph.total_edges} 边
              </Tag>
            }
          >
            <div
              ref={chartRef}
              style={{
                width: '100%',
                height: 600,
                minHeight: 400,
              }}
            />
          </Card>

          {/* 关键文献推荐 */}
          <Card
            title={
              <Space>
                <NodeIndexOutlined />
                <span>关键文献推荐（基于 PageRank）</span>
              </Space>
            }
            extra={
              <Tag color="green">{keyPapers.length} 篇</Tag>
            }
          >
            <Table
              size="small"
              pagination={{ pageSize: 10 }}
              dataSource={keyPapers}
              columns={keyPapersColumns}
              rowKey="item_id"
            />
          </Card>
        </>
      )}

      {/* 详情弹窗 */}
      {renderDetailModal()}
    </Space>
  );
}

export default CitationGraphPage;