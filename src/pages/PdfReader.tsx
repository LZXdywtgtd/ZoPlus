//! PDF 阅读页面
//!
//! 本页面整合了 PDF 阅读器、标注图层和标注列表，
//! 提供完整的 PDF 阅读和标注功能

import React, { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { Layout, Tooltip, Button, message } from 'antd';
import {
  ArrowLeftOutlined,
  HighlightOutlined,
  BorderOutlined,
  CiCircleOutlined,
  ArrowUpOutlined,
  EditOutlined,
  FileTextOutlined,
  UndoOutlined,
  SaveOutlined,
  FolderOutlined,
} from '@ant-design/icons';
import PdfViewer from '../components/PdfViewer';
import AnnotationLayer, {
  Annotation,
  AnnotationType,
  AnnotationMode,
  AnnotationColor,
  PdfPoint,
  getDefaultColor,
} from '../components/AnnotationLayer';
import AnnotationList from '../components/AnnotationList';
import { loadAnnotations, saveAnnotations } from '../utils/tauriCommands';
import type { PDFPageProxy } from 'pdfjs-dist';

// ============== 常量 ==============

/** 标注模式工具按钮配置 */
const TOOL_BUTTONS: Array<{
  mode: AnnotationMode;
  icon: React.ReactNode;
  tooltip: string;
  color: AnnotationColor;
}> = [
  {
    mode: 'select',
    icon: <FolderOutlined />,
    tooltip: '选择模式',
    color: { r: 128, g: 128, b: 128, a: 255 },
  },
  {
    mode: 'highlight',
    icon: <HighlightOutlined />,
    tooltip: '高亮标注',
    color: { r: 255, g: 255, b: 0, a: 128 },
  },
  {
    mode: 'rectangle',
    icon: <BorderOutlined />,
    tooltip: '矩形标注',
    color: { r: 255, g: 0, b: 0, a: 180 },
  },
  {
    mode: 'ellipse',
    icon: <CiCircleOutlined />,
    tooltip: '椭圆标注',
    color: { r: 0, g: 0, b: 255, a: 180 },
  },
  {
    mode: 'arrow',
    icon: <ArrowUpOutlined />,
    tooltip: '箭头标注',
    color: { r: 0, g: 255, b: 0, a: 200 },
  },
  {
    mode: 'free_draw',
    icon: <EditOutlined />,
    tooltip: '自由绘制',
    color: { r: 0, g: 0, b: 0, a: 255 },
  },
  {
    mode: 'text_note',
    icon: <FileTextOutlined />,
    tooltip: '文本笔记',
    color: { r: 255, g: 165, b: 0, a: 200 },
  },
];

// ============== 类型定义 ==============

interface PdfReaderProps {
  /** PDF 文件路径 */
  filePath?: string;
  /** PDF 文件名 */
  fileName?: string;
  /** 文献 ID（用于关联 Zotero） */
  itemId?: number;
}

// ============== 组件实现 ==============

/**
 * PDF 阅读页面组件
 * 整合 PDF 阅读器、标注图层和标注列表
 */
const PdfReader: React.FC<PdfReaderProps> = ({
  filePath: propFilePath,
  fileName: propFileName,
}) => {
  // 路由导航
  const navigate = useNavigate();

  // 状态
  const [filePath, setFilePath] = useState<string>(propFilePath || '');
  const [fileName, setFileName] = useState<string>(propFileName || '');
  const [annotations, setAnnotations] = useState<Annotation[]>([]);
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [scale, setScale] = useState<number>(1);
  const [rotation, setRotation] = useState<number>(0);
  const [mode, setMode] = useState<AnnotationMode>('select');
  const [color, setColor] = useState<AnnotationColor>(getDefaultColor('highlight'));
  const [selectedAnnotation, setSelectedAnnotation] = useState<Annotation | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [showAnnotationList, setShowAnnotationList] = useState<boolean>(true);
  const [viewport, setViewport] = useState<{
    scale: number;
    rotation: number;
    width: number;
    height: number;
  } | null>(null);

  /**
   * 加载标注数据
   */
  const loadAnnotationsData = useCallback(async () => {
    if (!filePath) return;

    setIsLoading(true);
    try {
      const result = await loadAnnotations(filePath);
      setAnnotations(result);
    } catch (error) {
      console.error('加载标注失败:', error);
      // 标注加载失败不影响主功能
    } finally {
      setIsLoading(false);
    }
  }, [filePath]);

  /**
   * 保存标注
   */
  const saveAnnotationsData = useCallback(async () => {
    if (!filePath) return;

    setIsLoading(true);
    try {
      await saveAnnotations(filePath, fileName, annotations);
      message.success('标注已保存');
    } catch (error) {
      console.error('保存标注失败:', error);
      message.error('保存标注失败');
    } finally {
      setIsLoading(false);
    }
  }, [filePath, fileName, annotations]);

  /**
   * 创建标注回调
   */
  const handleAnnotationCreate = useCallback(
    async (annotation: Annotation) => {
      setAnnotations((prev) => [...prev, annotation]);
      message.success('标注已创建');
    },
    []
  );

  /**
   * 更新标注回调
   */
  const handleAnnotationUpdate = useCallback((annotation: Annotation) => {
    setAnnotations((prev) =>
      prev.map((a) => (a.id === annotation.id ? annotation : a))
    );
  }, []);

  /**
   * 删除标注回调
   */
  const handleAnnotationDelete = useCallback(async (annotationId: string) => {
    setAnnotations((prev) => prev.filter((a) => a.id !== annotationId));
    // 如果删除了选中的标注，清空选中状态
    if (selectedAnnotation?.id === annotationId) {
      setSelectedAnnotation(null);
    }
  }, [selectedAnnotation]);

  /**
   * 点击标注回调
   */
  const handleAnnotationClick = useCallback((annotation: Annotation) => {
    setSelectedAnnotation(annotation);
  }, []);

  /**
   * 跳转到指定位置
   */
  const handleNavigate = useCallback((_page: number, _position: PdfPoint) => {
    // 目前只需要切换页码，位置跳转由视口控制
    setCurrentPage(_page);
  }, []);

  /**
   * 切换标注模式
   */
  const handleModeChange = (newMode: AnnotationMode) => {
    setMode(newMode);
    setColor(getDefaultColor(newMode as AnnotationType));
  };

  /**
   * PDF 加载完成回调
   */
  const handleLoadComplete = useCallback(
    (_totalPages: number) => {
      // 加载完成后读取已有标注
      loadAnnotationsData();
    },
    [loadAnnotationsData]
  );

  /**
   * 自定义渲染层
   */
  const renderLayer = useCallback(
    (_page: PDFPageProxy, _canvas: HTMLCanvasElement, vp: any) => {
      // 保存视口信息供标注层使用
      setViewport({
        scale: vp.scale,
        rotation: vp.rotation,
        width: vp.width,
        height: vp.height,
      });
    },
    []
  );

  /**
   * 返回上一页
   */
  const handleGoBack = () => {
    navigate(-1);
  };

  /**
   * 撤销（删除最后一条标注）
   */
  const handleUndo = () => {
    if (annotations.length > 0) {
      const lastAnnotation = annotations[annotations.length - 1];
      handleAnnotationDelete(lastAnnotation.id);
      message.info('已撤销');
    }
  };

  // 初始加载
  useEffect(() => {
    if (propFilePath) {
      setFilePath(propFilePath);
    }
    if (propFileName) {
      setFileName(propFileName);
    }
  }, [propFilePath, propFileName]);

  // 自动保存（标注变化后）
  useEffect(() => {
    if (annotations.length > 0 && filePath) {
      const timer = setTimeout(() => {
        saveAnnotationsData();
      }, 2000); // 2秒后自动保存
      return () => clearTimeout(timer);
    }
  }, [annotations, filePath, saveAnnotationsData]);

  // 页面离开时保存
  useEffect(() => {
    return () => {
      if (annotations.length > 0 && filePath) {
        saveAnnotationsData();
      }
    };
  }, []);

  return (
    <Layout style={styles.container}>
      {/* 顶部工具栏 */}
      <Layout.Header style={styles.header}>
        <div style={styles.headerContent}>
          {/* 左侧：返回按钮和文件名 */}
          <div style={styles.headerLeft}>
            <Tooltip title="返回">
              <Button
                type="text"
                icon={<ArrowLeftOutlined />}
                onClick={handleGoBack}
                style={{ color: '#fff' }}
              />
            </Tooltip>
            <span style={styles.fileName} title={fileName}>
              {fileName || '未命名.pdf'}
            </span>
          </div>

          {/* 中间：标注工具 */}
          <div style={styles.tools}>
            {TOOL_BUTTONS.map((btn) => (
              <Tooltip key={btn.mode} title={btn.tooltip}>
                <Button
                  type={mode === btn.mode ? 'primary' : 'text'}
                  icon={btn.icon}
                  onClick={() => handleModeChange(btn.mode)}
                  style={{
                    color: mode === btn.mode ? '#fff' : 'rgba(255,255,255,0.85)',
                    backgroundColor:
                      mode === btn.mode
                        ? 'rgba(255,255,255,0.2)'
                        : 'transparent',
                  }}
                />
              </Tooltip>
            ))}
          </div>

          {/* 右侧：操作按钮 */}
          <div style={styles.headerRight}>
            <Tooltip title="撤销">
              <Button
                type="text"
                icon={<UndoOutlined />}
                onClick={handleUndo}
                disabled={annotations.length === 0}
                style={{ color: '#fff' }}
              />
            </Tooltip>
            <Tooltip title="保存">
              <Button
                type="text"
                icon={<SaveOutlined />}
                onClick={saveAnnotationsData}
                loading={isLoading}
                style={{ color: '#fff' }}
              />
            </Tooltip>
            <Button
              type="text"
              onClick={() => setShowAnnotationList(!showAnnotationList)}
              style={{
                color: showAnnotationList ? '#1890ff' : '#fff',
              }}
            >
              {showAnnotationList ? '隐藏列表' : '显示列表'}
            </Button>
          </div>
        </div>
      </Layout.Header>

      {/* 主体内容 */}
      <Layout.Content style={styles.content}>
        {/* PDF 阅读器 */}
        <div style={styles.pdfContainer}>
          {filePath ? (
            <PdfViewer
              filePath={filePath}
              fileName={fileName}
              currentPage={currentPage}
              scale={scale}
              rotation={rotation}
              onPageChange={setCurrentPage}
              onScaleChange={setScale}
              onRotationChange={setRotation}
              onLoadComplete={handleLoadComplete}
              renderLayer={renderLayer}
            >
              {/* 标注层 */}
              <AnnotationLayer
                page={currentPage}
                annotations={annotations}
                mode={mode}
                color={color}
                enabled={mode !== 'select'}
                viewport={viewport}
                onAnnotationCreate={handleAnnotationCreate}
                onAnnotationUpdate={handleAnnotationUpdate}
                onAnnotationDelete={handleAnnotationDelete}
                onAnnotationClick={handleAnnotationClick}
                onAnnotationSelect={setSelectedAnnotation}
                onNavigateTo={handleNavigate}
              />
            </PdfViewer>
          ) : (
            <div style={styles.emptyState}>
              <p>暂无 PDF 文件</p>
            </div>
          )}
        </div>

        {/* 标注列表侧边栏 */}
        {showAnnotationList && filePath && (
          <Layout.Sider width={280} style={styles.sider}>
            <AnnotationList
              annotations={annotations}
              currentPage={currentPage}
              selectedId={selectedAnnotation?.id}
              onAnnotationClick={handleAnnotationClick}
              onAnnotationDelete={handleAnnotationDelete}
              onNavigate={handleNavigate}
            />
          </Layout.Sider>
        )}
      </Layout.Content>
    </Layout>
  );
};

// ============== 样式 ==============

const styles: { [key: string]: React.CSSProperties } = {
  container: {
    height: '100vh',
    backgroundColor: '#525659',
  },
  header: {
    backgroundColor: '#3d3d3d',
    padding: '0 16px',
    height: 48,
    lineHeight: '48px',
  },
  headerContent: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    height: '100%',
  },
  headerLeft: {
    display: 'flex',
    alignItems: 'center',
    gap: 12,
  },
  fileName: {
    color: '#fff',
    fontSize: 14,
    maxWidth: 200,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  tools: {
    display: 'flex',
    alignItems: 'center',
    gap: 4,
  },
  headerRight: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
  },
  content: {
    display: 'flex',
    height: 'calc(100vh - 48px)',
  },
  pdfContainer: {
    flex: 1,
    overflow: 'hidden',
  },
  sider: {
    backgroundColor: '#fff',
    overflow: 'auto',
  },
  emptyState: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    color: '#999',
    fontSize: 16,
  },
};

export default PdfReader;
