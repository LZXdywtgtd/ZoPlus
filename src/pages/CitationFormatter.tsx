//! 参考文献格式化工具页面
//!
//! 提供参考文献的解析、格式化和批量处理功能

import { useState, useEffect } from 'react';
import {
  Card,
  Input,
  Select,
  Button,
  Space,
  Typography,
  message,
  Alert,
  List,
  Tag,
  Modal,
  Form,
  InputNumber,
  Divider,
  Tooltip,
} from 'antd';
import {
  CopyOutlined,
  ClearOutlined,
  SwapOutlined,
  SaveOutlined,
  PlusOutlined,
  DeleteOutlined,
  EditOutlined,
} from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';

const { Title, Text } = Typography;
const { TextArea } = Input;
const { Option } = Select;

/// 作者信息接口
interface Author {
  last_name: string;
  first_name: string;
  middle_name?: string;
  prefix?: string;
  suffix?: string;
  author_type: string;
}

/// 文献元数据接口
interface CitationMetadata {
  authors: Author[];
  title: string;
  journal?: string;
  conference?: string;
  book_title?: string;
  year?: string;
  volume?: string;
  issue?: string;
  pages?: string;
  doi?: string;
  isbn?: string;
  issn?: string;
  url?: string;
  publisher?: string;
  location?: string;
  item_type: string;
  access_date?: string;
  degree?: string;
  school?: string;
  patent_number?: string;
  editors: Author[];
  translators: Author[];
  edition?: string;
  city?: string;
  status?: string;
}

/// 解析结果接口
interface ParsedCitation {
  original: string;
  metadata: CitationMetadata;
  warnings: string[];
  success: boolean;
}

/// 格式化结果接口
interface FormattedCitation {
  formatted: string;
  format: string;
  metadata: CitationMetadata;
  warnings: string[];
}

/// 引用格式信息接口
interface CitationFormatInfo {
  id: string;
  name: string;
}

/// 文献类型选项
const ITEM_TYPES = [
  { value: 'journal_article', label: '期刊文章' },
  { value: 'conference_paper', label: '会议论文' },
  { value: 'book', label: '书籍' },
  { value: 'book_chapter', label: '书籍章节' },
  { value: 'thesis', label: '学位论文' },
  { value: 'report', label: '报告' },
  { value: 'web_page', label: '网页' },
  { value: 'patent', label: '专利' },
  { value: 'newspaper_article', label: '报纸文章' },
];

