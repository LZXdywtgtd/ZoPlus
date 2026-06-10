//! 笔记列表组件
//!
//! 显示指定文献的所有笔记，支持查看、编辑和删除

import React, { useState, useEffect, useCallback } from 'react';
import {
  List,
  Button,
  Space,
  Tag,
  Tooltip,
  Modal,
  message,
  Empty,
  Typography,
  Dropdown,
} from 'antd';
import {
  FileTextOutlined,
  EditOutlined,
  DeleteOutlined,
  ExportOutlined,
  ReloadOutlined,
  MoreOutlined,
  EyeOutlined,
} from '@ant-design/icons';
import type { MenuProps } from 'antd';
import type { Note } from '../utils/tauriCommands';
import {
  getNotesForItem,
  deleteNote,
  exportNoteAsMarkdown,
  exportAllNotesAsMarkdown,
} from '../utils/tauriCommands';
import NoteEditor from './NoteEditor';

const { Text, Paragraph } = Typography;

interface NoteListProps {
  /** 文献ID */
  itemId: number;
  /** 文献标题 */
  itemTitle?: string;
  /** 笔记变化回调（用于刷新） */
  onNotesChange?: () => void;
  /** 笔记点击回调 */
  onNoteClick?: (note: Note) => void;
}

/**
 * 笔记列表组件
 *
 * 提供以下功能：
 * - 显示指定文献的所有笔记
 * - 查看笔记详情
 * - 编辑笔记
 * - 删除笔记
 * - 导出单条笔记或批量导出为 Markdown
 */
