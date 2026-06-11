//! 拖拽区域组件
//!
//! 支持拖拽 PDF 文件到此处进行导入

import { useCallback } from 'react';
import { Upload, message } from 'antd';
import type { UploadProps } from 'antd';
import { InboxOutlined } from '@ant-design/icons';
import useAppStore from '../store/appStore';

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

  /** 处理文件导入 */
  const handleImport = useCallback(
    async (file: File) => {
      console.log('[DropZone] 开始导入文件:', file.name);
      setItemsLoading(true);

      try {
        // 获取文件路径
        // 注意：在 Tauri 中，可以通过 file.path 获取完整路径
        // 但如果 path不可用，则使用文件对象
        const filePath = (file as unknown as { path?: string }).path || file.name;

        // 调用后端导入
        const { invoke } = await import('@tauri-apps/api/core');
        const result = await invoke<{ item_id: number; title: string; file_path: string; message: string }>('import_file', {
          filePath: filePath,
          maxFileSize: 100 * 1024 * 1024, // 100MB
        });

        console.log('[DropZone] 导入成功:', result);
        message.success(result.message);

        // 触发回调
        onImportSuccess?.();
      } catch (error) {
        console.error('[DropZone] 导入失败:', error);
        message.error(`导入失败: ${error instanceof Error ? error.message : String(error)}`);
      } finally {
        setItemsLoading(false);
      }
    },
    [onImportSuccess, setItemsLoading]
  );

  /** 上传组件的属性配置 */
  const uploadProps: UploadProps = {
    name: 'file',
    multiple: false,
    accept: '.pdf',
    showUploadList: false,
    beforeUpload: (file) => {
      // 阻止默认上传行为，自己处理
      handleImport(file);
      return false;
    },
  };

  return (
    <Dragger {...uploadProps}>
      <p className="ant-upload-drag-icon">
        <InboxOutlined />
      </p>
      <p className="ant-upload-text">点击或拖拽 PDF 文件到此区域导入</p>
      <p className="ant-upload-hint">支持 PDF 格式文件，单个文件最大 100MB</p>
    </Dragger>
  );
}

export default DropZone;