//! AI 跨文献智能问答页面
//!
//! 基于 RAG 的跨文献问答聊天界面

import React, { useState, useRef, useEffect } from 'react';
import {
  Layout,
  Typography,
  Input,
  Button,
  Space,
  Card,
  Slider,
  Switch,
  Tooltip,
  Spin,
  Alert,
  Divider,
  Empty,
  message,
} from 'antd';
import {
  SendOutlined,
  ClearOutlined,
  SettingOutlined,
  StopOutlined,
  BookOutlined,
} from '@ant-design/icons';
import ChatMessage from '../components/ChatMessage';
import type { ChatMessage as ChatMessageType, RagConfig } from '../utils/tauriCommands';

const { Title, Text } = Typography;
const { TextArea } = Input;

// 默认配置
const DEFAULT_CONFIG: RagConfig = {
  top_k: 5,
  streaming: true,
  min_score: 0.0,
};

// 欢迎消息
const WELCOME_MESSAGE = `欢迎使用跨文献智能问答！

我可以帮助您：
- 对比多篇文献中的研究方法和结论
- 总结特定主题的相关研究进展
- 分析不同文献中的观点差异
- 解答关于您文献库中的学术问题

请在下方输入您的问题，我会检索相关文献并给出带有引用的回答。`;

interface AIChatProps {}

/**
 * AI 跨文献智能问答页面
 *
 * 功能：
 * - 基于 Tantivy 检索相关文献
 * - 调用 AI 生成带引用的回答
 * - 支持流式输出
 * - 支持调整检索文献数量
 * - 支持清除聊天历史
 */