const NoteList: React.FC<NoteListProps> = ({
  itemId,
  itemTitle,
  onNotesChange,
  onNoteClick,
}) => {
  // 本地状态
  const [notes, setNotes] = useState<Note[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedNote, setSelectedNote] = useState<Note | null>(null);
  const [showEditor, setShowEditor] = useState(false);
  const [showDetail, setShowDetail] = useState(false);
  const [detailNote, setDetailNote] = useState<Note | null>(null);

  // 加载笔记列表
  const loadNotes = useCallback(async () => {
    setIsLoading(true);
    try {
      const fetchedNotes = await getNotesForItem(itemId);
      setNotes(fetchedNotes);
    } catch (error) {
      console.error('[笔记列表] 加载笔记失败:', error);
      message.error('加载笔记失败');
    } finally {
      setIsLoading(false);
    }
  }, [itemId]);

  // 初始加载
  useEffect(() => {
    loadNotes();
  }, [loadNotes]);

  // 刷新列表
  const handleRefresh = () => {
    loadNotes();
    onNotesChange?.();
  };

  // 查看笔记详情
  const handleViewNote = (note: Note) => {
    setDetailNote(note);
    setShowDetail(true);
    onNoteClick?.(note);
  };

  // 编辑笔记
  const handleEditNote = (note: Note) => {
    setSelectedNote(note);
    setShowEditor(true);
  };

  // 保存笔记后更新列表
  const handleSaveNote = (updatedNote: Note) => {
    setNotes(notes.map(n => n.note_id === updatedNote.note_id ? updatedNote : n));
    setShowEditor(false);
    setSelectedNote(null);
    onNotesChange?.();
  };

  // 删除笔记
  const handleDeleteNote = (note: Note) => {
    Modal.confirm({
      title: '确认删除',
      content: '确定要删除这条笔记吗？此操作无法撤销。',
      okText: '删除',
      cancelText: '取消',
      okType: 'danger',
      onOk: async () => {
        try {
          await deleteNote(note.note_id);
          setNotes(notes.filter(n => n.note_id !== note.note_id));
          message.success('笔记已删除');
          onNotesChange?.();
        } catch (error) {
          console.error('[笔记列表] 删除笔记失败:', error);
          message.error('删除失败');
        }
      },
    });
  };

  // 导出单条笔记
  const handleExportNote = async (note: Note) => {
    try {
      const markdown = await exportNoteAsMarkdown(note);

      // 创建下载
      const blob = new Blob([markdown], { type: 'text/markdown;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `笔记_${note.title}_${Date.now()}.md`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);

      message.success('导出成功');
    } catch (error) {
      message.error('导出失败: ' + (error instanceof Error ? error.message : String(error)));
    }
  };

  // 批量导出所有笔记
  const handleExportAll = async () => {
    if (notes.length === 0) {
      message.warning('没有笔记可导出');
      return;
    }

    try {
      const markdown = await exportAllNotesAsMarkdown(notes, itemTitle || '文献笔记');

      // 创建下载
      const blob = new Blob([markdown], { type: 'text/markdown;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `笔记汇总_${itemTitle || itemId}_${Date.now()}.md`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);

      message.success('导出成功');
    } catch (error) {
      message.error('导出失败: ' + (error instanceof Error ? error.message : String(error)));
    }
  };

  // 获取笔记操作菜单
  const getNoteActions = (note: Note): MenuProps['items'] => [
    {
      key: 'view',
      icon: <EyeOutlined />,
      label: '查看',
      onClick: () => handleViewNote(note),
    },
    {
      key: 'edit',
      icon: <EditOutlined />,
      label: '编辑',
      onClick: () => handleEditNote(note),
    },
    {
      key: 'export',
      icon: <ExportOutlined />,
      label: '导出',
      onClick: () => handleExportNote(note),
    },
    {
      type: 'divider',
    },
    {
      key: 'delete',
      icon: <DeleteOutlined />,
      label: '删除',
      danger: true,
      onClick: () => handleDeleteNote(note),
    },
  ];

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

  // 渲染笔记列表项
  const renderNoteItem = (note: Note) => (
    <List.Item
      actions={[
        <Tooltip title="查看">
          <Button
            icon={<EyeOutlined />}
            onClick={() => handleViewNote(note)}
            size="small"
          />
        </Tooltip>,
        <Tooltip title="编辑">
          <Button
            icon={<EditOutlined />}
            onClick={() => handleEditNote(note)}
            size="small"
          />
        </Tooltip>,
        <Dropdown menu={{ items: getNoteActions(note) }} trigger={['click']}>
          <Button icon={<MoreOutlined />} size="small" />
        </Dropdown>,
      ]}
    >
      <List.Item.Meta
        title={
          <Space>
            <FileTextOutlined />
            <span
              style={{ cursor: 'pointer', color: '#1890ff' }}
              onClick={() => handleViewNote(note)}
            >
              {note.title}
            </span>
          </Space>
        }
        description={
          <Space direction="vertical" size="small" style={{ width: '100%' }}>
            {/* 标签和元信息 */}
            <Space size="small">
              <Tag color="blue">{getTemplateName(note.template)}</Tag>
              {note.page && <Text type="secondary">第 {note.page} 页</Text>}
              {note.tags.slice(0, 3).map((tag, index) => (
                <Tag key={index}>{tag}</Tag>
              ))}
              {note.tags.length > 3 && (
                <Text type="secondary">+{note.tags.length - 3}</Text>
              )}
            </Space>
            {/* 内容预览 */}
            {note.content && (
              <Paragraph
                type="secondary"
                ellipsis={{ rows: 2, expandable: false }}
                style={{ marginBottom: 0, fontSize: 12 }}
              >
                {note.content.substring(0, 200)}
              </Paragraph>
            )}
            {/* 时间 */}
            <Text type="secondary" style={{ fontSize: 11 }}>
              {formatDate(note.created_at)}
            </Text>
          </Space>
        }
      />
    </List.Item>
  );

  return (
    <>
      {/* 工具栏 */}
      <div style={styles.toolbar}>
        <Space>
          <Text strong>笔记列表</Text>
          <Text type="secondary">({notes.length} 条)</Text>
        </Space>
        <Space>
          <Tooltip title="刷新">
            <Button
              icon={<ReloadOutlined />}
              onClick={handleRefresh}
              loading={isLoading}
              size="small"
            />
          </Tooltip>
          <Tooltip title="导出所有笔记">
            <Button
              icon={<ExportOutlined />}
              onClick={handleExportAll}
              disabled={notes.length === 0}
              size="small"
            >
              导出全部
            </Button>
          </Tooltip>
        </Space>
      </div>

      {/* 笔记列表 */}
      <List
        loading={isLoading}
        dataSource={notes}
        renderItem={renderNoteItem}
        locale={{
          emptyText: (
            <Empty
              image={Empty.PRESENTED_IMAGE_SIMPLE}
              description={
                <Space direction="vertical">
                  <Text type="secondary">暂无笔记</Text>
                  <Text type="secondary" style={{ fontSize: 12 }}>
                    在 PDF 中选中文字，点击"AI 生成笔记"创建笔记
                  </Text>
                </Space>
              }
            />
          ),
        }}
        style={styles.list}
      />

      {/* 笔记详情弹窗 */}
      {showDetail && detailNote && (
        <Modal
          open
          title={detailNote.title}
          onCancel={() => {
            setShowDetail(false);
            setDetailNote(null);
          }}
          footer={
            <Space>
              <Button onClick={() => handleExportNote(detailNote)}>
                导出
              </Button>
              <Button
                type="primary"
                onClick={() => {
                  setShowDetail(false);
                  setDetailNote(null);
                  handleEditNote(detailNote);
                }}
              >
                编辑
              </Button>
            </Space>
          }
          width={700}
        >
          <div style={styles.detailContent}>
            <Space direction="vertical" size="middle" style={{ width: '100%' }}>
              {/* 元信息 */}
              <Space size="small">
                <Tag color="blue">{getTemplateName(detailNote.template)}</Tag>
                {detailNote.page && <Text type="secondary">第 {detailNote.page} 页</Text>}
                {detailNote.tags.map((tag, index) => (
                  <Tag key={index}>{tag}</Tag>
                ))}
              </Space>

              {/* 原文引用 */}
              {detailNote.source_text && (
                <div style={styles.sourceQuote}>
                  <Text type="secondary" style={styles.sourceLabel}>
                    原文引用：
                  </Text>
                  <blockquote style={styles.quoteBlock}>
                    {detailNote.source_text}
                  </blockquote>
                </div>
              )}

              {/* 内容 */}
              <div>
                <Text strong>内容：</Text>
                <Paragraph style={styles.contentParagraph}>
                  {detailNote.content}
                </Paragraph>
              </div>

              {/* 底部信息 */}
              <Text type="secondary" style={{ fontSize: 12 }}>
                创建时间: {formatDate(detailNote.created_at)} | 版本: {detailNote.version}
              </Text>
            </Space>
          </div>
        </Modal>
      )}

      {/* 笔记编辑器 */}
      {showEditor && selectedNote && (
        <NoteEditor
          note={selectedNote}
          itemId={itemId}
          onSave={handleSaveNote}
          onCancel={() => {
            setShowEditor(false);
            setSelectedNote(null);
          }}
          isEditMode
        />
      )}
    </>
  );
};

// 样式
const styles: { [key: string]: React.CSSProperties } = {
  toolbar: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '8px 16px',
    borderBottom: '1px solid #e8e8e8',
    backgroundColor: '#fafafa',
  },
  list: {
    maxHeight: 'calc(100vh - 200px)',
    overflow: 'auto',
  },
  detailContent: {
    maxHeight: '60vh',
    overflow: 'auto',
  },
  sourceQuote: {
    marginBottom: 16,
  },
  sourceLabel: {
    marginBottom: 8,
    display: 'block',
  },
  quoteBlock: {
    margin: 0,
    padding: '12px 16px',
    backgroundColor: '#fffbfe',
    borderLeft: '3px solid #1890ff',
    fontStyle: 'italic',
    color: '#666',
  },
  contentParagraph: {
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-word',
  },
};

export default NoteList;