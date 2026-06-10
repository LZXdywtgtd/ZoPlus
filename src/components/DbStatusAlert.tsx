//! 数据库状态检测组件
//!
//! 页面加载时自动检测 Zotero 数据库连接状态

import { useEffect, useState } from 'react';
import { Alert, Space, Typography, Button, Collapse } from 'antd';
import { DatabaseOutlined, CheckCircleOutlined, CloseCircleOutlined, SettingOutlined } from '@ant-design/icons';
import useAppStore from '../store/appStore';
import { checkDbStatus, getDbDiagnosis, selectDatabasePath } from '../utils/tauriCommands';
import type { DatabaseDiagnosis } from '../utils/tauriCommands';

const { Text, Paragraph } = Typography;
const { Panel } = Collapse;

/// 数据库状态检测组件
/// 自动检测并在界面上显示 Zotero 数据库连接状态
function DbStatusAlert() {
  // 从状态管理获取数据库状态
  const { dbStatus, setDbStatus } = useAppStore();
  // 诊断信息状态
  const [diagnosis, setDiagnosis] = useState<DatabaseDiagnosis | null>(null);
  // 手动选择数据库弹窗状态
  const [dbPathInput, setDbPathInput] = useState('');
  const [isSelecting, setIsSelecting] = useState(false);

  useEffect(() => {
    // 组件挂载时自动检测数据库状态
    const detectDbStatus = async () => {
      setDbStatus({ isChecking: true, error: null });

      try {
        const isConnected = await checkDbStatus();
        if (isConnected) {
          // 如果连接成功，获取诊断信息
          try {
            const diag = await getDbDiagnosis();
            setDiagnosis(diag);
          } catch (e) {
            console.error('[DbStatusAlert] 获取诊断信息失败:', e);
          }
        }
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

  // 处理手动选择数据库
  const handleSelectDatabase = async () => {
    if (!dbPathInput.trim()) {
      return;
    }

    setIsSelecting(true);
    try {
      await selectDatabasePath(dbPathInput.trim());
      // 重新检测数据库状态
      const isConnected = await checkDbStatus();
      if (isConnected) {
        const diag = await getDbDiagnosis();
        setDiagnosis(diag);
      }
      setDbStatus({
        isConnected,
        isChecking: false,
        error: isConnected ? null : '手动指定的数据库连接失败',
      });
    } catch (error) {
      setDbStatus({
        isConnected: false,
        isChecking: false,
        error: error instanceof Error ? error.message : String(error),
      });
    } finally {
      setIsSelecting(false);
    }
  };

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
        description={
          <Space direction="vertical">
            <Text>已成功连接到 Zotero 数据库</Text>
            {diagnosis && (
              <Collapse ghost size="small">
                <Panel header="查看数据库诊断信息" key="diag">
                  <Space direction="vertical" size="small">
                    <Text type="secondary">总表数: {diagnosis.total_tables}</Text>
                    <Text type="secondary">检测到的表: {diagnosis.all_tables.slice(0, 10).join(', ')}
                      {diagnosis.all_tables.length > 10 && ` ... (共 ${diagnosis.all_tables.length} 个)`}
                    </Text>
                    {diagnosis.optional_missing.length > 0 && (
                      <Text type="warning">
                        缺失的可选表: {diagnosis.optional_missing.join(', ')}
                      </Text>
                    )}
                  </Space>
                </Panel>
              </Collapse>
            )}
          </Space>
        }
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
        <Space direction="vertical" style={{ width: '100%' }}>
          <Text>未能自动检测到 Zotero 数据库文件。</Text>
          <Text type="secondary">
            请确保已安装 Zotero 6.0 或更高版本，并确保数据库文件存在于：
          </Text>
          <Text code copyable>%USERPROFILE%\Zotero\zotero.sqlite</Text>

          {/* 手动选择数据库 */}
          <Collapse ghost>
            <Panel
              header={
                <Space>
                  <SettingOutlined />
                  <span>手动选择数据库文件</span>
                </Space>
              }
              key="manual-select"
            >
              <Space direction="vertical" style={{ width: '100%' }}>
                <Paragraph type="secondary">
                  如果自动检测失败，您可以手动输入 Zotero 数据库的完整路径：
                </Paragraph>
                <Space.Compact style={{ width: '100%' }}>
                  <input
                    type="text"
                    placeholder="例如: D:\Zotero\Date-Directary\zotero.sqlite"
                    value={dbPathInput}
                    onChange={(e) => setDbPathInput(e.target.value)}
                    style={{
                      flex: 1,
                      padding: '4px 11px',
                      border: '1px solid #d9d9d9',
                      borderRadius: '6px',
                    }}
                  />
                  <Button
                    type="primary"
                    onClick={handleSelectDatabase}
                    loading={isSelecting}
                    disabled={!dbPathInput.trim()}
                  >
                    连接
                  </Button>
                </Space.Compact>
              </Space>
            </Panel>
          </Collapse>

          {/* 显示诊断信息 */}
          {diagnosis && (
            <Collapse ghost size="small">
              <Panel header="查看诊断信息（表名列表）" key="diag">
                <Space direction="vertical" size="small">
                  <Text type="secondary">检测到 {diagnosis.total_tables} 个表</Text>
                  <Text type="secondary" style={{ wordBreak: 'break-all' }}>
                    可用表: {diagnosis.all_tables.join(', ')}
                  </Text>
                  {diagnosis.required_missing.length > 0 && (
                    <Text type="danger">
                      缺失必需表: {diagnosis.required_missing.join(', ')}
                    </Text>
                  )}
                </Space>
              </Panel>
            </Collapse>
          )}
        </Space>
      }
      type="error"
      showIcon
      icon={<CloseCircleOutlined />}
    />
  );
}

export default DbStatusAlert;