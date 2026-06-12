//! 拖拽区域组件
//!
//! 支持拖拽 PDF 文件到此处进行导入

import { useCallback, useState } from 'react';
import { Upload, message } from 'antd';
import type { UploadProps } from 'antd';
import { InboxOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import useAppStore from '../store/appStore';
import { logError, logInfo } from '../utils/tauriCommands';

const { Dragger } = Upload;

/// 拖拽区域组件属性
interface DropZoneProps {
  /** 导入成功后的回调函数 */
  onImportSuccess?: () => void;
}

/**
 * 拖拽区域组件
 *
 * 支持通过拖拽或点击上传 PDF 文件
 * 文件会被传递到 Rust 后端进行导入处理
 */
function DropZone({ onImportSuccess }: DropZoneProps) {
  const { setItemsLoading } = useAppStore();
  const [dragging, setDragging] = useState(false);

  /** 处理文件导入 */
  const handleImport = useCallback(
    async (filePath: string, fileName: string) => {
      logInfo(`[DropZone] 开始导入文件: ${filePath}`);
      setItemsLoading(true);

      try {
        // 调用后端导入
        const result = await invoke<{ item_id: number; title: string; file_path: string; message: string }>('import_file', {
          filePath: filePath,
          maxFileSize: 100 * 1024 * 1024, // 100MB
        });

        logInfo(`[DropZone] 导入成功: ${filePath} - ${result.title}`);
        message.success(result.message);

        // 触发回调
        onImportSuccess?.();
      } catch (error) {
        logError(`[DropZone] 导入失败: ${filePath}`, error);
        console.error('[DropZone] 导入失败:', error);
        message.error(`导入失败: ${fileName} - ${error instanceof Error ? error.message : String(error)}`);
      } finally {
        setItemsLoading(false);
      }
    },
    [onImportSuccess, setItemsLoading]
  );

  /** 处理拖拽结束 */
  const handleDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      setDragging(false);

      logInfo('[DropZone] 开始拖拽导入');

      // Tauri 的拖拽事件直接提供真实路径
      const files = Array.from(e.dataTransfer.files);

      for (const file of files as unknown as { path?: string; name: string }[]) {
        // file.path 是真实的绝对路径
        if (file.path) {
          try {
            await handleImport(file.path, file.name);
          } catch (err) {
            logError(`拖拽导入失败: ${file.name}`, err);
            message.error(`导入失败: ${file.name} - ${err}`);
          }
        }
      }
    },
    [handleImport]
  );

  /** 处理拖拽进入 */
  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragging(true);
  }, []);

  /** 处理拖拽离开 */
  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragging(false);
  }, []);

  /** 处理拖拽悬停 */
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
  }, []);

  /** 上传组件的属性配置 */
  const uploadProps: UploadProps = {
    name: 'file',
    multiple: false,
    accept: '.pdf',
    showUploadList: false,
    beforeUpload: (file) => {
      // 阻止默认上传行为，自己处理
      // 在 Tauri 环境中，可以通过 file.path 获取完整路径
      const filePath = (file as unknown as { path?: string }).path;
      if (filePath) {
        handleImport(filePath, file.name);
      } else {
        message.error('无法获取文件路径');
      }
      return false;
    },
  };

  return (
    <div
      onDrop={handleDrop}
      onDragEnter={handleDragEnter}
      onDragLeave={handleDragLeave}
      onDragOver={handleDragOver}
      style={{
        border: dragging ? '2px dashed #1890ff' : '2px dashed #d9d9d9',
        borderRadius: '8px',
        padding: '24px',
        transition: 'border-color 0.3s',
        backgroundColor: dragging ? '#f0f7ff' : 'transparent'
      }}
    >
      <Dragger {...uploadProps}>
        <p className="ant-upload-drag-icon">
          <InboxOutlined />
        </p>
        <p className="ant-upload-text">点击或拖拽 PDF 文件到此区域导入</p>
        <p className="ant-upload-hint">支持 PDF 格式文件，单个文件最大 100MB</p>
      </Dragger>
    </div>
  );
}

export default DropZone;