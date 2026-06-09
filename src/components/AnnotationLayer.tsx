//! PDF 标注绘制层组件
//!
//! 本组件通过 Canvas API 在 PDF 渲染层之上叠加标注绘制层，
//! 支持高亮、矩形、椭圆、箭头、自由绘制、文本笔记等标注类型

import React, { useEffect, useRef, useState, useCallback } from 'react';

// ============== 类型定义 ==============

/** 标注类型枚举 */
export type AnnotationType =
  | 'highlight'
  | 'rectangle'
  | 'ellipse'
  | 'arrow'
  | 'free_draw'
  | 'text_note';

/** 标注颜色 */
export interface AnnotationColor {
  r: number;
  g: number;
  b: number;
  a: number;
}

/** PDF 坐标点 */
export interface PdfPoint {
  x: number;
  y: number;
}

/** PDF 矩形区域 */
export interface PdfRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** 高亮标注数据 */
export interface HighlightData {
  rect: PdfRect;
  text: string;
}

/** 矩形标注数据 */
export interface RectangleData {
  rect: PdfRect;
  stroke_width: number;
}

/** 椭圆标注数据 */
export interface EllipseData {
  rect: PdfRect;
  stroke_width: number;
}

/** 箭头标注数据 */
export interface ArrowData {
  start: PdfPoint;
  end: PdfPoint;
  stroke_width: number;
}

/** 自由绘制标注数据 */
export interface FreeDrawData {
  points: PdfPoint[];
  stroke_width: number;
}

/** 文本笔记标注数据 */
export interface TextNoteData {
  position: PdfPoint;
  content: string;
  icon_size: number;
}

/** 标注数据联合类型 */
export type AnnotationData =
  | HighlightData
  | RectangleData
  | EllipseData
  | ArrowData
  | FreeDrawData
  | TextNoteData;

/** 单个标注结构 */
export interface Annotation {
  id: string;
  annotation_type: AnnotationType;
  color: AnnotationColor;
  page: number;
  data: AnnotationData;
  created_at: number;
  updated_at: number;
}

/** 默认高亮颜色（黄色） */
export const DEFAULT_HIGHLIGHT_COLOR: AnnotationColor = {
  r: 255,
  g: 255,
  b: 0,
  a: 128,
};

/** 默认矩形颜色（红色） */
export const DEFAULT_RECTANGLE_COLOR: AnnotationColor = {
  r: 255,
  g: 0,
  b: 0,
  a: 180,
};

/** 默认椭圆颜色（蓝色） */
export const DEFAULT_ELLIPSE_COLOR: AnnotationColor = {
  r: 0,
  g: 0,
  b: 255,
  a: 180,
};

/** 默认箭头颜色（绿色） */
export const DEFAULT_ARROW_COLOR: AnnotationColor = {
  r: 0,
  g: 255,
  b: 0,
  a: 200,
};

/** 默认自由绘制颜色（黑色） */
export const DEFAULT_FREE_DRAW_COLOR: AnnotationColor = {
  r: 0,
  g: 0,
  b: 0,
  a: 255,
};

/** 默认文本笔记颜色（橙色） */
export const DEFAULT_TEXT_NOTE_COLOR: AnnotationColor = {
  r: 255,
  g: 165,
  b: 0,
  a: 200,
};

/** 标注模式 */
export type AnnotationMode = AnnotationType | 'select';

/** 组件属性 */
interface AnnotationLayerProps {
  /** PDF 页码 */
  page: number;
  /** 当前页面的标注列表 */
  annotations: Annotation[];
  /** 当前标注模式 */
  mode: AnnotationMode;
  /** 当前标注颜色 */
  color: AnnotationColor;
  /** 是否启用标注功能 */
  enabled: boolean;
  /** 视口信息（用于坐标转换） */
  viewport: {
    scale: number;
    rotation: number;
    width: number;
    height: number;
  } | null;
  /** 创建标注回调 */
  onAnnotationCreate?: (annotation: Annotation) => void;
  /** 更新标注回调 */
  onAnnotationUpdate?: (annotation: Annotation) => void;
  /** 删除标注回调 */
  onAnnotationDelete?: (annotationId: string) => void;
  /** 点击标注回调 */
  onAnnotationClick?: (annotation: Annotation) => void;
  /** 选中标注回调 */
  onAnnotationSelect?: (annotation: Annotation | null) => void;
  /** 跳转回调（跳转到指定页面位置） */
  onNavigateTo?: (page: number, position: PdfPoint) => void;
}

