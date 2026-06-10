//! 智能笔记生成器组件
//!
//! 提供基于 AI 的智能笔记生成功能，支持多种笔记模板

import React, { useState } from 'react';
import { Button, Select, Space, message, Tooltip } from 'antd';
import {
  LoadingOutlined,
  EditOutlined,
} from '@ant-design/icons';
import {
  generateNote,
  generateNotesBatch,
  type Note,
  type NoteTemplateType,
} from '../utils/tauriCommands';
import NoteEditor from './NoteEditor';

interface NoteGeneratorProps {
  /** 文献ID */
  itemId: number;
  /** PDF密钥（用于批量生成） */
  pdfKey?: string;
  /** 选中的原文内容（单条生成时使用） */
  sourceText?: string;
  /** 页码 */
  page?: number;
  /** 是否显示为完整按钮（带下拉菜单） */
  showDropdown?: boolean;
  /** 保存笔记回调 */
  onSaveNote?: (note: Note) => void;
  /** 生成进度回调（批量生成时使用） */
  onProgress?: (current: number, total: number) => void;
}

/// 笔记模板选项
const NOTE_TEMPLATES: { value: NoteTemplateType; label: string; description: string }[] = [
  {
    value: 'key_points',
    label: '要点笔记',
    description: '提取文本中的关键信息点，适合快速回顾',
  },
  {
    value: 'methods',
    label: '方法笔记',
    description: '详细记录研究方法、技术路线和实验设计',
  },
  {
    value: 'conclusions',
    label: '结论笔记',
    description: '总结主要发现、结论和研究贡献',
  },
  {
    value: 'critical',
    label: '批判性笔记',
    description: '批判性分析：优点、局限性和潜在改进方向',
  },
  {
    value: 'general',
    label: '通用笔记',
    description: '自由格式笔记，可根据内容灵活记录',
  },
];

/**
 * 智能笔记生成器组件
 *
 * 提供以下功能：
 * - 生成单条笔记（基于选中文字）
 * - 批量生成笔记（基于多个高亮）
 * - 支持多种笔记模板
 * - 笔记编辑和保存
 */
const NoteGenerator: React.FC<NoteGeneratorProps> = ({
  itemId,
  pdfKey,
  sourceText,
  page,
  showDropdown = false,
  onSaveNote,
  onProgress,
}) => {
  // 本地状态
  const [isLoading, setIsLoading] = useState(false);
  const [showEditor, setShowEditor] = useState(false);
  const [generatedNote, setGeneratedNote] = useState<Note | null>(null);
  const [selectedTemplate, setSelectedTemplate] = useState<NoteTemplateType>('key_points');
  const [isBatchMode, setIsBatchMode] = useState(false);

  // 生成单条笔记
  const handleGenerateSingle = async () => {
    if (isLoading) return;

    if (!sourceText) {
      message.warning('请先选中要生成笔记的文本');
      return;
    }

    setIsLoading(true);
    setIsBatchMode(false);

    try {
      console.log('[笔记生成器] 开始生成单条笔记: item_id=', itemId);
      const note = await generateNote(itemId, sourceText, page || null, selectedTemplate);
      setGeneratedNote(note);
      setShowEditor(true);
      message.success('笔记生成成功');
    } catch (error) {
      console.error('[笔记生成器] 生成笔记失败:', error);
      message.error('笔记生成失败: ' + (error instanceof Error ? error.message : String(error)));
    } finally {
      setIsLoading(false);
    }
  };

  // 批量生成笔记
  const handleGenerateBatch = async () => {
    if (isLoading || !pdfKey) return;

    setIsLoading(true);
    setIsBatchMode(true);

    try {
      console.log('[笔记生成器] 开始批量生成笔记: item_id=', itemId, 'pdf_key=', pdfKey);
      const notes = await generateNotesBatch(itemId, pdfKey, selectedTemplate);

      if (notes.length === 0) {
        message.warning('没有找到可用的标注内容，请先在 PDF 中添加高亮');
        return;
      }

      // 显示进度
      if (onProgress) {
        onProgress(notes.length, notes.length);
      }

      // 逐个保存笔记
      for (let i = 0; i < notes.length; i++) {
        const note = notes[i];
        try {
          await onSaveNote?.(note);
          if (onProgress) {
            onProgress(i + 1, notes.length);
          }
        } catch (err) {
          console.error('[笔记生成器] 保存笔记失败:', err);
        }
      }

      message.success(`成功生成 ${notes.length} 条笔记`);
    } catch (error) {
      console.error('[笔记生成器] 批量生成笔记失败:', error);
      message.error('批量生成失败: ' + (error instanceof Error ? error.message : String(error)));
    } finally {
      setIsLoading(false);
      setIsBatchMode(false);
    }
  };

  // 保存笔记回调
  const handleSave = (note: Note) => {
    onSaveNote?.(note);
    setShowEditor(false);
    setGeneratedNote(null);
    message.success('笔记已保存');
  };

  // 取消回调
  const handleCancel = () => {
    setShowEditor(false);
    setGeneratedNote(null);
  };

  // 按钮图标
  const getIcon = () => {
    if (isLoading) {
      return <LoadingOutlined />;
    }
    return <EditOutlined />;
  };

  // 提示文本
  const getTooltip = () => {
    if (isLoading) {
      return isBatchMode ? '正在批量生成笔记...' : '正在生成笔记...';
    }
    if (showDropdown) {
      return '生成笔记';
    }
    return 'AI 生成笔记';
  };

  // 完整按钮（带下拉菜单）
  if (showDropdown) {
    return (
      <>
        <Space>
          <Select
            value={selectedTemplate}
            onChange={setSelectedTemplate}
            options={NOTE_TEMPLATES.map(t => ({
              value: t.value,
              label: t.label,
            }))}
            style={{ width: 120 }}
            disabled={isLoading}
          />

          <Tooltip title="基于选中文字生成笔记">
            <Button
              icon={getIcon()}
              onClick={handleGenerateSingle}
              disabled={isLoading || !sourceText}
            >
              生成笔记
            </Button>
          </Tooltip>

          <Tooltip title="基于所有高亮批量生成笔记">
            <Button
              icon={getIcon()}
              onClick={handleGenerateBatch}
              disabled={isLoading || !pdfKey}
            >
              批量生成
            </Button>
          </Tooltip>
        </Space>

        {showEditor && generatedNote && (
          <NoteEditor
            note={generatedNote}
            itemId={itemId}
            onSave={handleSave}
            onCancel={handleCancel}
          />
        )}
      </>
    );
  }

  // 简洁按钮
  return (
    <>
      <Tooltip title={getTooltip()}>
        <Button
          icon={getIcon()}
          onClick={handleGenerateSingle}
          disabled={isLoading || !sourceText}
          size="small"
        />
      </Tooltip>

      {showEditor && generatedNote && (
        <NoteEditor
          note={generatedNote}
          itemId={itemId}
          onSave={handleSave}
          onCancel={handleCancel}
        />
      )}
    </>
  );
};

export default NoteGenerator;