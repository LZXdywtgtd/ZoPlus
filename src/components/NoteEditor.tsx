//! 笔记编辑器组件
//!
//! 提供笔记编辑功能，支持标题、内容和标签的编辑

import React, { useState, useEffect } from 'react';
import {
  Modal,
  Typography,
  Input,
  Button,
  Space,
  Tag,
  Tooltip,
  Divider,
  message,
} from 'antd';
import {
  SaveOutlined,
  CloseOutlined,
  PlusOutlined,
  ExportOutlined,
} from '@ant-design/icons';
import type { Note } from '../utils/tauriCommands';
import {
  saveNoteToItem,
  updateNote,
  exportNoteAsMarkdown,
} from '../utils/tauriCommands';

const { Title, Text } = Typography;
const { TextArea } = Input;

interface NoteEditorProps {
  /** 笔记数据（新建时为 null） */
  note: Note;
  /** 文献ID */
  itemId: number;
  /** 保存回调 */
  onSave: (note: Note) => void;
  /** 取消回调 */
  onCancel: () => void;
  /** 是否为编辑模式 */
  isEditMode?: boolean;
}

/**
 * 笔记编辑器组件
 *
 * 提供以下功能：
 * - 编辑笔记标题
 * - 编辑笔记内容（支持 Markdown）
 * - 管理笔记标签
 * - 保存笔记到 Zotero itemNotes 表
 * - 导出笔记为 Markdown 格式
 */
