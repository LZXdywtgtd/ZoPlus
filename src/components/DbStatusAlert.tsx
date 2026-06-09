//! 数据库状态检测组件
//!
//! 页面加载时自动检测 Zotero 数据库连接状态

import { useEffect } from 'react';
import { Alert, Space, Typography } from 'antd';
import { DatabaseOutlined, CheckCircleOutlined, CloseCircleOutlined } from '@ant-design/icons';
import useAppStore from '../store/appStore';
import { checkDbStatus } from '../utils/tauriCommands';

const { Text } = Typography;

/// 数据库状态检测组件
/// 自动检测并在界面上显示 Zotero 数据库连接状态
function DbStatusAlert() {
  // 从状态管理获取数据库状态
  const { dbStatus, setDbStatus } = useAppStore();

  useEffect(() => {
    // 组件挂载时自动检测数据库状态
    const detectDbStatus = async () => {
      setDbStatus({ isChecking: true, error: null });

      try {
        const isConnected = await checkDbStatus();
        setDbStatus({
          isConnected,
          isChecking: false,
          error: isConnected ? null : '未检测到 Zotero 数据库文件',
        });
      } catch (error) {
        setDbStatus({
          isConnected: false,
          isChecking: false,
          error: error instanceof Error ? error.message : String(error),
        });
      }
    };

    detectDbStatus();
  }, [setDbStatus]);

  // 显示加载状态
  if (dbStatus.isChecking) {
    return (
      <Alert
        message="正在检测 Zotero 数据库..."
        type="info"
        showIcon
        icon={<DatabaseOutlined spin />}
      />
    );
  }

  // 显示连接成功
  if (dbStatus.isConnected) {
    return (
      <Alert
        message="数据库连接成功"
        description="已成功连接到 Zotero 数据库"
        type="success"
        showIcon
        icon={<CheckCircleOutlined />}
      />
    );
  }

  // 显示连接失败
  return (
    <Alert
      message="数据库连接失败"
      description={
        <Space direction="vertical">
          <Text>未能检测到 Zotero 数据库文件。</Text>
          <Text type="secondary">
            请确保已安装 Zotero 6.0 或更高版本，并确保数据库文件存在于：
          </Text>
          <Text code> %USERPROFILE%\Zotero\zotero.sqlite </Text>
        </Space>
      }
      type="error"
      showIcon
      icon={<CloseCircleOutlined />}
    />
  );
}

export default DbStatusAlert;