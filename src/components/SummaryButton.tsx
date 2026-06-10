//! 摘要生成按钮组件
//!
//! 提供文献摘要生成功能，支持查看已有摘要和重新生成

import React, { useState, useEffect } from 'react';
import { Button, Tooltip, message, Modal } from 'antd';
import {
  FileTextOutlined,
  LoadingOutlined,
  CheckCircleOutlined,
} from '@ant-design/icons';
import useAppStore from '../store/appStore';
import {
  getArticleSummary,
  hasCachedSummary,
  getCachedSummary,
  exportSummaryAsMarkdown,
  type ArticleSummary,
} from '../utils/tauriCommands';
import SummaryResult from './SummaryResult';

interface SummaryButtonProps {
  /** 文献ID */
  itemId: number;
  /** PDF密钥（用于提取用户标注） */
  pdfKey?: string;
  /** 是否显示为完整按钮（带下拉菜单） */
  showDropdown?: boolean;
}

/**
 * 摘要生成按钮组件
 *
 * 提供以下功能：
 * - 生成新的文献摘要
 * - 查看已缓存的摘要
 * - 导出摘要为 Markdown 格式
 * - 重新生成摘要
 */
const SummaryButton: React.FC<SummaryButtonProps> = ({
  itemId,
  pdfKey,
  showDropdown = false,
}) => {
  // 本地状态
  const [isLoading, setIsLoading] = useState(false);
  const [hasCached, setHasCached] = useState(false);
  const [cachedSummary, setCachedSummary] = useState<ArticleSummary | null>(null);
  const [showResult, setShowResult] = useState(false);

  // 全局状态
  const { summaryStatus, setSummaryGenerating, setSummaryError, setCurrentSummary } = useAppStore();

  // 检查是否有缓存的摘要
  useEffect(() => {
    checkCache();
  }, [itemId]);

  // 检查缓存
  const checkCache = async () => {
    try {
      const cached = await hasCachedSummary(itemId);
      setHasCached(cached);

      if (cached) {
        const summary = await getCachedSummary(itemId);
        setCachedSummary(summary);
      }
    } catch (error) {
      console.error('[摘要按钮] 检查缓存失败:', error);
    }
  };

  // 生成摘要
  const handleGenerate = async () => {
    if (isLoading || summaryStatus.isGenerating) {
      return;
    }

    setIsLoading(true);
    setSummaryGenerating(true, itemId);
    setSummaryError(null);

    try {
      console.log('[摘要按钮] 开始生成摘要: item_id=', itemId);
      const summary = await getArticleSummary(itemId, pdfKey);
      setCachedSummary(summary);
      setHasCached(true);
      setCurrentSummary(summary);
      setShowResult(true);
      message.success('摘要生成成功');
    } catch (error) {
      console.error('[摘要按钮] 生成摘要失败:', error);
      const errorMsg = error instanceof Error ? error.message : String(error);
      setSummaryError(errorMsg);
      message.error('摘要生成失败: ' + errorMsg);
    } finally {
      setIsLoading(false);
      setSummaryGenerating(false, null);
    }
  };

  // 查看摘要
  const handleViewSummary = async () => {
    if (cachedSummary) {
      setCurrentSummary(cachedSummary);
      setShowResult(true);
    } else {
      // 尝试重新获取缓存
      try {
        const summary = await getCachedSummary(itemId);
        if (summary) {
          setCachedSummary(summary);
          setCurrentSummary(summary);
          setShowResult(true);
        } else {
          message.info('暂无缓存的摘要，请先生成');
        }
      } catch (error) {
        message.error('获取摘要失败');
      }
    }
  };

  // 导出 Markdown
  const handleExportMarkdown = async () => {
    try {
      const markdown = await exportSummaryAsMarkdown(itemId);

      // 创建下载
      const blob = new Blob([markdown], { type: 'text/markdown;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `摘要_${itemId}_${Date.now()}.md`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);

      message.success('导出成功');
    } catch (error) {
      message.error('导出失败: ' + (error instanceof Error ? error.message : String(error)));
    }
  };

  // 重新生成
  const handleRegenerate = () => {
    Modal.confirm({
      title: '确认重新生成',
      content: '确定要重新生成摘要吗？之前的摘要将被覆盖。',
      okText: '确定',
      cancelText: '取消',
      onOk: () => {
        setCachedSummary(null);
        setHasCached(false);
        handleGenerate();
      },
    });
  };

  // 判断按钮状态
  const isGeneratingThis = summaryStatus.isGenerating && summaryStatus.currentItemId === itemId;
  const isLoadingThis = isLoading && summaryStatus.currentItemId === itemId;

  // 图标
  const getIcon = () => {
    if (isLoadingThis || isGeneratingThis) {
      return <LoadingOutlined />;
    }
    if (hasCached) {
      return <CheckCircleOutlined />;
    }
    return <FileTextOutlined />;
  };

  // 提示文本
  const getTooltip = () => {
    if (isLoadingThis || isGeneratingThis) {
      return '正在生成摘要...';
    }
    if (hasCached) {
      return '查看/重新生成摘要';
    }
    return '生成摘要';
  };

  // 按钮文本
  const getButtonText = () => {
    if (isLoadingThis || isGeneratingThis) {
      return '生成中...';
    }
    return '摘要';
  };

  // 完整按钮（带下拉菜单）
  if (showDropdown) {
    return (
      <>
        <Button
          icon={getIcon()}
          onClick={hasCached ? handleViewSummary : handleGenerate}
          disabled={isLoadingThis || isGeneratingThis}
          type={hasCached ? 'default' : 'primary'}
        >
          {getButtonText()}
        </Button>

        {showResult && cachedSummary && (
          <SummaryResult
            summary={cachedSummary}
            onClose={() => setShowResult(false)}
            onRegenerate={handleRegenerate}
            onExport={handleExportMarkdown}
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
          onClick={hasCached ? handleViewSummary : handleGenerate}
          disabled={isLoadingThis || isGeneratingThis}
          size="small"
        />
      </Tooltip>

      {showResult && cachedSummary && (
        <SummaryResult
          summary={cachedSummary}
          onClose={() => setShowResult(false)}
          onRegenerate={handleRegenerate}
          onExport={handleExportMarkdown}
        />
      )}
    </>
  );
};

export default SummaryButton;