const NoteEditor: React.FC<NoteEditorProps> = ({
  note,
  itemId,
  onSave,
  onCancel,
  isEditMode = false,
}) => {
  // 本地状态
  const [title, setTitle] = useState(note.title);
  const [content, setContent] = useState(note.content);
  const [tags, setTags] = useState<string[]>(note.tags);
  const [newTag, setNewTag] = useState('');
  const [isSaving, setIsSaving] = useState(false);
  const [isDirty, setIsDirty] = useState(false);

  // 监听笔记变化
  useEffect(() => {
    setTitle(note.title);
    setContent(note.content);
    setTags([...note.tags]);
    setIsDirty(false);
  }, [note]);

  //标记为已修改
  const markDirty = () => {
    if (!isDirty) {
      setIsDirty(true);
    }
  };

  // 添加标签
  const handleAddTag = () => {
    const tag = newTag.trim();
    if (tag && !tags.includes(tag)) {
      setTags([...tags, tag]);
      markDirty();
    }
    setNewTag('');
  };

  // 移除标签
  const handleRemoveTag = (tagToRemove: string) => {
    setTags(tags.filter(t => t !== tagToRemove));
    markDirty();
  };

  // 保存笔记
  const handleSave = async () => {
    if (!title.trim()) {
      message.warning('请输入笔记标题');
      return;
    }

    if (!content.trim()) {
      message.warning('请输入笔记内容');
      return;
    }

    setIsSaving(true);

    try {
      // 更新笔记数据
      const updatedNote: Note = {
        ...note,
        title: title.trim(),
        content: content.trim(),
        tags: tags,
        updated_at: Date.now(),
        version: note.version + 1,
      };

      if (isEditMode) {
        // 编辑模式：更新笔记
        await updateNote(updatedNote);
        message.success('笔记已更新');
      } else {
        // 新建模式：保存笔记
        await saveNoteToItem(itemId, updatedNote);
        message.success('笔记已保存到 Zotero');
      }

      onSave(updatedNote);
    } catch (error) {
      console.error('[笔记编辑器] 保存失败:', error);
      message.error('保存失败: ' + (error instanceof Error ? error.message : String(error)));
    } finally {
      setIsSaving(false);
    }
  };

  // 导出 Markdown
  const handleExport = async () => {
    try {
      const updatedNote: Note = {
        ...note,
        title: title.trim(),
        content: content.trim(),
        tags: tags,
      };

      const markdown = await exportNoteAsMarkdown(updatedNote);

      // 创建下载
      const blob = new Blob([markdown], { type: 'text/markdown;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `笔记_${note.note_id}_${Date.now()}.md`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);

      message.success('导出成功');
    } catch (error) {
      message.error('导出失败: ' + (error instanceof Error ? error.message : String(error)));
    }
  };

  // 格式化时间
  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp);
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  // 模板显示名称
  const getTemplateName = (template: string) => {
    const names: Record<string, string> = {
      key_points: '要点笔记',
      methods: '方法笔记',
      conclusions: '结论笔记',
      critical: '批判性笔记',
      general: '通用笔记',
    };
    return names[template] || '通用笔记';
  };

  return (
    <Modal
      open
      title={null}
      footer={null}
      onCancel={onCancel}
      width={700}
      closable={false}
      centered
      styles={{
        body: { padding: 0 },
      }}
    >
      <div style={styles.container}>
        {/* 头部 */}
        <div style={styles.header}>
          <div style={styles.headerContent}>
            <Title level={4} style={styles.title}>
              {isEditMode ? '编辑笔记' : '新建笔记'}
            </Title>
            <Space split={<span style={styles.split}>|</span>}>
              <Text type="secondary">{note.item_title || '未知文献'}</Text>
              <Text type="secondary">{getTemplateName(note.template)}</Text>
            </Space>
            {note.page && (
              <Text type="secondary" style={styles.pageInfo}>
                第 {note.page} 页
              </Text>
            )}
          </div>
          <div style={styles.headerActions}>
            <Tooltip title="导出为 Markdown">
              <Button
                icon={<ExportOutlined />}
                onClick={handleExport}
              />
            </Tooltip>
            <Tooltip title="关闭">
              <Button icon={<CloseOutlined />} onClick={onCancel} />
            </Tooltip>
          </div>
        </div>

        <Divider style={styles.divider} />

        {/* 原文引用 */}
        {note.source_text && (
          <>
            <div style={styles.sourceSection}>
              <Text type="secondary" style={styles.sourceLabel}>
                原文引用：
              </Text>
              <div style={styles.sourceText}>
                {note.source_text}
              </div>
            </div>
            <Divider style={styles.divider} />
          </>
        )}

        {/* 编辑区域 */}
        <div style={styles.editorSection}>
          {/* 标题 */}
          <div style={styles.field}>
            <Text strong style={styles.fieldLabel}>标题</Text>
            <Input
              value={title}
              onChange={(e) => {
                setTitle(e.target.value);
                markDirty();
              }}
              placeholder="请输入笔记标题"
              maxLength={200}
              showCount
            />
          </div>

          {/* 内容 */}
          <div style={styles.field}>
            <Text strong style={styles.fieldLabel}>内容</Text>
            <TextArea
              value={content}
              onChange={(e) => {
                setContent(e.target.value);
                markDirty();
              }}
              placeholder="请输入笔记内容（支持 Markdown格式）"
              rows={10}
              maxLength={50000}
              showCount
              style={{ fontFamily: 'monospace' }}
            />
          </div>

          {/* 标签 */}
          <div style={styles.field}>
            <Text strong style={styles.fieldLabel}>标签</Text>
            <div style={styles.tagsContainer}>
              {tags.map((tag, index) => (
                <Tag
                  key={index}
                  closable
                  onClose={() => handleRemoveTag(tag)}
                  style={styles.tag}
                >
                  {tag}
                </Tag>
              ))}
              <Input
                value={newTag}
                onChange={(e) => setNewTag(e.target.value)}
                onPressEnter={handleAddTag}
                placeholder="输入标签后按回车添加"
                style={{ width: 150 }}
                size="small"
              />
              <Button
                icon={<PlusOutlined />}
                onClick={handleAddTag}
                size="small"
                disabled={!newTag.trim()}
              />
            </div>
          </div>
        </div>

        {/* 底部操作 */}
        <div style={styles.footer}>
          <Space>
            <Button
              type="primary"
              icon={<SaveOutlined />}
              onClick={handleSave}
              loading={isSaving}
            >
              {isEditMode ? '更新' : '保存'}
            </Button>
            <Button onClick={onCancel} disabled={isSaving}>
              取消
            </Button>
          </Space>
          <Text type="secondary" style={styles.footerText}>
            {isDirty &&<span style={styles.dirtyIndicator}>*</span>}
            版本: {note.version}
            {note.created_at && ` | 创建时间: ${formatDate(note.created_at)}`}
          </Text>
        </div>
      </div>
    </Modal>
  );
};

// 样式
const styles: { [key: string]: React.CSSProperties } = {
  container: {
    maxHeight: '80vh',
    overflow: 'auto',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    padding: '16px 24px',
    backgroundColor: '#f5f5f5',
  },
  headerContent: {
    flex: 1,
  },
  headerActions: {
    display: 'flex',
    gap: 8,
  },
  title: {
    marginBottom: 8,
  },
  split: {
    color: '#d9d9d9',
    margin: '0 8px',
  },
  pageInfo: {
    marginTop: 4,
    display: 'block',
  },
  divider: {
    margin: '12px 0',
  },
  sourceSection: {
    padding: '0 24px 16px',
  },
  sourceLabel: {
    marginBottom: 8,
    display: 'block',
  },
  sourceText: {
    padding: '12px 16px',
    backgroundColor: '#fffbfe',
    borderLeft: '3px solid #1890ff',
    fontStyle: 'italic',
    color: '#666',
    borderRadius: 4,
  },
  editorSection: {
    padding: '0 24px',
  },
  field: {
    marginBottom: 16,
  },
  fieldLabel: {
    display: 'block',
    marginBottom: 8,
  },
  tagsContainer: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: 8,
    alignItems: 'center',
  },
  tag: {
    marginRight: 0,
  },
  footer: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '12px 24px',
    backgroundColor: '#f5f5f5',
    borderTop: '1px solid #e8e8e8',
  },
  footerText: {
    fontSize: 12,
  },
  dirtyIndicator: {
    color: '#faad14',
  },
};

export default NoteEditor;