//! 主界面布局页面
//!
//! 左侧导航栏 + 右侧内容区域布局

import { Layout, Menu, Typography, Space } from 'antd';
import {
  FileTextOutlined,
  SearchOutlined,
  CloudSyncOutlined,
  SettingOutlined,
  FormatPainterOutlined,
  RobotOutlined,
  SwapOutlined,
} from '@ant-design/icons';
import { useState, useEffect } from 'react';
import DbStatusAlert from '../components/DbStatusAlert';
import ItemList from '../components/ItemList';
import Search from '../pages/Search';
import CitationFormatter from '../pages/CitationFormatter';
import AIChat from '../pages/AIChat';
import ArticleComparison from '../pages/ArticleComparison';
import useAppStore from '../store/appStore';

const { Sider, Content } = Layout;
const { Title, Text } = Typography;

/// 菜单项类型
type MenuItemKey = 'items' | 'search' | 'sync' | 'settings' | 'citation' | 'aichat' | 'comparison';

/// 主界面布局页面组件
function Dashboard() {
  // 当前选中的菜单项
  const [selectedKey, setSelectedKey] = useState<MenuItemKey>('items');
  // 批量对比选中的文献ID
  const { setSelectedItemIds } = useAppStore();

  // 页面加载时检查是否有预选的对比文献
  useEffect(() => {
    const storedIds = localStorage.getItem('zoplus_comparison_ids');
    if (storedIds) {
      try {
        const ids = JSON.parse(storedIds);
        if (Array.isArray(ids) && ids.length > 0) {
          setSelectedItemIds(ids);
          setSelectedKey('comparison');
          // 清除存储以避免重复使用
          localStorage.removeItem('zoplus_comparison_ids');
        }
      } catch (e) {
        console.error('Failed to parse stored comparison IDs:', e);
      }
    }
  }, [setSelectedItemIds]);

  // 渲染右侧内容区域
  const renderContent = () => {
    switch (selectedKey) {
      case 'items':
        // 文献列表页面（当前默认展示）
        return (
          <Space direction="vertical" style={{ width: '100%' }} size="large">
            <DbStatusAlert />
            <ItemList />
          </Space>
        );
      case 'search':
        // 搜索页面
        return <Search />;
      case 'sync':
        // 同步页面（预留）
        return (
          <Space direction="vertical" style={{ width: '100%' }} size="large">
            <Title level={4}>云同步</Title>
            <Text type="secondary">同步功能开发中...</Text>
          </Space>
        );
      case 'settings':
        // 设置页面（预留）
        return (
          <Space direction="vertical" style={{ width: '100%' }} size="large">
            <Title level={4}>设置</Title>
            <Text type="secondary">设置功能开发中...</Text>
          </Space>
        );
      case 'citation':
        // 参考文献格式化页面
        return <CitationFormatter />;
      case 'aichat':
        // AI 跨文献问答页面
        return <AIChat />;
      case 'comparison':
        // 文献对比页面
        return <ArticleComparison />;
      default:
        return null;
    }
  };

  return (
    <Layout style={{ minHeight: '100vh' }}>
      {/* 左侧导航栏 */}
      <Sider
        theme="light"
        style={{
          borderRight: '1px solid #f0f0f0',
        }}
      >
        <div
          style={{
            height: 64,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            borderBottom: '1px solid #f0f0f0',
          }}
        >
          <Title level={4} style={{ margin: 0 }}>
            ZoPlus
          </Title>
        </div>
        <Menu
          mode="inline"
          selectedKeys={[selectedKey]}
          onClick={({ key }) => setSelectedKey(key as MenuItemKey)}
          style={{ borderRight: 0 }}
          items={[
            {
              key: 'items',
              icon: <FileTextOutlined />,
              label: '文献列表',
            },
            {
              key: 'search',
              icon: <SearchOutlined />,
              label: '全文搜索',
            },
            {
              key: 'sync',
              icon: <CloudSyncOutlined />,
              label: '云同步',
            },
            {
              key: 'settings',
              icon: <SettingOutlined />,
              label: '设置',
            },
            {
              key: 'citation',
              icon: <FormatPainterOutlined />,
              label: '引用格式化',
            },
            {
              key: 'aichat',
              icon: <RobotOutlined />,
              label: 'AI 问答',
            },
            {
              key: 'comparison',
              icon: <SwapOutlined />,
              label: '文献对比',
            },
          ]}
        />
      </Sider>

      {/* 右侧内容区域 */}
      <Layout>
        <Content
          style={{
            padding: 24,
            background: '#fff',
            overflow: 'auto',
          }}
        >
          {renderContent()}
        </Content>
      </Layout>
    </Layout>
  );
}

export default Dashboard;