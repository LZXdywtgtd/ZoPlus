//!搜索框组件
//!
//! 提供全文搜索的输入框和搜索按钮，支持模糊搜索开关。

import { Input, Button, Space, Switch, Tooltip, Typography } from 'antd';
import { SearchOutlined, ClearOutlined } from '@ant-design/icons';
import { useState } from 'react';

const { Text } = Typography;

export interface SearchBoxProps {
    /** 搜索回调函数 */
    onSearch: (query: string, fuzzy: boolean) => void;
    /** 是否显示模糊搜索开关 */
    showFuzzySwitch?: boolean;
    /** 是否显示清除按钮 */
    showClearButton?: boolean;
    /** 搜索按钮点击回调（用于清除索引等操作） */
    onClear?: () => void;
    /** 是否禁用输入框 */
    disabled?: boolean;
    /** 是否显示加载状态 */
    loading?: boolean;
    /** 占位符文本 */
    placeholder?: string;
}

const { Search } = Input;

/// 搜索框组件
function SearchBox({
    onSearch,
    showFuzzySwitch = true,
    showClearButton = false,
    onClear,
    disabled = false,
    loading = false,
    placeholder = '请输入搜索关键词...',
}: SearchBoxProps) {
    // 搜索关键词
    const [query, setQuery] = useState<string>('');
    // 是否启用模糊搜索
    const [fuzzyEnabled, setFuzzyEnabled] = useState<boolean>(false);

    // 处理搜索
    const handleSearch = (value: string) => {
        if (value.trim()) {
            onSearch(value.trim(), fuzzyEnabled);
        }
    };

    // 处理回车键
    const handleKeyPress = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Enter') {
            handleSearch(query);
        }
    };

    // 清除搜索
    const handleClear = () => {
        setQuery('');
        if (onClear) {
            onClear();
        }
    };

    return (
        <Space direction="vertical" style={{ width: '100%' }} size="middle">
            <Space.Compact style={{ width: '100%' }}>
                <Search
                    placeholder={placeholder}
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    onSearch={handleSearch}
                    onKeyPress={handleKeyPress}
                    disabled={disabled}
                    loading={loading}
                    allowClear
                    style={{ flex: 1 }}
                    size="large"
                    prefix={<SearchOutlined />}
                />
                {showClearButton && (
                    <Tooltip title="清除索引">
                        <Button
                            icon={<ClearOutlined />}
                            onClick={handleClear}
                            size="large"
                            danger
                        />
                    </Tooltip>
                )}
            </Space.Compact>

            {showFuzzySwitch && (
                <Space>
                    <Switch
                        checked={fuzzyEnabled}
                        onChange={setFuzzyEnabled}
                        disabled={disabled}
                    />
                    <Text type="secondary">启用模糊搜索</Text>
                </Space>
            )}
        </Space>
    );
}

export default SearchBox;