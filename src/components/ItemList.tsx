//! 文献列表展示组件
//!
//! 以表格形式展示文献列表，支持加载状态和错误提示

import { useEffect, useRef } from 'react';
import { Table, Alert, Space, Typography, Empty, Tag } from 'antd';
import { FileTextOutlined, LoadingOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import useAppStore from '../store/appStore';
import { getItems } from '../utils/tauriCommands';
import type { ItemInfo } from '../store/appStore';

const { Text, Title } = Typography;

/// 文献列表表格列定义
const columns: ColumnsType<ItemInfo> = [
  {
    title: '文献ID',
    dataIndex: 'item_id',
    key: 'item_id',
    width: 100,
    sorter: (a, b) => a.item_id - b.item_id,
  },
  {
    title: '标题',
    dataIndex: 'title',
    key: 'title',
    ellipsis: true,
    render: (text: string) => (
      <Space>
        <FileTextOutlined />
        <Text strong>{text || '无标题'}</Text>
      </Space>
    ),
  },
  {
    title: '作者',
    dataIndex: 'authors',
    key: 'authors',
    ellipsis: true,
    render: (text: string) => (
      <Text type="secondary">{text || '未知作者'}</Text>
    ),
  },
  {
    title: '年份',
    dataIndex: 'year',
    key: 'year',
    width: 120,
    sorter: (a, b) => {
      const yearA = parseInt(a.year) || 0;
      const yearB = parseInt(b.year) || 0;
      return yearA - yearB;
    },
    render: (text: string) => {
      if (!text) return <Text type="secondary">-</Text>;
      return <Tag color="blue">{text}</Tag>;
    },
  },
];

/// 文献列表展示组件
function ItemList() {
  // 从状态管理获取文献列表和相关状态
  const { items, itemsLoading, itemsError, dbStatus } = useAppStore();

  // 加载状态追踪，避免重复调用
  const loadTriggerRef = useRef<{ isLoading: boolean; dbConnected: boolean }>({
    isLoading: false,
    dbConnected: false,
  });

  useEffect(() => {
    // 当数据库连接成功后，自动加载文献列表
    // 使用 loadTriggerRef 避免重复调用
    const state = useAppStore.getState();
    const isConnected = state.dbStatus.isConnected;

    if (
      isConnected &&
      state.items.length === 0 &&
      !state.itemsLoading &&
      !loadTriggerRef.current.isLoading &&
      !loadTriggerRef.current.dbConnected
    ) {
      loadTriggerRef.current.isLoading = true;
      loadTriggerRef.current.dbConnected = true;
      loadItems();
    }
  }, [dbStatus.isConnected]);

  // 加载文献列表
  const loadItems = async () => {
    useAppStore.getState().setItemsLoading(true);
    useAppStore.getState().setItemsError(null);

    try {
      console.log('[ItemList] 开始加载文献列表...');
      const data = await getItems();
      console.log('[ItemList] 文献列表加载完成，共', data.length, '条');
      useAppStore.getState().setItems(data);
    } catch (error) {
      console.error('[ItemList] 文献列表加载失败:', error);
      useAppStore.getState().setItemsError(
        error instanceof Error ? error.message : String(error)
      );
    } finally {
      useAppStore.getState().setItemsLoading(false);
      loadTriggerRef.current.isLoading = false;
    }
  };

  // 数据库未连接时显示提示
  if (!dbStatus.isConnected) {
    return (
      <Alert
        message="无法加载文献列表"
        description="请先确保 Zotero 数据库连接正常"
        type="warning"
        showIcon
      />
    );
  }

  // 显示加载状态
  if (itemsLoading) {
    return (
      <Space direction="vertical" style={{ width: '100%' }} size="large">
        <Title level={5}>文献列表</Title>
        <Alert
          message="正在加载文献列表..."
          type="info"
          showIcon
          icon={<LoadingOutlined spin />}
        />
      </Space>
    );
  }

  // 显示错误提示
  if (itemsError) {
    return (
      <Space direction="vertical" style={{ width: '100%' }} size="large">
        <Title level={5}>文献列表</Title>
        <Alert
          message="加载文献列表失败"
          description={itemsError}
          type="error"
          showIcon
        />
      </Space>
    );
  }

  // 空数据提示
  if (items.length === 0) {
    return (
      <Space direction="vertical" style={{ width: '100%' }} size="large">
        <Title level={5}>文献列表</Title>
        <Empty
          image={Empty.PRESENTED_IMAGE_SIMPLE}
          description="暂无文献数据"
        />
      </Space>
    );
  }

  // 渲染文献列表表格
  return (
    <Space direction="vertical" style={{ width: '100%' }} size="large">
      <Title level={5}>文献列表</Title>
      <Table
        columns={columns}
        dataSource={items}
        rowKey="item_id"
        pagination={{
          pageSize: 20,
          showSizeChanger: true,
          showQuickJumper: true,
          showTotal: (total) => `共 ${total} 条文献`,
        }}
        size="middle"
      />
    </Space>
  );
}

export default ItemList;