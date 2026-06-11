//! 聊天消息组件
//!
//! 展示单条聊天消息，支持用户消息和助手消息两种样式

import React from 'react';
import { Avatar, Typography, Tooltip } from 'antd';
import { UserOutlined, RobotOutlined } from '@ant-design/icons';
import type { DocumentContext } from '../utils/tauriCommands';

const { Text, Paragraph } = Typography;

interface ChatMessageProps {
  /** 消息角色：user / assistant */
  role: 'user' | 'assistant';
  /** 消息内容 */
  content: string;
  /** 引用的文献上下文（仅助手消息有） */
  citations?: DocumentContext[];
  /** 时间戳 */
  timestamp?: number;
}

/**
 * 聊天消息组件
 *
 * 展示单条聊天消息：
 * - 用户消息：右侧显示，蓝色背景
 * - 助手消息：左侧显示，白色背景，支持显示引用文献
 */
const ChatMessage: React.FC<ChatMessageProps> = ({
  role,
  content,
  citations = [],
  timestamp,
}) => {
  const isUser = role === 'user';

  // 格式化时间
  const formatTime = (ts: number) => {
    const date = new Date(ts);
    return date.toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  // 渲染引用文献
  const renderCitations = () => {
    if (citations.length === 0) return null;

    return (
      <div style={styles.citationsContainer}>
        <Text type="secondary" style={styles.citationsLabel}>
          参考文献：
        </Text>
        <div style={styles.citationsList}>
          {citations.map((citation, index) => (
            <Tooltip
              key={citation.item_id}
              title={
                <div style={styles.citationTooltip}>
                  <div style={styles.citationTitle}>{citation.title}</div>
                  <div style={styles.citationAbstract}>
                    {citation.abstract_text || '无摘要'}
                  </div>
                </div>
              }
            >
              <div style={styles.citationItem}>
                <span style={styles.citationIndex}>{index + 1}.</span>
                <span style={styles.citationKey}>{citation.citation_key}</span>
                <span style={styles.citationTitle} className="citation-title">
                  {citation.title.length > 30
                    ? citation.title.slice(0, 30) + '...'
                    : citation.title}
                </span>
              </div>
            </Tooltip>
          ))}
        </div>
      </div>
    );
  };

  return (
    <div style={isUser ? styles.userContainer : styles.assistantContainer}>
      {/* 头像 */}
      <Avatar
        icon={isUser ? <UserOutlined /> : <RobotOutlined />}
        style={{
          ...styles.avatar,
          backgroundColor: isUser ? '#1890ff' : '#52c41a',
        }}
      />

      {/* 消息内容 */}
      <div style={styles.messageContent}>
        {/* 用户/助手标签 */}
        <Text type="secondary" style={styles.roleLabel}>
          {isUser ? '我' : 'AI助手'}
        </Text>

        {/* 消息气泡 */}
        <div
          style={{
            ...styles.bubble,
            backgroundColor: isUser ? '#e6f7ff' : '#f5f5f5',
            borderRadius: isUser ? '16px 16px 4px 16px' : '16px 16px 16px 4px',
          }}
        >
          <Paragraph
            style={styles.messageText}
            ellipsis={{
              rows: 0,
              expandable: true,
              symbol: '展开',
            }}
          >
            {content}
          </Paragraph>
        </div>

        {/* 引用文献 */}
        {!isUser && renderCitations()}

        {/* 时间戳 */}
        {timestamp && (
          <Text type="secondary" style={styles.timestamp}>
            {formatTime(timestamp)}
          </Text>
        )}
      </div>
    </div>
  );
};

// 样式
const styles: { [key: string]: React.CSSProperties } = {
  userContainer: {
    display: 'flex',
    justifyContent: 'flex-end',
    alignItems: 'flex-start',
    marginBottom: 16,
    gap: 8,
  },
  assistantContainer: {
    display: 'flex',
    justifyContent: 'flex-start',
    alignItems: 'flex-start',
    marginBottom: 16,
    gap: 8,
  },
  avatar: {
    flexShrink: 0,
  },
  messageContent: {
    maxWidth: '70%',
    display: 'flex',
    flexDirection: 'column',
    gap: 4,
  },
  roleLabel: {
    fontSize: 12,
    marginLeft: 12,
  },
  bubble: {
    padding: '12px 16px',
    boxShadow: '0 1px 2px rgba(0, 0, 0, 0.1)',
  },
  messageText: {
    margin: 0,
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-word',
    lineHeight: 1.6,
  },
  citationsContainer: {
    marginTop: 8,
    padding: '8px 12px',
    backgroundColor: '#fafafa',
    borderRadius: 8,
    border: '1px solid #e8e8e8',
  },
  citationsLabel: {
    fontSize: 12,
    marginBottom: 4,
    display: 'block',
  },
  citationsList: {
    display: 'flex',
    flexDirection: 'column',
    gap: 4,
  },
  citationItem: {
    fontSize: 12,
    display: 'flex',
    alignItems: 'baseline',
    gap: 4,
    cursor: 'pointer',
    padding: '2px 4px',
    borderRadius: 4,
    transition: 'background-color 0.2s',
  },
  citationIndex: {
    color: '#999',
    marginRight: 4,
  },
  citationKey: {
    color: '#1890ff',
    fontWeight: 500,
    marginRight: 4,
  },
  citationTitle: {
    color: '#666',
    fontWeight: 500,
    marginBottom: 4,
  },
  citationTooltip: {
    maxWidth: 300,
  },
  citationAbstract: {
    fontSize: 12,
    color: '#999',
    lineHeight: 1.4,
  },
  timestamp: {
    fontSize: 11,
    marginLeft: 12,
  },
};

export default ChatMessage;