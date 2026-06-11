//!主题切换组件
//!
//! 提供暗色/亮色主题切换功能

import React from 'react';
import { Button, Tooltip } from 'antd';
import { SunOutlined, MoonOutlined } from '@ant-design/icons';

interface ThemeSwitchProps {
  isDark: boolean;
  onToggle: () => void;
}

/// 主题切换组件
/// 点击按钮切换暗色/亮色主题
const ThemeSwitch: React.FC<ThemeSwitchProps> = ({ isDark, onToggle }) => {
  return (
    <Tooltip title={isDark ? '切换到亮色主题' : '切换到暗色主题'}>
      <Button
        icon={isDark ? <SunOutlined /> : <MoonOutlined />}
        onClick={onToggle}
        type="text"
        size="large"
      />
    </Tooltip>
  );
};

export default ThemeSwitch;