/// 参考文献格式化工具页面组件
function CitationFormatter() {
  // 输入文本
  const [inputText, setInputText] = useState<string>('');
  // 解析结果
  const [parsedResult, setParsedResult] = useState<ParsedCitation | null>(null);
  // 格式化结果
  const [formattedResult, setFormattedResult] = useState<FormattedCitation | null>(null);
  // 当前格式
  const [selectedFormat, setSelectedFormat] = useState<string>('apa7');
  // 可用格式列表
  const [formats, setFormats] = useState<CitationFormatInfo[]>([]);
  // 批量处理模式
  const [batchMode, setBatchMode] = useState<boolean>(false);
  // 批量结果列表
  const [batchResults, setBatchResults] = useState<FormattedCitation[]>([]);
  // 加载状态
  const [loading, setLoading] = useState<boolean>(false);
  // 编辑元数据弹窗
  const [editModalVisible, setEditModalVisible] = useState<boolean>(false);
  // 编辑表单
  const [editForm] = Form.useForm();
  // 当前编辑的元数据
  const [editingMetadata, setEditingMetadata] = useState<CitationMetadata | null>(null);

  // 加载可用格式
  useEffect(() => {
    loadFormats();
  }, []);

  /// 加载可用引用格式
  const loadFormats = async () => {
    try {
      const formatList = await invoke<CitationFormatInfo[]>('get_citation_formats');
      setFormats(formatList);
    } catch (error) {
      console.error('加载引用格式失败:', error);
      message.error('加载引用格式失败');
    }
  };

  /// 解析参考文献
  const handleParse = async () => {
    if (!inputText.trim()) {
      message.warning('请输入参考文献文本');
      return;
    }

    setLoading(true);
    try {
      const result = await invoke<ParsedCitation>('parse_citation_text', {
        input: inputText,
      });
      setParsedResult(result);

      if (result.warnings.length > 0) {
        result.warnings.forEach((warning) => {
          message.warning(warning);
        });
      }
    } catch (error) {
      console.error('解析参考文献失败:', error);
      message.error(`解析失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  /// 格式化参考文献
  const handleFormat = async () => {
    if (!parsedResult && !editingMetadata) {
      message.warning('请先解析参考文献或手动输入元数据');
      return;
    }

    setLoading(true);
    try {
      const metadata = editingMetadata || parsedResult?.metadata;
      if (!metadata) {
        message.error('没有可用的元数据');
        return;
      }

      const result = await invoke<FormattedCitation>('format_citation', {
        metadata,
        format: selectedFormat,
      });
      setFormattedResult(result);

      if (result.warnings.length > 0) {
        result.warnings.forEach((warning) => {
          message.warning(warning);
        });
      } else {
        message.success('格式化成功');
      }
    } catch (error) {
      console.error('格式化失败:', error);
      message.error(`格式化失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  /// 批量格式化
  const handleBatchFormat = async () => {
    const lines = inputText.split('\n').filter((line) => line.trim());
    if (lines.length === 0) {
      message.warning('请输入参考文献文本（每行一条）');
      return;
    }

    setLoading(true);
    setBatchMode(true);
    setBatchResults([]);

    try {
      // 逐条解析并格式化
      const results: FormattedCitation[] = [];
      for (const line of lines) {
        try {
          const parsed = await invoke<ParsedCitation>('parse_citation_text', {
            input: line,
          });
          const formatted = await invoke<FormattedCitation>('format_citation', {
            metadata: parsed.metadata,
            format: selectedFormat,
          });
          results.push(formatted);
        } catch (error) {
          console.error(`处理行失败: ${line.slice(0, 50)}...`, error);
          // 继续处理下一条
        }
      }

      setBatchResults(results);
      message.success(`批量处理完成: ${results.length}/${lines.length} 条`);
    } catch (error) {
      console.error('批量格式化失败:', error);
      message.error(`批量处理失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  /// 复制结果到剪贴板
  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text).then(
      () => {
        message.success('已复制到剪贴板');
      },
      () => {
        message.error('复制失败');
      }
    );
  };

  /// 清空所有内容
  const handleClear = () => {
    setInputText('');
    setParsedResult(null);
    setFormattedResult(null);
    setBatchResults([]);
    setBatchMode(false);
    setEditingMetadata(null);
    message.info('已清空');
  };

  /// 打开编辑弹窗
  const handleEditMetadata = () => {
    const metadata = editingMetadata || parsedResult?.metadata;
    if (!metadata) {
      message.warning('没有可编辑的元数据');
      return;
    }
    setEditingMetadata(metadata);
    editForm.setFieldsValue({
      ...metadata,
      authors: metadata.authors.map((a) => ({
        ...a,
        name: `${a.last_name}, ${a.first_name}`.trim(),
      })),
    });
    setEditModalVisible(true);
  };

  /// 保存编辑的元数据
  const handleSaveMetadata = async () => {
    try {
      const values = await editForm.validateFields();
      const metadata: CitationMetadata = {
        ...values,
        authors: values.authors.map((a: any) => {
          const parts = (a.name || '').split(',').map((s: string) => s.trim());
          return {
            last_name: parts[0] || '',
            first_name: parts[1] || '',
            middle_name: parts[2] || undefined,
            prefix: undefined,
            suffix: undefined,
            author_type: 'primary',
          };
        }),
      };
      setEditingMetadata(metadata);
      setEditModalVisible(false);
      message.success('元数据已更新');
    } catch (error) {
      console.error('表单验证失败:', error);
    }
  };

  /// 从 Zotero 补全元数据
  const handleEnrichFromZotero = async (itemId: number) => {
    const metadata = editingMetadata || parsedResult?.metadata;
    if (!metadata) {
      message.warning('没有可补全的元数据');
      return;
    }

    setLoading(true);
    try {
      const enriched = await invoke<CitationMetadata>('enrich_citation_metadata', {
        itemId,
        metadata,
      });
      setEditingMetadata(enriched);
      message.success('元数据已从 Zotero 补全');
    } catch (error) {
      console.error('补全元数据失败:', error);
      message.error(`补全失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Space direction="vertical" style={{ width: '100%' }} size="large">
      <Card>
        <Title level={4}>参考文献格式化工具</Title>
        <Text type="secondary">
          输入不规范的参考文献，选择目标格式，一键转换为标准引用格式。支持批量处理多条引用。
        </Text>
      </Card>

      {/* 输入区域 */}
      <Card title="输入参考文献">
        <TextArea
          value={inputText}
          onChange={(e) => setInputText(e.target.value)}
          placeholder="输入或粘贴参考文献文本，每行一条（批量模式）..."
          rows={6}
          style={{ marginBottom: 16 }}
        />

        <Space wrap>
          <Button
            type="primary"
            icon={<SwapOutlined />}
            onClick={handleParse}
            loading={loading}
          >
            解析
          </Button>
          <Button icon={<SwapOutlined />} onClick={handleBatchFormat} loading={loading}>
            批量解析
          </Button>
          <Button icon={<ClearOutlined />} onClick={handleClear}>
            清空
          </Button>
        </Space>
      </Card>

      {/* 格式选择 */}
      <Card title="选择目标格式">
        <Space wrap>
          <Select
            value={selectedFormat}
            onChange={setSelectedFormat}
            style={{ width: 200 }}
          >
            {formats.map((format) => (
              <Option key={format.id} value={format.id}>
                {format.name}
              </Option>
            ))}
          </Select>
          <Button
            type="primary"
            icon={<SwapOutlined />}
            onClick={handleFormat}
            loading={loading}
            disabled={!parsedResult && !editingMetadata}
          >
            格式化
          </Button>
          <Button
            icon={<EditOutlined />}
            onClick={handleEditMetadata}
            disabled={!parsedResult && !editingMetadata}
          >
            编辑元数据
          </Button>
        </Space>
      </Card>

      {/* 解析结果 */}
      {parsedResult && !batchMode && (
        <Card title="解析结果" extra={<Tag color={parsedResult.success ? 'green' : 'orange'}>
          {parsedResult.success ? '解析成功' : '部分解析'}
        </Tag>}>
          {parsedResult.warnings.length > 0 && (
            <Alert
              message="解析警告"
              description={
                <ul>
                  {parsedResult.warnings.map((w, i) => (
                    <li key={i}>{w}</li>
                  ))}
                </ul>
              }
              type="warning"
              style={{ marginBottom: 16 }}
            />
          )}

          <List
            size="small"
            bordered
            dataSource={[
              { label: '标题', value: parsedResult.metadata.title || '-' },
              { label: '作者', value: parsedResult.metadata.authors.map((a) => `${a.last_name}, ${a.first_name}`).join('; ') || '-' },
              { label: '年份', value: parsedResult.metadata.year || '-' },
              { label: '期刊', value: parsedResult.metadata.journal || '-' },
              { label: 'DOI', value: parsedResult.metadata.doi || '-' },
              { label: '类型', value: parsedResult.metadata.item_type || '-' },
            ]}
            renderItem={(item) => (
              <List.Item>
                <Text strong style={{ width: 80 }}>{item.label}:</Text>
                <Text>{item.value}</Text>
              </List.Item>
            )}
          />
        </Card>
      )}

      {/* 格式化结果 */}
      {(formattedResult || batchResults.length > 0) && (
        <Card title={batchMode ? '批量格式化结果' : '格式化结果'}>
          {batchMode ? (
            <List
              size="small"
              bordered
              dataSource={batchResults}
              renderItem={(item, index) => (
                <List.Item
                  actions={[
                    <Tooltip title="复制">
                      <Button
                        type="text"
                        icon={<CopyOutlined />}
                        onClick={() => handleCopy(item.formatted)}
                      />
                    </Tooltip>,
                  ]}
                >
                  <Text>{item.formatted}</Text>
                </List.Item>
              )}
            />
          ) : formattedResult && (
            <>
              {formattedResult.warnings.length > 0 && (
                <Alert
                  message="格式化警告"
                  description={
                    <ul>
                      {formattedResult.warnings.map((w, i) => (
                        <li key={i}>{w}</li>
                      ))}
                    </ul>
                  }
                  type="warning"
                  style={{ marginBottom: 16 }}
                />
              )}

              <Card
                size="small"
                style={{
                  backgroundColor: '#f5f5f5',
                  marginBottom: 16,
                  fontFamily: 'Times New Roman, serif',
                  fontSize: 14,
                  lineHeight: 1.8,
                }}
              >
                <div dangerouslySetInnerHTML={{ __html: formattedResult.formatted.replace(/\*/g, '') }} />
              </Card>

              <Space>
                <Button
                  type="primary"
                  icon={<CopyOutlined />}
                  onClick={() => handleCopy(formattedResult.formatted)}
                >
                  复制结果
                </Button>
              </Space>
            </>
          )}
        </Card>
      )}

      {/* 编辑元数据弹窗 */}
      <Modal
        title="编辑元数据"
        open={editModalVisible}
        onOk={handleSaveMetadata}
        onCancel={() => setEditModalVisible(false)}
        width={700}
        okText="保存"
        cancelText="取消"
      >
        <Form
          form={editForm}
          layout="vertical"
          initialValues={{
            authors: [],
            editors: [],
            translators: [],
          }}
        >
          <Divider>作者信息</Divider>
          <Form.List name="authors">
            {(fields, { add, remove }) => (
              <>
                {fields.map(({ key, name, ...restField }) => (
                  <Space key={key} style={{ display: 'flex', marginBottom: 8 }} align="baseline">
                    <Form.Item
                      {...restField}
                      name={[name, 'name']}
                      rules={[{ required: true, message: '请输入作者姓名' }]}
                    >
                      <Input placeholder="姓, 名" style={{ width: 200 }} />
                    </Form.Item>
                    <Button type="text" danger icon={<DeleteOutlined />} onClick={() => remove(name)}>
                      删除
                    </Button>
                  </Space>
                ))}
                <Button type="dashed" onClick={() => add({ name: '' })} block icon={<PlusOutlined />}>
                  添加作者
                </Button>
              </>
            )}
          </Form.List>

          <Divider>文献信息</Divider>
          <Space wrap>
            <Form.Item label="标题" name="title" style={{ width: 300 }}>
              <Input />
            </Form.Item>
            <Form.Item label="年份" name="year" style={{ width: 120 }}>
              <Input />
            </Form.Item>
            <Form.Item label="期刊" name="journal" style={{ width: 200 }}>
              <Input />
            </Form.Item>
          </Space>

          <Space wrap>
            <Form.Item label="卷号" name="volume" style={{ width: 100 }}>
              <Input />
            </Form.Item>
            <Form.Item label="期号" name="issue" style={{ width: 100 }}>
              <Input />
            </Form.Item>
            <Form.Item label="页码" name="pages" style={{ width: 120 }}>
              <Input />
            </Form.Item>
            <Form.Item label="DOI" name="doi" style={{ width: 200 }}>
              <Input />
            </Form.Item>
          </Space>

          <Space wrap>
            <Form.Item label="出版商" name="publisher" style={{ width: 200 }}>
              <Input />
            </Form.Item>
            <Form.Item label="出版地" name="location" style={{ width: 150 }}>
              <Input />
            </Form.Item>
            <Form.Item label="ISBN" name="isbn" style={{ width: 150 }}>
              <Input />
            </Form.Item>
          </Space>

          <Space wrap>
            <Form.Item label="URL" name="url" style={{ width: 300 }}>
              <Input />
            </Form.Item>
            <Form.Item label="文献类型" name="item_type" style={{ width: 150 }}>
              <Select>
                {ITEM_TYPES.map((type) => (
                  <Option key={type.value} value={type.value}>
                    {type.label}
                  </Option>
                ))}
              </Select>
            </Form.Item>
          </Space>
        </Form>
      </Modal>
    </Space>
  );
}

export default CitationFormatter;