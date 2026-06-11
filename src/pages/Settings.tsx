//! AI 设置页面
//!
//! 提供 AI 厂商配置、模型选择、API Key 管理功能
//! 提供云同步配置功能
//! 提供通用设置（主题、语言等）

import React, { useState, useEffect } from 'react';
import { Card, Form, Select, Input, Button, Switch, Space, message, Alert, Divider, Typography } from 'antd';
const { Text } = Typography;
import { KeyOutlined, ApiOutlined, ExperimentOutlined, SaveOutlined, CloudOutlined, SyncOutlined, SettingOutlined, InfoCircleOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import ThemeSwitch from '../components/ThemeSwitch';

// AI 厂商类型
type AIProviderType = 'openai' | 'anthropic' | 'deepseek' | 'doubao' | 'qwen' | 'glm' | 'minimax' | 'mimo';

// 模型信息
interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  max_tokens: number;
  supports_streaming: boolean;
}

// AI 配置
interface AIConfig {
  enabled: boolean;
  provider: AIProviderType;
  model_id: string;
  api_key: string;
  base_url?: string;
}

// 厂商显示名称映射
const providerDisplayNames: Record<AIProviderType, string> = {
  openai: 'OpenAI',
  anthropic: 'Anthropic',
  deepseek: 'DeepSeek',
  doubao: '豆包',
  qwen: '通义千问',
  glm: '智谱GLM',
  minimax: 'MiniMax',
  mimo: '小米MiMo',
};

interface SettingsProps {
  isDark?: boolean;
  onToggleTheme?: () => void;
}