const AIChat: React.FC<AIChatProps> = () => {
  // 聊天消息列表
  const [messages, setMessages] = useState<ChatMessageType[]>([]);
  // 输入内容
  const [inputValue, setInputValue] = useState('');
  // 加载状态
  const [loading, setLoading] = useState(false);
  // 是否正在生成
  const [generating, setGenerating] = useState(false);
  // 配置
  const [config, setConfig] = useState<RagConfig>(DEFAULT_CONFIG);
  // 是否显示设置面板
  const [showSettings, setShowSettings] = useState(false);
  // 错误信息
  const [error, setError] = useState<string | null>(null);

  // 聊天区域引用
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // 自动滚动到底部
  useEffect(() => {
    if (messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [messages]);

  // 加载配置
  useEffect(() => {
    loadConfig();
  }, []);

  // 加载 RAG 配置
  const loadConfig = async () => {
    try {
      const { getRagConfig } = await import('../utils/tauriCommands');
      const cfg = await getRagConfig();
      setConfig(cfg);
    } catch (err) {
      console.error('加载配置失败:', err);
    }
  };

  // 发送消息
  const handleSend = async () => {
    if (!inputValue.trim() || generating) return;

    const userMessage = inputValue.trim();
    setInputValue('');
    setError(null);
    setGenerating(true);
    setLoading(true);

    try {
      const { aiChatStream, getChatHistory } = await import('../utils/tauriCommands');

      // 使用流式 API（发送消息并获取流式响应）
      await aiChatStream(userMessage);

      // 获取更新后的聊天历史
      const history = await getChatHistory();
      setMessages(history);

      message.success('回答已生成');
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '发送消息失败';
      setError(errorMsg);
      message.error(errorMsg);
    } finally {
      setLoading(false);
      setGenerating(false);
    }
  };

  // 清除聊天历史
  const handleClear = async () => {
    try {
      const { clearChatHistory } = await import('../utils/tauriCommands');
      await clearChatHistory();
      setMessages([]);
      message.success('聊天历史已清除');
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '清除失败';
      message.error(errorMsg);
    }
  };

  //停止生成
  const handleStop = () => {
    // 注意：当前实现不支持取消，正在开发中
    message.info('取消功能开发中...');
  };

  // 更新配置
  const handleConfigChange = async (key: keyof RagConfig, value: number | boolean) => {
    try {
      const { updateRagConfig } = await import('../utils/tauriCommands');
      const updateObj = { [key]: value };
      const newConfig = await updateRagConfig(updateObj);
      setConfig(newConfig);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '配置更新失败';
      message.error(errorMsg);
    }
  };

  //键盘事件处理
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <Layout style={styles.container}>
      {/* 头部 */}
      <div style={styles.header}>
        <Space>
          <BookOutlined style={styles.headerIcon} />
          <Title level={4} style={styles.headerTitle}>
            跨文献智能问答
          </Title>
        </Space>
        <Space>
          <Tooltip title={generating ? '停止生成' : '清除历史'}>
            <Button
              icon={generating ? <StopOutlined /> : <ClearOutlined />}
              onClick={generating ? handleStop : handleClear}
              danger={!generating}
              disabled={messages.length === 0 && !generating}
            />
          </Tooltip>
          <Tooltip title="设置">
            <Button
              icon={<SettingOutlined />}
              onClick={() => setShowSettings(!showSettings)}
              type={showSettings ? 'primary' : 'default'}
            />
          </Tooltip>
        </Space>
      </div>

      {/* 设置面板 */}
      {showSettings && (
        <Card size="small" style={styles.settingsCard}>
          <Space direction="vertical" style={{ width: '100%' }} size="middle">
            <div>
              <Text strong>检索文献数量：{config.top_k} 篇</Text>
              <Slider
                min={1}
                max={20}
                value={config.top_k}
                onChange={(value) => handleConfigChange('top_k', value)}
                style={{ width: 200 }}
              />
            </div>
            <div>
              <Space>
                <Text strong>流式输出</Text>
                <Switch
                  checked={config.streaming}
                  onChange={(checked) => handleConfigChange('streaming', checked)}
                />
              </Space>
            </div>
            <div>
              <Text strong>最小相关度：{config.min_score.toFixed(2)}</Text>
              <Slider
                min={0}
                max={100}
                value={config.min_score * 100}
                onChange={(value) => handleConfigChange('min_score', value / 100)}
                style={{ width: 200 }}
              />
            </div>
          </Space>
        </Card>
      )}

      {/* 错误提示 */}
      {error && (
        <Alert
          message="错误"
          description={error}
          type="error"
          showIcon
          closable
          onClose={() => setError(null)}
          style={styles.errorAlert}
        />
      )}

      {/* 聊天消息区域 */}
      <div style={styles.messagesContainer}>
        {messages.length === 0 ? (
          <Empty
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            description={
              <div style={styles.welcomeMessage}>
                <Text>{WELCOME_MESSAGE}</Text>
              </div>
            }
          />
        ) : (
          <div style={styles.messagesList}>
            {messages.map((msg) => (
              <ChatMessage
                key={msg.id}
                role={msg.role as 'user' | 'assistant'}
                content={msg.content}
                citations={msg.citations}
                timestamp={msg.timestamp}
              />
            ))}
            <div ref={messagesEndRef} />
          </div>
        )}

        {/* 加载指示器 */}
        {loading && (
          <div style={styles.loadingContainer}>
            <Spin tip="AI 正在思考..." />
          </div>
        )}
      </div>

      <Divider style={styles.divider} />

      {/* 输入区域 */}
      <div style={styles.inputContainer}>
        <TextArea
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          onKeyPress={handleKeyPress}
          placeholder="请输入您的问题，例如：这几篇文献中关于 Transformer 的优化方法有什么不同？"
          autoSize={{ minRows: 2, maxRows: 4 }}
          style={styles.input}
          disabled={generating}
        />
        <Button
          type="primary"
          icon={<SendOutlined />}
          onClick={handleSend}
          loading={generating}
          disabled={!inputValue.trim() || generating}
          style={styles.sendButton}
        >
          发送
        </Button>
      </div>
    </Layout>
  );
};

// 样式
const styles: { [key: string]: React.CSSProperties } = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: '#fff',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '12px 16px',
    borderBottom: '1px solid #f0f0f0',
    backgroundColor: '#fafafa',
  },
  headerIcon: {
    fontSize: 20,
    color: '#1890ff',
  },
  headerTitle: {
    margin: 0,
  },
  settingsCard: {
    margin: 8,
    backgroundColor: '#fafafa',
  },
  errorAlert: {
    margin: 8,
  },
  messagesContainer: {
    flex: 1,
    overflow: 'auto',
    padding: 16,
  },
  messagesList: {
    display: 'flex',
    flexDirection: 'column',
  },
  welcomeMessage: {
    textAlign: 'left',
    whiteSpace: 'pre-wrap',
    lineHeight: 1.8,
    maxWidth: 500,
  },
  loadingContainer: {
    display: 'flex',
    justifyContent: 'center',
    padding: 16,
  },
  divider: {
    margin: 0,
  },
  inputContainer: {
    display: 'flex',
    gap: 8,
    padding: 12,
    backgroundColor: '#fafafa',
    borderTop: '1px solid #f0f0f0',
  },
  input: {
    flex: 1,
  },
  sendButton: {
    height: 'auto',
    alignSelf: 'flex-end',
  },
};

export default AIChat;