//! PDF 标注列表组件
//!
//! 本组件用于显示当前 PDF 的所有标注列表，
//! 支持点击标注跳转到对应位置，显示标注预览和删除功能

import React, { useState, useMemo } from 'react';
import { List, Button, Tag, Popconfirm, Empty, message } from 'antd';
import {
  DeleteOutlined,
  HighlightOutlined,
  BorderOutlined,
  CiCircleOutlined,
  ArrowUpOutlined,
  EditOutlined,
  FileTextOutlined,
} from '@ant-design/icons';
import type { Annotation, AnnotationType, PdfPoint } from './AnnotationLayer';
import { colorToCss } from './AnnotationLayer';

// ============== 图标映射 ==============

/** 标注类型图标映射 */
const ANNOTATION_ICONS: Record<AnnotationType, React.ReactNode> = {
  highlight: <HighlightOutlined />,
  rectangle: <BorderOutlined />,
  ellipse: <CiCircleOutlined />,
  arrow: <ArrowUpOutlined />,
  free_draw: <EditOutlined />,
  text_note: <FileTextOutlined />,
};

/** 标注类型名称映射 */
const ANNOTATION_LABELS: Record<AnnotationType, string> = {
  highlight: '高亮',
  rectangle: '矩形',
  ellipse: '椭圆',
  arrow: '箭头',
  free_draw: '自由绘制',
  text_note: '文本笔记',
};

// ============== 组件属性 ==============

interface AnnotationListProps {
  /** 标注列表 */
  annotations: Annotation[];
  /** 当前页码 */
  currentPage: number;
  /** 选中的标注 ID */
  selectedId?: string;
  /** 是否显示页码过滤 */
  showPageFilter?: boolean;
  /** 是否显示删除按钮 */
  showDeleteButton?: boolean;
  /** 是否显示颜色标签 */
  showColorTag?: boolean;
  /** 点击标注回调 */
  onAnnotationClick?: (annotation: Annotation) => void;
  /** 删除标注回调 */
  onAnnotationDelete?: (annotationId: string) => void;
  /** 跳转回调 */
  onNavigate?: (page: number, position: PdfPoint) => void;
}

// ============== 组件实现 ==============

/**
 * PDF 标注列表组件
 * 显示标注列表，支持点击跳转和删除
 */