const Settings: React.FC<SettingsProps> = ({ isDark = false, onToggleTheme }) => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [testing, setTesting] = useState(false);
  const [aiEnabled, setAiEnabled] = useState(false);
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [filteredModels, setFilteredModels] = useState<ModelInfo[]>([]);
  const [selectedProvider, setSelectedProvider] = useState<AIProviderType>('minimax');
  const [selectedModel, setSelectedModel] = useState<string>('MiniMax-M2.7');
  const [apiKey, setApiKey] = useState('');
  const [isConfigured, setIsConfigured] = useState(false);

  // 云同步相关状态
  const [syncEnabled, setSyncEnabled] = useState(false);
  const [syncServerUrl, setSyncServerUrl] = useState('http://121.41.223.84:4000');
  const [syncToken, setSyncToken] = useState('');
  const [syncMode, setSyncMode] = useState<'incremental' | 'full'>('incremental');
  const [syncing, setSyncing] = useState(false);
  const [syncStatus, setSyncStatus] = useState<any>(null);

  // 语言设置
  const [language, setLanguage] = useState<string>('zh-CN');

  // 加载 AI 配置
  useEffect(() => {
    loadAIConfig();
    loadAllModels();
    loadSyncConfig();
    loadLanguageSetting();
  }, []);

  // 根据选择的厂商过滤模型
  useEffect(() => {
    const filtered = models.filter(m => m.provider === selectedProvider);
    setFilteredModels(filtered);
    if (filtered.length > 0 && !filtered.some(m => m.id === selectedModel)) {
      setSelectedModel(filtered[0].id);
    }
  }, [selectedProvider, models]);

  // 加载语言设置
  const loadLanguageSetting = () => {
    try {
      const storedLang = localStorage.getItem('zoplus-language');
      if (storedLang) {
        setLanguage(storedLang);
      }
    } catch (e) {
      console.warn('无法读取语言设置:', e);
    }
  };

  // 保存语言设置
  const handleLanguageChange = (value: string) => {
    setLanguage(value);
    try {
      localStorage.setItem('zoplus-language', value);
      message.success('语言设置已保存');
    } catch (e) {
      console.warn('无法保存语言设置:', e);
    }
  };

  const loadAIConfig = async () => {
    try {
      const config = await invoke<AIConfig>('get_ai_config');
      setAiEnabled(config.enabled);
      setSelectedProvider(config.provider);
      setSelectedModel(config.model_id);
      setApiKey(config.api_key);
      setIsConfigured(await invoke<boolean>('is_ai_configured'));
      form.setFieldsValue({
        provider: config.provider,
        model: config.model_id,
        apiKey: config.api_key,
        baseUrl: config.base_url,
      });
    } catch (error) {
      console.error('加载 AI 配置失败:', error);
    }
  };

  const loadAllModels = async () => {
    try {
      const allModels = await invoke<ModelInfo[]>('get_all_ai_models');
      setModels(allModels);
    } catch (error) {
      console.error('加载模型列表失败:', error);
    }
  };

  // 加载云同步配置
  const loadSyncConfig = async () => {
    try {
      const config = await invoke<any>('get_sync_config');
      setSyncEnabled(config.enabled);
      setSyncServerUrl(config.server_url);
      setSyncToken(config.auth_token);
      setSyncMode(config.sync_mode);
      const status = await invoke<any>('get_sync_status');
      setSyncStatus(status);
    } catch (error) {
      console.error('加载云同步配置失败:', error);
    }
  };

  // 处理云同步开关
  const toggleSync = async (checked: boolean) => {
    setSyncEnabled(checked);
    await saveSyncConfig();
  };

  // 保存云同步配置
  const saveSyncConfig = async () => {
    try {
      const config = {
        enabled: syncEnabled,
        server_url: syncServerUrl,
        auth_token: syncToken,
        sync_mode: syncMode,
        conflict_strategy: 'local_first',
        last_sync_time: null,
        device_id: '',
      };
      await invoke('configure_sync', { config });
      message.success('云同步配置已保存');
    } catch (error) {
      message.error('保存云同步配置失败: ' + error);
    }
  };

  // 立即同步
  const handleSyncNow = async () => {
    setSyncing(true);
    try {
      const result = await invoke<any>('sync_now');
      if (result.success) {
        message.success(`同步完成：上传 ${result.uploaded} 条，下载 ${result.downloaded} 条`);
      } else {
        message.error('同步失败: ' + (result.error_message || '未知错误'));
      }
      await loadSyncConfig();
    } catch (error) {
      message.error('同步失败: ' + error);
    } finally {
      setSyncing(false);
    }
  };

  const handleProviderChange = (value: AIProviderType) => {
    setSelectedProvider(value);
    //厂商变更时，清空 API Key（不同厂商需要不同的 Key）
    setApiKey('');
    form.setFieldsValue({ apiKey: '' });
  };

  const handleSaveConfig = async () => {
    setLoading(true);
    try {
      const values = form.getFieldsValue();
      const config: AIConfig = {
        enabled: aiEnabled,
        provider: selectedProvider,
        model_id: selectedModel,
        api_key: values.apiKey || apiKey,
        base_url: values.baseUrl,
      };

      await invoke('update_ai_config', { config });
      await invoke('set_ai_enabled', { enabled: aiEnabled });
      message.success('AI 配置已保存');
      setIsConfigured(!!values.apiKey || !!apiKey);
    } catch (error) {
      message.error('保存配置失败: ' + error);
    } finally {
      setLoading(false);
    }
  };

  const handleTestConnection = async () => {
    setTesting(true);
    try {
      // 先保存配置
      await handleSaveConfig();
      // 测试连接
      const result = await invoke<boolean>('test_ai_connection');
      if (result) {
        message.success('连接成功！AI 功能正常工作');
      } else {
        message.warning('连接失败，请检查 API Key 是否正确');
      }
    } catch (error) {
      message.error('连接测试失败: ' + error);
    } finally {
      setTesting(false);
    }
  };

  return (
    <div style={{ padding: '24px', maxWidth: '800px', margin: '0 auto' }}>
      {/* 通用设置 */}
      <Card
        title={
          <Space>
            <SettingOutlined />
            通用设置
          </Space>
        }
        style={{ marginBottom: '24px' }}
      >
        <Form layout="vertical">
          <Divider orientation={'left' as any}>界面设置</Divider>

          <Form.Item label="主题">
            <Space>
              <ThemeSwitch isDark={isDark} onToggle={onToggleTheme || (() => {})} />
              <span style={{ color: 'rgba(0, 0, 0, 0.65)' }}>
                {isDark ? '当前：暗色主题' : '当前：亮色主题'}
              </span>
            </Space>
          </Form.Item>

          <Form.Item label="语言">
            <Select
              value={language}
              onChange={handleLanguageChange}
              style={{ width: 200 }}
            >
              <Select.Option value="zh-CN">简体中文</Select.Option>
              <Select.Option value="en-US">English</Select.Option>
            </Select>
          </Form.Item>
        </Form>
      </Card>

      {/* AI 设置 */}
      <Card
        title={
          <Space>
            <ApiOutlined />
            AI 设置
          </Space>
        }
        extra={
          <Switch
            checked={aiEnabled}
            onChange={setAiEnabled}
            checkedChildren="启用"
            unCheckedChildren="禁用"
          />
        }
      >
        {!isConfigured && aiEnabled && (
          <Alert
            message="AI 功能未配置"
            description="请配置 API Key 后才能使用 AI 功能"
            type="warning"
            showIcon
            style={{ marginBottom: '16px' }}
          />
        )}

        <Form form={form} layout="vertical" initialValues={{}}>
          <Divider orientation={'left' as any}>基本设置</Divider>

          <Form.Item label="AI 厂商" name="provider">
            <Select
              value={selectedProvider}
              onChange={handleProviderChange}
              placeholder="选择 AI 厂商"
            >
              {Object.entries(providerDisplayNames).map(([value, label]) => (
                <Select.Option key={value} value={value}>
                  {label}
                </Select.Option>
              ))}
            </Select>
          </Form.Item>

          <Form.Item label="模型" name="model">
            <Select
              value={selectedModel}
              onChange={setSelectedModel}
              placeholder="选择模型"
            >
              {filteredModels.map(model => (
                <Select.Option key={model.id} value={model.id}>
                  {model.name} {model.supports_streaming && '(流式)'}
                </Select.Option>
              ))}
            </Select>
          </Form.Item>

          <Divider orientation={'left' as any}>API 配置</Divider>

          <Form.Item
            label="API Key"
            name="apiKey"
            rules={[{ required: true, message: '请输入 API Key' }]}
          >
            <Input.Password
              prefix={<KeyOutlined />}
              placeholder="输入 API Key"
              value={apiKey}
              onChange={e => setApiKey(e.target.value)}
            />
          </Form.Item>

          <Form.Item label="自定义 API 地址（可选）" name="baseUrl">
            <Input
              placeholder="如需使用代理，请输入自定义 API 地址"
            />
          </Form.Item>

          <Divider />

          <Space>
            <Button
              type="primary"
              icon={<SaveOutlined />}
              onClick={handleSaveConfig}
              loading={loading}
            >
              保存配置
            </Button>
            <Button
              icon={<ExperimentOutlined />}
              onClick={handleTestConnection}
              loading={testing}
            >
              测试连接
            </Button>
          </Space>
        </Form>
      </Card>

      {/* 云同步设置 */}
      <Card
        title={
          <Space>
            <CloudOutlined />
            云同步设置
          </Space>
        }
        extra={
          <Switch
            checked={syncEnabled}
            onChange={toggleSync}
            checkedChildren="启用"
            unCheckedChildren="禁用"
          />
        }
        style={{ marginTop: '24px' }}
      >
        <Form layout="vertical">
          <Divider orientation={'left' as any}>服务器配置</Divider>

          <Form.Item label="服务器地址">
            <Input
              value={syncServerUrl}
              onChange={e => setSyncServerUrl(e.target.value)}
              placeholder="http://121.41.223.84:4000"
              disabled={!syncEnabled}
            />
          </Form.Item>

          <Form.Item label="同步令牌">
            <Input.Password
              value={syncToken}
              onChange={e => setSyncToken(e.target.value)}
              placeholder="输入同步认证令牌"
              disabled={!syncEnabled}
            />
          </Form.Item>

          <Divider orientation={'left' as any}>同步选项</Divider>

          <Form.Item label="同步模式">
            <Select
              value={syncMode}
              onChange={setSyncMode}
              disabled={!syncEnabled}
            >
              <Select.Option value="incremental">增量同步（推荐）</Select.Option>
              <Select.Option value="full">全量同步</Select.Option>
            </Select>
          </Form.Item>

          <Divider />

          <Space>
            <Button
              type="primary"
              icon={<SyncOutlined />}
              onClick={handleSyncNow}
              loading={syncing}
              disabled={!syncEnabled}
            >
              立即同步
            </Button>
            <Button
              icon={<SaveOutlined />}
              onClick={saveSyncConfig}
              disabled={!syncEnabled}
            >
              保存配置
            </Button>
          </Space>

          {syncStatus?.last_result && (
            <div style={{ marginTop: '16px' }}>
              <Alert
                message={`上次同步：${syncStatus.last_result.success ? '成功' : '失败'}`}
                description={syncStatus.last_result.success
                  ? `上传 ${syncStatus.last_result.uploaded} 条，下载 ${syncStatus.last_result.downloaded} 条`
                  : syncStatus.last_result.error_message}
                type={syncStatus.last_result.success ? 'success' : 'error'}
                showIcon
              />
            </div>
          )}
        </Form>
      </Card>

      {/* 关于 */}
      <Card
        title={
          <Space>
            <InfoCircleOutlined />
            关于
          </Space>
        }
        style={{ marginTop: '24px' }}
      >
        <Space direction="vertical">
          <div>
            <strong>ZoPlus</strong> v0.1.0
          </div>
          <Text type="secondary">
            基于 Tauri + React + Rust 构建的论文管理软件
          </Text>
          <Text type="secondary">
            集成 MiniMax AI 与阿里云云同步功能
          </Text>
        </Space>
      </Card>
    </div>
  );
};

export default Settings;