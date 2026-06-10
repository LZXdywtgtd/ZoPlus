//! 摘要结果显示组件
//!
//! 以结构化方式展示文献摘要内容，支持导出和重新生成

import React from 'react';
import {
  Modal,
  Typography,
  Space,
  Tag,
  Divider,
  Button,
  Tooltip,
  List,
} from 'antd';
import {
  ReloadOutlined,
  ExportOutlined,
  CloseOutlined,
  ClockCircleOutlined,
  CheckCircleOutlined,
} from '@ant-design/icons';
import type { ArticleSummary } from '../utils/tauriCommands';

const { Title, Text, Paragraph } = Typography;

interface SummaryResultProps {
  /** 摘要数据 */
  summary: ArticleSummary;
  /** 关闭回调 */
  onClose: () => void;
  /** 重新生成回调 */
  onRegenerate?: () => void;
  /** 导出回调 */
  onExport?: () => void;
}

/**
 * 摘要结果显示组件
 *
 * 展示结构化摘要内容：
 * - 核心问题
 * - 研究方法
 * - 关键结论
 * - 创新点
 * - 局限性
 * - 关键词
 * - 用户标注重点
 */
const SummaryResult: React.FC<SummaryResultProps> = ({
  summary,
  onClose,
  onRegenerate,
  onExport,
}) => {
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

  // 渲染摘要章节
  const renderSection = (title: string, content: string) => {
    if (!content) return null;
    return (
      <div style={styles.section}>
        <Title level={5} style={styles.sectionTitle}>
          {title}
        </Title>
        <Paragraph style={styles.sectionContent}>{content}</Paragraph>
      </div>
    );
  };

  return (
    <Modal
      open
      title={null}
      footer={null}
      onCancel={onClose}
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
              {summary.title}
            </Title>
            <Space split={<span style={styles.split}>|</span>}>
              <Text type="secondary">{summary.authors || '未知作者'}</Text>
              <Text type="secondary">{summary.year || '未知年份'}</Text>
            </Space>
            <div style={styles.citationBox}>
              <Text type="secondary" style={styles.citationLabel}>
                引用格式：
              </Text>
              <Text>{summary.citation}</Text>
            </div>
          </div>
          <div style={styles.headerActions}>
            <Tooltip title="重新生成">
              <Button
                icon={<ReloadOutlined />}
                onClick={onRegenerate}
                disabled={!onRegenerate}
              />
            </Tooltip>
            <Tooltip title="导出为 Markdown">
              <Button
                icon={<ExportOutlined />}
                onClick={onExport}
                disabled={!onExport}
              />
            </Tooltip>
            <Tooltip title="关闭">
              <Button icon={<CloseOutlined />} onClick={onClose} />
            </Tooltip>
          </div>
        </div>

        <Divider style={styles.divider} />

        {/* 关键词 */}
        {summary.keywords && summary.keywords.length > 0 && (
          <div style={styles.keywords}>
            <Text strong style={styles.keywordsLabel}>
              关键词：
            </Text>
            {summary.keywords.map((kw, i) => (
              <Tag key={i} color="blue">
                {kw}
              </Tag>
            ))}
          </div>
        )}

        {/* 摘要内容 */}
        <div style={styles.content}>
          {renderSection('核心问题', summary.core_problem)}
          {renderSection('研究方法', summary.research_methods)}
          {renderSection('关键结论', summary.key_conclusions)}
          {renderSection('创新点', summary.innovation)}
          {renderSection('局限性', summary.limitations)}
        </div>

        {/* 用户标注重点 */}
        {summary.highlighted_content && summary.highlighted_content.length > 0 && (
          <>
            <Divider style={styles.divider} />
            <div style={styles.section}>
              <Title level={5} style={styles.sectionTitle}>
                用户标注重点
              </Title>
              <List
                size="small"
                dataSource={summary.highlighted_content}
                renderItem={(item) => (
                  <List.Item style={styles.listItem}>
                    <Space>
                      <CheckCircleOutlined style={styles.listIcon} />
                      <Text>{item}</Text>
                    </Space>
                  </List.Item>
                )}
              />
            </div>
          </>
        )}

        {/* 底部信息 */}
        <div style={styles.footer}>
          <Space>
            <ClockCircleOutlined />
            <Text type="secondary" style={styles.footerText}>
              生成时间: {formatDate(summary.generated_at)}
            </Text>
          </Space>
          <Text type="secondary" style={styles.footerText}>
            版本: {summary.version}
          </Text>
        </div>
      </div>
    </Modal>
  );
};

// 样式
const styles: { [key: string]: React.CSSProperties } = {
  container: {
    maxHeight: '70vh',
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
    wordBreak: 'break-word',
  },
  split: {
    color: '#d9d9d9',
    margin: '0 8px',
  },
  citationBox: {
    marginTop: 8,
    padding: '8px 12px',
    backgroundColor: '#fff',
    borderRadius: 4,
  },
  citationLabel: {
    marginRight: 8,
  },
  divider: {
    margin: '12px 0',
  },
  keywords: {
    padding: '0 24px 16px',
    display: 'flex',
    alignItems: 'center',
    flexWrap: 'wrap',
    gap: 8,
  },
  keywordsLabel: {
    marginRight: 8,
  },
  content: {
    padding: '0 24px',
  },
  section: {
    marginBottom: 16,
  },
  sectionTitle: {
    color: '#1890ff',
    marginBottom: 8,
  },
  sectionContent: {
    color: '#333',
    lineHeight: 1.8,
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-word',
  },
  listItem: {
    padding: '8px 0',
  },
  listIcon: {
    color: '#52c41a',
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
};

export default SummaryResult;