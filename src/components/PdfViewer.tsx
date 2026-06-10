//! PDF 阅读器主组件
//!
//! 本组件集成了 pdf.js 用于 PDF 文档的加载和渲染，
//! 支持页码导航、缩放控制、旋转功能

import React, { useEffect, useRef, useState, useCallback } from 'react';
import * as pdfjsLib from 'pdfjs-dist';
import { Button, Space, InputNumber, Tooltip, Select, message } from 'antd';
import {
  LeftOutlined,
  RightOutlined,
  ZoomInOutlined,
  ZoomOutOutlined,
  RotateRightOutlined,
  RotateLeftOutlined,
  FullscreenOutlined,
} from '@ant-design/icons';
import type { PDFDocumentProxy, PDFPageProxy } from 'pdfjs-dist';
import SummaryButton from './SummaryButton';
import NoteGenerator from './NoteGenerator';

// 设置 PDF.js worker 文件路径
pdfjsLib.GlobalWorkerOptions.workerSrc = `/pdf.worker.min.mjs`;

// 缩放级别选项
const ZOOM_LEVELS = [
  { value: 0.5, label: '50%' },
  { value: 0.75, label: '75%' },
  { value: 1, label: '100%' },
  { value: 1.25, label: '125%' },
  { value: 1.5, label: '150%' },
  { value: 2, label: '200%' },
];

// 适应窗口模式
const FIT_SCALE_FACTOR = 1.2;

interface PdfViewerProps {
  /** PDF 文件路径（文件 URL 或本地路径） */
  filePath: string;
  /** 文件名（用于标注存储） */
  fileName: string;
  /** 当前页码（从 1 开始） */
  currentPage?: number;
  /** 缩放级别 */
  scale?: number;
  /** 旋转角度（0, 90, 180, 270） */
  rotation?: number;
  /** 页码变化回调 */
  onPageChange?: (page: number) => void;
  /** 缩放变化回调 */
  onScaleChange?: (scale: number) => void;
  /** 旋转变化回调 */
  onRotationChange?: (rotation: number) => void;
  /** 渲染完成回调 */
  onLoadComplete?: (totalPages: number) => void;
  /** 自定义渲染层 */
  renderLayer?: (page: PDFPageProxy, canvas: HTMLCanvasElement, viewport: any) => void;
  /** 子组件（标注层等） */
  children?: React.ReactNode;
  /** 文献ID（用于摘要功能） */
  itemId?: number;
  /** PDF密钥（用于摘要功能） */
  pdfKey?: string;
}

interface PdfViewerState {
  /** PDF 文档对象 */
  pdfDoc: PDFDocumentProxy | null;
  /** 当前页码 */
  currentPage: number;
  /** 总页数 */
  totalPages: number;
  /** 缩放级别 */
  scale: number;
  /** 旋转角度 */
  rotation: number;
  /** 是否加载中 */
  isLoading: boolean;
  /** 错误信息 */
  error: string | null;
  /** Canvas 引用 */
  canvas: HTMLCanvasElement | null;
}

/**
 * PDF 阅读器组件
 * 支持页码导航、缩放控制、旋转功能
 */
