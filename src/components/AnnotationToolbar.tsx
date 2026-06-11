//! PDF 标注工具栏组件
//!
//! 本组件提供标注工具的选择 UI，包括高亮、矩形、椭圆、箭头、自由绘制、文本笔记等工具

import React from 'react';
import { Button, Tooltip, Divider } from 'antd';
import {
  HighlightOutlined,
  BorderOutlined,
  MinusSquareOutlined,
  ArrowUpOutlined,
  EditOutlined,
  DeleteOutlined,
  SelectOutlined,
} from '@ant-design/icons';
import type { AnnotationMode } from './AnnotationLayer';

// 工具配置
interface ToolConfig {
  key: AnnotationMode;
  icon: React.ReactNode;
  label: string;
}

const TOOLS: ToolConfig[] = [
  { key: 'select', icon: <SelectOutlined />, label: '选择' },
  { key: 'highlight', icon: <HighlightOutlined />, label: '高亮' },
  { key: 'rectangle', icon: <BorderOutlined />, label: '矩形' },
  { key: 'ellipse', icon: <MinusSquareOutlined />, label: '椭圆' },
  { key: 'arrow', icon: <ArrowUpOutlined />, label: '箭头' },
  { key: 'free_draw', icon: <EditOutlined />, label: '自由绘制' },
];

/** 标注工具栏属性 */
interface AnnotationToolbarProps {
  /** 当前标注模式 */
  currentMode: AnnotationMode;
  /** 模式变化回调 */
  onModeChange: (mode: AnnotationMode) => void;
  /** 删除选中标注回调 */
  onDelete?: () => void;
  /** 是否有选中的标注 */
  hasSelection?: boolean;
  /** 是否禁用 */
  disabled?: boolean;
}

/**
 * PDF 标注工具栏组件
 * 提供标注工具的选择 UI
 */
const AnnotationToolbar: React.FC<AnnotationToolbarProps> = ({
  currentMode,
  onModeChange,
  onDelete,
  hasSelection = false,
  disabled = false,
}) => {
  return (
    <div style={styles.container}>
      <div style={styles.toolGroup}>
        {TOOLS.map((tool) => (
          <Tooltip key={tool.key} title={tool.label}>
            <Button
              type={currentMode === tool.key ? 'primary' : 'default'}
              icon={tool.icon}
              onClick={() => onModeChange(tool.key)}
              disabled={disabled}
              size="middle"
            >
              {tool.label}
            </Button>
          </Tooltip>
        ))}
      </div>

      {onDelete && (
        <>
          <Divider type="vertical" style={styles.divider} />
          <Tooltip title="删除选中标注">
            <Button
              danger
              icon={<DeleteOutlined />}
              onClick={onDelete}
              disabled={disabled || !hasSelection}
              size="middle"
            >
              删除
            </Button>
          </Tooltip>
        </>
      )}
    </div>
  );
};

// 样式
const styles: { [key: string]: React.CSSProperties } = {
  container: {
    display: 'flex',
    alignItems: 'center',
    padding: '8px 12px',
    backgroundColor: '#3d3d3d',
    borderBottom: '1px solid #555',
    flexWrap: 'wrap',
    gap: 8,
  },
  toolGroup: {
    display: 'flex',
    alignItems: 'center',
    gap: 4,
  },
  divider: {
    height: 24,
    margin: '0 8px',
    backgroundColor: '#555',
  },
};

export default AnnotationToolbar;