interface AnnotationLayerState {
  /** 是否正在绘制 */
  isDrawing: boolean;
  /** 当前绘制类型 */
  drawingType: AnnotationType | null;
  /** 绘制起点 */
  startPoint: PdfPoint | null;
  /** 当前绘制点 */
  currentPoint: PdfPoint | null;
  /** 自由绘制路径 */
  freeDrawPoints: PdfPoint[];
  /** 选中的标注 */
  selectedAnnotation: Annotation | null;
  /** 文本输入框位置 */
  textInputPosition: PdfPoint | null;
  /** 文本输入内容 */
  textInputValue: string;
}

// ============== 工具函数 ==============

/**
 * 将页面坐标转换为屏幕坐标
 */
export function pageToScreen(
  point: PdfPoint,
  viewport: { scale: number; rotation: number; width: number; height: number }
): { x: number; y: number } {
  return {
    x: point.x * viewport.scale,
    y: (viewport.height - point.y) * viewport.scale, // PDF Y轴向上，屏幕Y轴向下
  };
}

/**
 * 将屏幕坐标转换为页面坐标
 */
export function screenToPage(
  point: { x: number; y: number },
  viewport: { scale: number; rotation: number; width: number; height: number }
): PdfPoint {
  return {
    x: point.x / viewport.scale,
    y: (viewport.height - point.y / viewport.scale),
  };
}

/**
 * 将颜色对象转换为 CSS 字符串
 */
export function colorToCss(color: AnnotationColor): string {
  return `rgba(${color.r}, ${color.g}, ${color.b}, ${color.a / 255})`;
}

/**
 * 生成唯一 ID
 */
export function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * 获取默认颜色
 */
export function getDefaultColor(type: AnnotationType): AnnotationColor {
  switch (type) {
    case 'highlight':
      return DEFAULT_HIGHLIGHT_COLOR;
    case 'rectangle':
      return DEFAULT_RECTANGLE_COLOR;
    case 'ellipse':
      return DEFAULT_ELLIPSE_COLOR;
    case 'arrow':
      return DEFAULT_ARROW_COLOR;
    case 'free_draw':
      return DEFAULT_FREE_DRAW_COLOR;
    case 'text_note':
      return DEFAULT_TEXT_NOTE_COLOR;
    default:
      return DEFAULT_HIGHLIGHT_COLOR;
  }
}

// ============== 组件实现 ==============

/**
 * PDF 标注绘制层组件
 * 在 PDF 渲染层之上叠加 Canvas 绘制层，支持各种标注类型
 */