const PdfViewer: React.FC<PdfViewerProps> = ({
  filePath,
  fileName: _fileName,
  currentPage = 1,
  scale = 1,
  rotation = 0,
  onPageChange,
  onScaleChange,
  onRotationChange,
  onLoadComplete,
  renderLayer,
  children,
  itemId,
  pdfKey,
}) => {
  // 状态
  const [state, setState] = useState<PdfViewerState>({
    pdfDoc: null,
    currentPage: currentPage,
    totalPages: 0,
    scale: scale,
    rotation: rotation,
    isLoading: true,
    error: null,
    canvas: null,
  });

  //refs
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const renderTaskRef = useRef<any>(null);

  /**
   * 加载 PDF 文档
   */
  useEffect(() => {
    if (!filePath) return;

    const loadPdf = async () => {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      try {
        // 加载 PDF 文档
        const loadingTask = pdfjsLib.getDocument({
          url: filePath,
        });
        const pdf = await loadingTask.promise;

        setState((prev) => ({
          ...prev,
          pdfDoc: pdf,
          totalPages: pdf.numPages,
          currentPage: Math.min(Math.max(currentPage, 1), pdf.numPages),
          isLoading: false,
        }));

        // 触发加载完成回调
        if (onLoadComplete) {
          onLoadComplete(pdf.numPages);
        }
      } catch (error) {
        console.error('PDF 加载失败:', error);
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: `PDF 加载失败: ${error instanceof Error ? error.message : '未知错误'}`,
        }));
        message.error('PDF 加载失败');
      }
    };

    loadPdf();

    // 清理函数
    return () => {
      if (renderTaskRef.current) {
        renderTaskRef.current.cancel();
      }
    };
  }, [filePath]);

  /**
   * 渲染当前页面
   */
  const renderPage = useCallback(async () => {
    if (!state.pdfDoc || !canvasRef.current) return;

    // 取消之前的渲染任务
    if (renderTaskRef.current) {
      renderTaskRef.current.cancel();
    }

    try {
      const page = await state.pdfDoc.getPage(state.currentPage);

      // 计算视口
      const viewport = page.getViewport({
        scale: state.scale,
        rotation: state.rotation,
      });

      const canvas = canvasRef.current;
      const context = canvas.getContext('2d');

      if (!context) {
        throw new Error('无法获取 Canvas 上下文');
      }

      // 设置 Canvas 尺寸
      canvas.width = viewport.width;
      canvas.height = viewport.height;

      // 渲染 PDF 页面
      const renderTask = page.render({
        canvasContext: context,
        viewport: viewport,
        canvas: canvas,
      });

      renderTaskRef.current = renderTask;

      await renderTask.promise;

      // 如果有自定义渲染层，调用它
      if (renderLayer) {
        renderLayer(page, canvas, viewport);
      }

      setState((prev) => ({ ...prev, canvas }));
    } catch (error: any) {
      if (error?.name !== 'RenderingCancelledException') {
        console.error('页面渲染失败:', error);
      }
    }
  }, [state.pdfDoc, state.currentPage, state.scale, state.rotation, renderLayer]);

  /**
   * 页面或缩放变化时重新渲染
   */
  useEffect(() => {
    renderPage();
  }, [renderPage]);

  /**
   * 上一页
   */
  const goToPreviousPage = () => {
    if (state.currentPage > 1) {
      const newPage = state.currentPage - 1;
      setState((prev) => ({ ...prev, currentPage: newPage }));
      if (onPageChange) {
        onPageChange(newPage);
      }
    }
  };

  /**
   * 下一页
   */
  const goToNextPage = () => {
    if (state.currentPage < state.totalPages) {
      const newPage = state.currentPage + 1;
      setState((prev) => ({ ...prev, currentPage: newPage }));
      if (onPageChange) {
        onPageChange(newPage);
      }
    }
  };

  /**
   * 跳转到指定页
   */
  const goToPage = (page: number | null) => {
    if (page === null || page < 1 || page > state.totalPages) return;

    setState((prev) => ({ ...prev, currentPage: page }));
    if (onPageChange) {
      onPageChange(page);
    }
  };

  /**
   * 放大
   */
  const zoomIn = () => {
    const currentIndex = ZOOM_LEVELS.findIndex((z) => z.value === state.scale);
    if (currentIndex < ZOOM_LEVELS.length - 1) {
      const newScale = ZOOM_LEVELS[currentIndex + 1].value;
      setState((prev) => ({ ...prev, scale: newScale }));
      if (onScaleChange) {
        onScaleChange(newScale);
      }
    }
  };

  /**
   * 缩小
   */
  const zoomOut = () => {
    const currentIndex = ZOOM_LEVELS.findIndex((z) => z.value === state.scale);
    if (currentIndex > 0) {
      const newScale = ZOOM_LEVELS[currentIndex - 1].value;
      setState((prev) => ({ ...prev, scale: newScale }));
      if (onScaleChange) {
        onScaleChange(newScale);
      }
    }
  };

  /**
   * 适应窗口宽度
   */
  const fitToWindow = () => {
    if (!containerRef.current || !state.pdfDoc) return;

    // 计算适应窗口的缩放比例
    const containerWidth = containerRef.current.clientWidth - 40; // 减去边距
    state.pdfDoc.getPage(state.currentPage).then((page) => {
      const defaultViewport = page.getViewport({ scale: 1 });
      const scaleFactor = containerWidth / defaultViewport.width;
      const newScale = Math.min(scaleFactor, FIT_SCALE_FACTOR);

      setState((prev) => ({ ...prev, scale: newScale }));
      if (onScaleChange) {
        onScaleChange(newScale);
      }
    });
  };

  /**
   * 顺时针旋转
   */
  const rotateClockwise = () => {
    const newRotation = (state.rotation + 90) % 360;
    setState((prev) => ({ ...prev, rotation: newRotation }));
    if (onRotationChange) {
      onRotationChange(newRotation);
    }
  };

  /**
   * 逆时针旋转
   */
  const rotateCounterClockwise = () => {
    const newRotation = (state.rotation - 90 + 360) % 360;
    setState((prev) => ({ ...prev, rotation: newRotation }));
    if (onRotationChange) {
      onRotationChange(newRotation);
    }
  };

  /**
   * 缩放级别变化
   */
  const handleScaleChange = (value: number | null) => {
    if (value !== null) {
      setState((prev) => ({ ...prev, scale: value }));
      if (onScaleChange) {
        onScaleChange(value);
      }
    }
  };

  // 键盘事件处理
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'ArrowLeft':
        case 'PageUp':
          goToPreviousPage();
          break;
        case 'ArrowRight':
        case 'PageDown':
          goToNextPage();
          break;
        case 'Home':
          goToPage(1);
          break;
        case 'End':
          goToPage(state.totalPages);
          break;
        case '+':
        case '=':
          zoomIn();
          break;
        case '-':
          zoomOut();
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [state.currentPage, state.totalPages, state.scale]);

  return (
    <div className="pdf-viewer" style={styles.container}>
      {/* 工具栏 */}
      <div style={styles.toolbar}>
        <Space size="small" wrap>
          {/* 页码导航 */}
          <Tooltip title="上一页">
            <Button
              icon={<LeftOutlined />}
              onClick={goToPreviousPage}
              disabled={state.currentPage <= 1 || state.isLoading}
            />
          </Tooltip>

          <InputNumber
            min={1}
            max={state.totalPages || 1}
            value={state.currentPage}
            onChange={goToPage}
            style={{ width: 60 }}
            disabled={state.isLoading}
          />
          <span style={styles.pageInfo}>/ {state.totalPages}</span>

          <Tooltip title="下一页">
            <Button
              icon={<RightOutlined />}
              onClick={goToNextPage}
              disabled={state.currentPage >= state.totalPages || state.isLoading}
            />
          </Tooltip>

          <div style={styles.divider} />

          {/* 缩放控制 */}
          <Tooltip title="缩小">
            <Button
              icon={<ZoomOutOutlined />}
              onClick={zoomOut}
              disabled={state.isLoading}
            />
          </Tooltip>

          <Select
            value={state.scale}
            onChange={handleScaleChange}
            options={ZOOM_LEVELS}
            style={{ width: 80 }}
            disabled={state.isLoading}
          />

          <Tooltip title="放大">
            <Button
              icon={<ZoomInOutlined />}
              onClick={zoomIn}
              disabled={state.isLoading}
            />
          </Tooltip>

          <Tooltip title="适应窗口">
            <Button
              icon={<FullscreenOutlined />}
              onClick={fitToWindow}
              disabled={state.isLoading}
            />
          </Tooltip>

          <div style={styles.divider} />

          {/* 旋转控制 */}
          <Tooltip title="逆时针旋转">
            <Button
              icon={<RotateLeftOutlined />}
              onClick={rotateCounterClockwise}
              disabled={state.isLoading}
            />
          </Tooltip>

          <Tooltip title="顺时针旋转">
            <Button
              icon={<RotateRightOutlined />}
              onClick={rotateClockwise}
              disabled={state.isLoading}
            />
          </Tooltip>

          {itemId && (
            <>
              <div style={styles.divider} />
              <SummaryButton itemId={itemId} pdfKey={pdfKey} showDropdown />
              <NoteGenerator
                itemId={itemId}
                pdfKey={pdfKey}
                showDropdown
              />
            </>
          )}
        </Space>
      </div>

      {/* PDF 渲染区域 */}
      <div ref={containerRef} style={styles.viewerContainer}>
        {state.isLoading && (
          <div style={styles.loadingOverlay}>
            <span>加载中...</span>
          </div>
        )}

        {state.error && (
          <div style={styles.errorOverlay}>
            <span>{state.error}</span>
          </div>
        )}

        <div style={styles.canvasContainer}>
          <canvas ref={canvasRef} style={styles.canvas} />
          {/* 渲染子组件（标注层等） */}
          {children}
        </div>
      </div>
    </div>
  );
};

// 样式
const styles: { [key: string]: React.CSSProperties } = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: '#525659',
  },
  toolbar: {
    display: 'flex',
    alignItems: 'center',
    padding: '8px 12px',
    backgroundColor: '#3d3d3d',
    borderBottom: '1px solid #555',
  },
  pageInfo: {
    color: '#ccc',
    fontSize: 14,
  },
  divider: {
    width: 1,
    height: 24,
    backgroundColor: '#555',
    margin: '0 8px',
  },
  viewerContainer: {
    flex: 1,
    overflow: 'auto',
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'flex-start',
    padding: 20,
  },
  canvasContainer: {
    position: 'relative',
    boxShadow: '0 2px 10px rgba(0, 0, 0, 0.3)',
  },
  canvas: {
    display: 'block',
    backgroundColor: 'white',
  },
  loadingOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    color: 'white',
    fontSize: 16,
  },
  errorOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    color: '#ff6b6b',
    fontSize: 16,
  },
};

export default PdfViewer;