const AnnotationList: React.FC<AnnotationListProps> = ({
  annotations,
  currentPage,
  selectedId,
  showPageFilter = true,
  showDeleteButton = true,
  showColorTag = true,
  onAnnotationClick,
  onAnnotationDelete,
  onNavigate,
}) => {
  // 页码过滤状态
  const [filterPage, setFilterPage] = useState<number | null>(null);

  // 获取唯一页码列表
  const pageNumbers = useMemo(() => {
    const pages = new Set(annotations.map((a) => a.page));
    return Array.from(pages).sort((a, b) => a - b);
  }, [annotations]);

  // 过滤后的标注列表
  const filteredAnnotations = useMemo(() => {
    let result = [...annotations];

    // 按页码过滤
    if (filterPage !== null) {
      result = result.filter((a) => a.page === filterPage);
    }

    // 按类型排序，同一页的标注按创建时间排序
    result.sort((a, b) => {
      if (a.page !== b.page) {
        return a.page - b.page;
      }
      return a.created_at - b.created_at;
    });

    return result;
  }, [annotations, filterPage]);

  /**
   * 处理标注点击
   */
  const handleAnnotationClick = (annotation: Annotation) => {
    if (onAnnotationClick) {
      onAnnotationClick(annotation);
    }
    if (onNavigate) {
      // 跳转到标注所在页面
      const position = getAnnotationPosition(annotation);
      onNavigate(annotation.page, position);
    }
  };

  /**
   * 处理删除确认
   */
  const handleDeleteConfirm = (annotationId: string) => {
    if (onAnnotationDelete) {
      onAnnotationDelete(annotationId);
      message.success('标注已删除');
    }
  };

  /**
   * 获取标注位置（用于跳转）
   */
  const getAnnotationPosition = (annotation: Annotation): PdfPoint => {
    switch (annotation.annotation_type) {
      case 'highlight':
      case 'rectangle':
      case 'ellipse': {
        const data = annotation.data as { rect: { x: number; y: number } };
        return { x: data.rect.x, y: data.rect.y };
      }
      case 'arrow': {
        const data = annotation.data as { start: PdfPoint };
        return { x: data.start.x, y: data.start.y };
      }
      case 'free_draw': {
        const data = annotation.data as { points: PdfPoint[] };
        return data.points.length > 0 ? data.points[0] : { x: 0, y: 0 };
      }
      case 'text_note': {
        const data = annotation.data as { position: PdfPoint };
        return { x: data.position.x, y: data.position.y };
      }
      default:
        return { x: 0, y: 0 };
    }
  };

  /**
   * 获取标注描述文本
   */
  const getAnnotationDescription = (annotation: Annotation): string => {
    switch (annotation.annotation_type) {
      case 'highlight': {
        const data = annotation.data as { text: string };
        return data.text || '文本高亮';
      }
      case 'rectangle':
      case 'ellipse':
      case 'arrow':
      case 'free_draw':
        return '';
      case 'text_note': {
        const data = annotation.data as { content: string };
        return data.content || '文本笔记';
      }
      default:
        return '';
    }
  };

  /**
   * 渲染单个标注项
   */
  const renderAnnotationItem = (annotation: Annotation) => {
    const isSelected = annotation.id === selectedId;
    const isCurrentPage = annotation.page === currentPage;

    return (
      <List.Item
        key={annotation.id}
        style={{
          cursor: 'pointer',
          backgroundColor: isSelected ? '#e6f7ff' : 'transparent',
          borderLeft: isSelected ? '3px solid #1890ff' : '3px solid transparent',
          paddingLeft: isSelected ? '8px' : '11px',
          transition: 'all 0.2s ease',
        }}
        onClick={() => handleAnnotationClick(annotation)}
        actions={
          showDeleteButton
            ? [
                <Popconfirm
                  key="delete"
                  title="确定要删除这条标注吗？"
                  onConfirm={(e) => {
                    e?.stopPropagation();
                    handleDeleteConfirm(annotation.id);
                  }}
                  onCancel={(e) => e?.stopPropagation()}
                  okText="确定"
                  cancelText="取消"
                >
                  <Button
                    type="text"
                    size="small"
                    danger
                    icon={<DeleteOutlined />}
                    onClick={(e) => e.stopPropagation()}
                  />
                </Popconfirm>,
              ]
            : []
        }
      >
        <List.Item.Meta
          avatar={
            <div
              style={{
                width: 32,
                height: 32,
                borderRadius: 4,
                backgroundColor: colorToCss(annotation.color),
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: '#fff',
                fontSize: 14,
              }}
            >
              {ANNOTATION_ICONS[annotation.annotation_type]}
            </div>
          }
          title={
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <span>{ANNOTATION_LABELS[annotation.annotation_type]}</span>
              <Tag
                color={isCurrentPage ? 'blue' : 'default'}
                style={{ margin: 0 }}
              >
                第 {annotation.page} 页
              </Tag>
              {showColorTag && (
                <Tag
                  style={{
                    margin: 0,
                    backgroundColor: colorToCss(annotation.color),
                    border: 'none',
                  }}
                >
                  {annotation.annotation_type === 'highlight'
                    ? '高亮'
                    : annotation.annotation_type === 'rectangle'
                    ? '矩形'
                    : annotation.annotation_type === 'ellipse'
                    ? '椭圆'
                    : annotation.annotation_type === 'arrow'
                    ? '箭头'
                    : annotation.annotation_type === 'free_draw'
                    ? '自由绘制'
                    : '文本笔记'}
                </Tag>
              )}
            </div>
          }
          description={
            getAnnotationDescription(annotation) && (
              <div
                style={{
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                  maxWidth: 200,
                  color: '#999',
                }}
              >
                {getAnnotationDescription(annotation)}
              </div>
            )
          }
        />
      </List.Item>
    );
  };

  return (
    <div style={styles.container}>
      {/* 页码过滤 */}
      {showPageFilter && pageNumbers.length > 0 && (
        <div style={styles.filterBar}>
          <span style={styles.filterLabel}>筛选页面：</span>
          <div style={styles.pageButtons}>
            <Button
              type={filterPage === null ? 'primary' : 'default'}
              size="small"
              onClick={() => setFilterPage(null)}
            >
              全部
            </Button>
            {pageNumbers.map((page) => (
              <Button
                key={page}
                type={filterPage === page ? 'primary' : 'default'}
                size="small"
                onClick={() => setFilterPage(page)}
              >
                {page}
              </Button>
            ))}
          </div>
        </div>
      )}

      {/* 标注列表 */}
      {filteredAnnotations.length > 0 ? (
        <List
          size="small"
          dataSource={filteredAnnotations}
          renderItem={renderAnnotationItem}
          style={styles.list}
        />
      ) : (
        <Empty
          image={Empty.PRESENTED_IMAGE_SIMPLE}
          description={
            filterPage !== null
              ? '该页面暂无标注'
              : '暂无标注'
          }
          style={{ marginTop: 24 }}
        />
      )}

      {/* 统计信息 */}
      {annotations.length > 0 && (
        <div style={styles.stats}>
          共 {annotations.length} 条标注
          {filterPage !== null && `，其中第 ${filterPage} 页 ${filteredAnnotations.length} 条`}
        </div>
      )}
    </div>
  );
};

// ============== 样式 ==============

const styles: { [key: string]: React.CSSProperties } = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: '#fff',
  },
  filterBar: {
    padding: '8px 12px',
    borderBottom: '1px solid #f0f0f0',
  },
  filterLabel: {
    fontSize: 12,
    color: '#666',
    marginRight: 8,
  },
  pageButtons: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: 4,
    marginTop: 4,
  },
  list: {
    flex: 1,
    overflow: 'auto',
  },
  stats: {
    padding: '8px 12px',
    borderTop: '1px solid #f0f0f0',
    fontSize: 12,
    color: '#999',
    textAlign: 'center',
  },
};

export default AnnotationList;
