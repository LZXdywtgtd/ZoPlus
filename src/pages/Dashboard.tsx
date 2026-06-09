//! 主界面布局页面
//!
//! 左侧导航栏 + 右侧内容区域布局

import { Layout, Menu, Typography, Space } from 'antd';
import {
  FileTextOutlined,
  SearchOutlined,
  CloudSyncOutlined,
  SettingOutlined,
} from '@ant-design/icons';
import { useState } from 'react';
import DbStatusAlert from '../components/DbStatusAlert';
import ItemList from '../components/ItemList';

const { Sider, Content } = Layout;
const { Title, Text } = Typography;

/// 菜单项类型
type MenuItemKey = 'items' | 'search' | 'sync' | 'settings';

/// 主界面布局页面组件
function Dashboard() {
  // 当前选中的菜单项
  const [selectedKey, setSelectedKey] = useState<MenuItemKey>('items');

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
        // 搜索页面（预留）
        return (
          <Space direction="vertical" style={{ width: '100%' }} size="large">
            <Title level={4}>全文搜索</Title>
            <Text type="secondary">搜索功能开发中...</Text>
          </Space>
        );
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