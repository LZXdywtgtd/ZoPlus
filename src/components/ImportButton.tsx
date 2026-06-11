//! 导入按钮组件
//!
//! 点击后弹出导入对话框，支持拖拽上传 PDF 文件

import { useState } from 'react';
import { Button, Modal, Space, message } from 'antd';
import { ImportOutlined, CloudUploadOutlined } from '@ant-design/icons';
import DropZone from './DropZone';

/// 导入按钮组件属性
interface ImportButtonProps {
  /** 导入成功后的回调函数 */
  onImportSuccess?: () => void;
}

/**
 * 导入按钮组件
 *
 * 点击后弹出导入对话框，支持拖拽上传 PDF 文件到 Zotero 数据库
 */
function ImportButton({ onImportSuccess }: ImportButtonProps) {
  // 控制对话框显示
  const [modalVisible, setModalVisible] = useState(false);

  /** 打开导入对话框 */
  const handleOpenImport = () => {
    setModalVisible(true);
  };

  /** 关闭导入对话框 */
  const handleCloseImport = () => {
    setModalVisible(false);
  };

  /** 导入成功处理 */
  const handleImportSuccess = () => {
    message.success('文件导入成功');
    handleCloseImport();
    // 触发回调刷新列表
    onImportSuccess?.();
  };

  return (
    <>
      <Button
        type="primary"
        icon={<ImportOutlined />}
        onClick={handleOpenImport}
      >
        导入文献
      </Button>

      <Modal
        title={
          <Space>
            <CloudUploadOutlined />
            <span>导入本地文献</span>
          </Space>
        }
        open={modalVisible}
        onCancel={handleCloseImport}
        footer={null}
        width={500}
        destroyOnClose
      >
        <Space direction="vertical" style={{ width: '100%' }} size="large">
          {/* 拖拽上传区域 */}
          <DropZone onImportSuccess={handleImportSuccess} />

          {/* 提示信息 */}
          <Space direction="vertical" style={{ width: '100%' }}>
            <p style={{ color: '#888', fontSize: '12px' }}>
              提示：
            </p>
            <ul style={{ color: '#888', fontSize: '12px', paddingLeft: '20px', margin: 0 }}>
              <li>支持 PDF 格式文件</li>
              <li>文件名将自动提取为文献标题</li>
              <li>文件将复制到 Zotero 存储目录</li>
              <li>单文件最大 100MB</li>
            </ul>
          </Space>
        </Space>
      </Modal>
    </>
  );
}

export default ImportButton;