const AnnotationLayer: React.FC<AnnotationLayerProps> = ({
  page,
  annotations,
  mode,
  color,
  enabled,
  viewport,
  onAnnotationCreate,
  onAnnotationUpdate: _onAnnotationUpdate,
  onAnnotationDelete: _onAnnotationDelete,
  onAnnotationClick,
  onAnnotationSelect,
  onNavigateTo: _onNavigateTo,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [state, setState] = useState<AnnotationLayerState>({
    isDrawing: false,
    drawingType: null,
    startPoint: null,
    currentPoint: null,
    freeDrawPoints: [],
    selectedAnnotation: null,
    textInputPosition: null,
    textInputValue: '',
  });

  /**
   * 绘制所有标注
   */
  const drawAnnotations = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas || !viewport) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // 清空画布
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // 绘制所有标注
    for (const annotation of annotations) {
      if (annotation.page !== page) continue;
      drawAnnotation(ctx, annotation, annotation === state.selectedAnnotation, canvas.width, canvas.height);
    }

    // 绘制正在创建的标注
    if (state.isDrawing && state.drawingType) {
      drawDrawingPreview(ctx);
    }
  }, [annotations, page, viewport, state]);

  /**
   * 绘制单个标注
   */
  const drawAnnotation = (
    ctx: CanvasRenderingContext2D,
    annotation: Annotation,
    isSelected: boolean,
    canvasWidth: number,
    canvasHeight: number
  ) => {
    ctx.save();

    const cssColor = colorToCss(annotation.color);
    ctx.strokeStyle = cssColor;
    ctx.fillStyle = cssColor;
    ctx.lineWidth = 2;

    switch (annotation.annotation_type) {
      case 'highlight': {
        const data = annotation.data as HighlightData;
        ctx.fillStyle = cssColor;
        ctx.fillRect(
          data.rect.x * (viewport?.scale || 1),
          (viewport?.height || 0) - (data.rect.y + data.rect.height) * (viewport?.scale || 1),
          data.rect.width * (viewport?.scale || 1),
          data.rect.height * (viewport?.scale || 1)
        );
        break;
      }
      case 'rectangle': {
        const data = annotation.data as RectangleData;
        const x = data.rect.x * (viewport?.scale || 1);
        const y = (viewport?.height || 0) - (data.rect.y + data.rect.height) * (viewport?.scale || 1);
        const w = data.rect.width * (viewport?.scale || 1);
        const h = data.rect.height * (viewport?.scale || 1);
        ctx.strokeRect(x, y, w, h);
        break;
      }
      case 'ellipse': {
        const data = annotation.data as EllipseData;
        const cx = (data.rect.x + data.rect.width / 2) * (viewport?.scale || 1);
        const cy = (viewport?.height || 0) - (data.rect.y + data.rect.height / 2) * (viewport?.scale || 1);
        const rx = (data.rect.width / 2) * (viewport?.scale || 1);
        const ry = (data.rect.height / 2) * (viewport?.scale || 1);
        ctx.beginPath();
        ctx.ellipse(cx, cy, rx, ry, 0, 0, Math.PI * 2);
        ctx.stroke();
        break;
      }
      case 'arrow': {
        const data = annotation.data as ArrowData;
        const startX = data.start.x * (viewport?.scale || 1);
        const startY = (viewport?.height || 0) - data.start.y * (viewport?.scale || 1);
        const endX = data.end.x * (viewport?.scale || 1);
        const endY = (viewport?.height || 0) - data.end.y * (viewport?.scale || 1);

        // 绘制线条
        ctx.beginPath();
        ctx.moveTo(startX, startY);
        ctx.lineTo(endX, endY);
        ctx.stroke();

        // 绘制箭头
        const angle = Math.atan2(endY - startY, endX - startX);
        const arrowLength = 15;
        ctx.beginPath();
        ctx.moveTo(endX, endY);
        ctx.lineTo(
          endX - arrowLength * Math.cos(angle - Math.PI / 6),
          endY - arrowLength * Math.sin(angle - Math.PI / 6)
        );
        ctx.moveTo(endX, endY);
        ctx.lineTo(
          endX - arrowLength * Math.cos(angle + Math.PI / 6),
          endY - arrowLength * Math.sin(angle + Math.PI / 6)
        );
        ctx.stroke();
        break;
      }
      case 'free_draw': {
        const data = annotation.data as FreeDrawData;
        if (data.points.length < 2) break;
        ctx.beginPath();
        const firstPoint = data.points[0];
        ctx.moveTo(
          firstPoint.x * (viewport?.scale || 1),
          (viewport?.height || 0) - firstPoint.y * (viewport?.scale || 1)
        );
        for (let i = 1; i < data.points.length; i++) {
          const pt = data.points[i];
          ctx.lineTo(
            pt.x * (viewport?.scale || 1),
            (viewport?.height || 0) - pt.y * (viewport?.scale || 1)
          );
        }
        ctx.stroke();
        break;
      }
      case 'text_note': {
        const data = annotation.data as TextNoteData;
        const x = data.position.x * (viewport?.scale || 1);
        const y = (viewport?.height || 0) - data.position.y * (viewport?.scale || 1);
        const size = data.icon_size * (viewport?.scale || 1);

        // 绘制笔记图标（简化的方块图标）
        ctx.fillStyle = cssColor;
        ctx.fillRect(x - size / 2, y - size / 2, size, size);
        ctx.strokeStyle = '#333';
        ctx.strokeRect(x - size / 2, y - size / 2, size, size);

        // 绘制折角效果
        ctx.beginPath();
        ctx.moveTo(x + size / 2 - size / 4, y - size / 2);
        ctx.lineTo(x + size / 2, y - size / 2 + size / 4);
        ctx.lineTo(x + size / 2, y - size / 2);
        ctx.closePath();
        ctx.fillStyle = '#666';
        ctx.fill();
        break;
      }
    }

    // 选中效果
    if (isSelected) {
      ctx.strokeStyle = 'rgba(0, 120, 215, 0.8)';
      ctx.lineWidth = 2;
      ctx.setLineDash([5, 5]);
      ctx.strokeRect(-2, -2, canvasWidth + 4, canvasHeight + 4);
    }

    ctx.restore();
  };

  /**
   * 绘制正在创建的标注预览
   */
  const drawDrawingPreview = (ctx: CanvasRenderingContext2D) => {
    if (!state.startPoint || !state.currentPoint || !viewport) return;

    ctx.save();
    ctx.strokeStyle = colorToCss(color);
    ctx.fillStyle = colorToCss(color);
    ctx.lineWidth = 2;
    ctx.setLineDash([5, 5]);

    const scale = viewport.scale;

    switch (state.drawingType) {
      case 'rectangle':
      case 'ellipse': {
        const x = state.startPoint.x * scale;
        const y = (viewport.height) - state.startPoint.y * scale;
        const w = (state.currentPoint.x - state.startPoint.x) * scale;
        const h = -(state.currentPoint.y - state.startPoint.y) * scale;

        if (state.drawingType === 'rectangle') {
          ctx.strokeRect(x, y, w, h);
        } else {
          const cx = x + w / 2;
          const cy = y + h / 2;
          const rx = Math.abs(w / 2);
          const ry = Math.abs(h / 2);
          ctx.beginPath();
          ctx.ellipse(cx, cy, rx, ry, 0, 0, Math.PI * 2);
          ctx.stroke();
        }
        break;
      }
      case 'arrow': {
        const startX = state.startPoint.x * scale;
        const startY = (viewport.height) - state.startPoint.y * scale;
        const endX = state.currentPoint.x * scale;
        const endY = (viewport.height) - state.currentPoint.y * scale;

        ctx.beginPath();
        ctx.moveTo(startX, startY);
        ctx.lineTo(endX, endY);
        ctx.stroke();
        break;
      }
      case 'free_draw': {
        if (state.freeDrawPoints.length < 2) break;
        ctx.setLineDash([]);
        ctx.beginPath();
        const firstPt = state.freeDrawPoints[0];
        ctx.moveTo(firstPt.x * scale, (viewport.height) - firstPt.y * scale);
        for (const pt of state.freeDrawPoints) {
          ctx.lineTo(pt.x * scale, (viewport.height) - pt.y * scale);
        }
        ctx.stroke();
        break;
      }
    }

    ctx.restore();
  };

  /**
   * 更新画布大小
   */
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !viewport) return;

    canvas.width = viewport.width;
    canvas.height = viewport.height;
    drawAnnotations();
  }, [viewport, drawAnnotations]);

  /**
   * 重新绘制标注
   */
  useEffect(() => {
    drawAnnotations();
  }, [drawAnnotations]);

  /**
   * 鼠标按下事件
   */
  const handleMouseDown = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!enabled || !viewport) return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const screenX = e.clientX - rect.left;
    const screenY = e.clientY - rect.top;
    const pagePoint = screenToPage({ x: screenX, y: screenY }, viewport);

    if (mode === 'select') {
      // 检查是否点击了某个标注
      const clickedAnnotation = findAnnotationAtPoint(pagePoint);
      setState((prev) => ({ ...prev, selectedAnnotation: clickedAnnotation }));
      if (onAnnotationSelect) {
        onAnnotationSelect(clickedAnnotation);
      }
      if (clickedAnnotation && onAnnotationClick) {
        onAnnotationClick(clickedAnnotation);
      }
    } else {
      // 开始绘制
      setState((prev) => ({
        ...prev,
        isDrawing: true,
        drawingType: mode as AnnotationType,
        startPoint: pagePoint,
        currentPoint: pagePoint,
        freeDrawPoints: [pagePoint],
      }));
    }
  };

  /**
   * 鼠标移动事件
   */
  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!state.isDrawing || !viewport) return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const screenX = e.clientX - rect.left;
    const screenY = e.clientY - rect.top;
    const pagePoint = screenToPage({ x: screenX, y: screenY }, viewport);

    if (state.drawingType === 'free_draw') {
      setState((prev) => ({
        ...prev,
        currentPoint: pagePoint,
        freeDrawPoints: [...prev.freeDrawPoints, pagePoint],
      }));
    } else {
      setState((prev) => ({
        ...prev,
        currentPoint: pagePoint,
      }));
    }
  };

  /**
   * 鼠标释放事件
   */
  const handleMouseUp = () => {
    if (!state.isDrawing || !state.startPoint || !state.currentPoint) return;

    // 创建标注
    if (state.drawingType && onAnnotationCreate) {
      const annotation = createAnnotationFromDrawing();
      if (annotation) {
        onAnnotationCreate(annotation);
      }
    }

    // 重置状态
    setState((prev) => ({
      ...prev,
      isDrawing: false,
      drawingType: null,
      startPoint: null,
      currentPoint: null,
      freeDrawPoints: [],
    }));
  };

  /**
   * 从当前绘制状态创建标注
   */
  const createAnnotationFromDrawing = (): Annotation | null => {
    if (!state.drawingType || !state.startPoint || !state.currentPoint) return null;

    const now = Date.now();
    let data: AnnotationData;

    switch (state.drawingType) {
      case 'highlight':
      case 'rectangle':
      case 'ellipse': {
        const minX = Math.min(state.startPoint.x, state.currentPoint.x);
        const minY = Math.min(state.startPoint.y, state.currentPoint.y);
        const width = Math.abs(state.currentPoint.x - state.startPoint.x);
        const height = Math.abs(state.currentPoint.y - state.startPoint.y);

        if (state.drawingType === 'highlight') {
          data = {
            rect: { x: minX, y: minY, width, height },
            text: '', // 高亮文本稍后补充
          } as HighlightData;
        } else if (state.drawingType === 'rectangle') {
          data = {
            rect: { x: minX, y: minY, width, height },
            stroke_width: 2,
          } as RectangleData;
        } else {
          data = {
            rect: { x: minX, y: minY, width, height },
            stroke_width: 2,
          } as EllipseData;
        }
        break;
      }
      case 'arrow': {
        data = {
          start: state.startPoint,
          end: state.currentPoint,
          stroke_width: 2,
        } as ArrowData;
        break;
      }
      case 'free_draw': {
        if (state.freeDrawPoints.length < 2) return null;
        data = {
          points: [...state.freeDrawPoints],
          stroke_width: 2,
        } as FreeDrawData;
        break;
      }
      default:
        return null;
    }

    return {
      id: generateId(),
      annotation_type: state.drawingType,
      color,
      page,
      data,
      created_at: now,
      updated_at: now,
    };
  };

  /**
   * 在指定点查找标注
   */
  const findAnnotationAtPoint = (point: PdfPoint): Annotation | null => {
    // 简单实现：检查点击位置是否在标注范围内
    for (let i = annotations.length - 1; i >= 0; i--) {
      const annotation = annotations[i];
      if (annotation.page !== page) continue;

      if (isPointInAnnotation(point, annotation)) {
        return annotation;
      }
    }
    return null;
  };

  /**
   * 检查点是否在标注范围内
   */
  const isPointInAnnotation = (point: PdfPoint, annotation: Annotation): boolean => {
    const tolerance = 10; // 容差范围

    switch (annotation.annotation_type) {
      case 'highlight':
      case 'rectangle':
      case 'ellipse': {
        const data = annotation.data as { rect: PdfRect };
        return (
          point.x >= data.rect.x - tolerance &&
          point.x <= data.rect.x + data.rect.width + tolerance &&
          point.y >= data.rect.y - tolerance &&
          point.y <= data.rect.y + data.rect.height + tolerance
        );
      }
      case 'arrow': {
        const data = annotation.data as ArrowData;
        // 检查点是否在直线附近
        const dist = pointToLineDistance(point, data.start, data.end);
        return dist <= tolerance;
      }
      case 'free_draw': {
        const data = annotation.data as FreeDrawData;
        // 检查点是否在路径附近
        for (const pt of data.points) {
          const dist = Math.sqrt(Math.pow(point.x - pt.x, 2) + Math.pow(point.y - pt.y, 2));
          if (dist <= tolerance) return true;
        }
        return false;
      }
      case 'text_note': {
        const data = annotation.data as TextNoteData;
        return (
          point.x >= data.position.x - data.icon_size / 2 &&
          point.x <= data.position.x + data.icon_size / 2 &&
          point.y >= data.position.y - data.icon_size / 2 &&
          point.y <= data.position.y + data.icon_size / 2
        );
      }
      default:
        return false;
    }
  };

  /**
   * 计算点到直线的距离
   */
  const pointToLineDistance = (
    point: PdfPoint,
    lineStart: PdfPoint,
    lineEnd: PdfPoint
  ): number => {
    const A = point.x - lineStart.x;
    const B = point.y - lineStart.y;
    const C = lineEnd.x - lineStart.x;
    const D = lineEnd.y - lineStart.y;

    const dot = A * C + B * D;
    const lenSq = C * C + D * D;
    let param = -1;

    if (lenSq !== 0) {
      param = dot / lenSq;
    }

    let xx, yy;

    if (param < 0) {
      xx = lineStart.x;
      yy = lineStart.y;
    } else if (param > 1) {
      xx = lineEnd.x;
      yy = lineEnd.y;
    } else {
      xx = lineStart.x + param * C;
      yy = lineStart.y + param * D;
    }

    const dx = point.x - xx;
    const dy = point.y - yy;
    return Math.sqrt(dx * dx + dy * dy);
  };

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        pointerEvents: enabled ? 'auto' : 'none',
        cursor: enabled ? (mode === 'select' ? 'default' : 'crosshair') : 'default',
      }}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
    />
  );
};

export default AnnotationLayer;
