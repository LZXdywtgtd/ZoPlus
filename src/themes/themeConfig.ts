//!主题配置模块
//!
//! 提供亮色主题和暗色主题的 Ant Design 配置

import type { ThemeConfig } from 'antd';

// 亮色主题配置
export const lightTheme: ThemeConfig = {
  token: {
    // 主题色
    colorPrimary: '#1890ff',
    // 圆角
    borderRadius: 4,
    // 成功色
    colorSuccess: '#52c41a',
    // 警告色
    colorWarning: '#faad14',
    // 错误色
    colorError: '#ff4d4f',
    // 信息色
    colorInfo: '#1890ff',
    // 背景色
    colorBgBase: '#ffffff',
    // 文字色
    colorTextBase: '#000000',
    // 边框色
    colorBorderBg: '#d9d9d9',
    // 链接色
    colorLink: '#1890ff',
    // 链接悬停色
    colorLinkHover: '#40a9ff',
    // 文字次级色
    colorTextSecondary: '#00000073',
    // 文字禁用色
    colorTextDisabled: '#0000003b',
    // 背景色（次级）
    colorBgLayout: '#f5f5f5',
    // 容器背景色
    colorBgContainer: '#ffffff',
    // 组件背景色
    colorBgElevated: '#ffffff',
    // 斑马纹背景色
    colorBgSpotlight: '#fafafa',
    // 字体
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif",
    // 字号
    fontSize: 14,
    // 行高
    lineHeight: 1.5715,
  },
  components: {
    // 按钮
    Button: {
      colorPrimary: '#1890ff',
      colorPrimaryHover: '#40a9ff',
      colorPrimaryActive: '#096dd9',
    },
    // 菜单
    Menu: {
      darkItemBg: '#001629',
      darkSubMenuItemBg: '#000c17',
      darkItemColor: 'rgba(255, 255, 255, 0.85)',
      darkItemHoverColor: '#ffffff',
    },
    // 布局
    Layout: {
      headerBg: '#ffffff',
      siderBg: '#001629',
      bodyBg: '#f5f5f5',
    },
    // 卡片
    Card: {
      colorBgContainer: '#ffffff',
    },
  },
};

// 暗色主题配置
export const darkTheme: ThemeConfig = {
  token: {
    // 主题色
    colorPrimary: '#1890ff',
    // 圆角
    borderRadius: 4,
    // 成功色
    colorSuccess: '#52c41a',
    // 警告色
    colorWarning: '#faad14',
    // 错误色
    colorError: '#ff4d4f',
    // 信息色
    colorInfo: '#1890ff',
    // 背景色
    colorBgBase: '#141414',
    // 文字色
    colorTextBase: '#ffffff',
    // 边框色
    colorBorderBg: '#434343',
    // 链接色
    colorLink: '#1890ff',
    // 链接悬停色
    colorLinkHover: '#40a9ff',
    // 文字次级色
    colorTextSecondary: '#ffffff73',
    // 文字禁用色
    colorTextDisabled: '#ffffff3b',
    // 背景色（次级）
    colorBgLayout: '#000000',
    // 容器背景色
    colorBgContainer: '#1f1f1f',
    // 组件背景色
    colorBgElevated: '#262626',
    // 斑马纹背景色
    colorBgSpotlight: '#262626',
    // 字体
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif",
    // 字号
    fontSize: 14,
    // 行高
    lineHeight: 1.5715,
  },
  components: {
    // 按钮
    Button: {
      colorPrimary: '#1890ff',
      colorPrimaryHover: '#40a9ff',
      colorPrimaryActive: '#096dd9',
    },
    // 菜单
    Menu: {
      darkItemBg: '#001629',
      darkSubMenuItemBg: '#000c17',
      darkItemColor: 'rgba(255, 255, 255, 0.85)',
      darkItemHoverColor: '#ffffff',
    },
    // 布局
    Layout: {
      headerBg: '#141414',
      siderBg: '#001629',
      bodyBg: '#000000',
    },
    // 卡片
    Card: {
      colorBgContainer: '#1f1f1f',
    },
  },
};