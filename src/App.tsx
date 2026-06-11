//! ZoPlus 论文管理软件 - 前端根组件
//!
//! 基于 React 18 + TypeScript + Ant Design 构建
//! 使用 React Router 实现页面路由
//! 支持暗色/亮色主题切换

import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { ConfigProvider, theme } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import { useState, useEffect, useCallback } from 'react';
import Dashboard from './pages/Dashboard';
import PdfReader from './pages/PdfReader';
import { lightTheme, darkTheme } from './themes/themeConfig';
import './App.css';

// 主题存储键名
const THEME_STORAGE_KEY = 'zoplus-theme-is-dark';

/// App 根组件
/// 配置 Ant Design 中文语言环境，设置 React Router 路由，集成主题切换
function App() {
  // 从 localStorage 读取初始主题状态
  const [isDark, setIsDark] = useState<boolean>(() => {
    try {
      const stored = localStorage.getItem(THEME_STORAGE_KEY);
      if (stored !== null) {
        return stored === 'true';
      }
    } catch (e) {
      console.warn('无法读取主题设置:', e);
    }
    // 默认使用亮色主题
    return false;
  });

  // 主题变化时保存到 localStorage
  useEffect(() => {
    try {
      localStorage.setItem(THEME_STORAGE_KEY, String(isDark));
    } catch (e) {
      console.warn('无法保存主题设置:', e);
    }
  }, [isDark]);

  // 切换主题回调
  const handleToggleTheme = useCallback(() => {
    setIsDark(prev => !prev);
  }, []);

  return (
    <ConfigProvider
      locale={zhCN}
      theme={{
        algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
        ...(isDark ? darkTheme : lightTheme),
      }}
    >
      <BrowserRouter>
       <Routes>
          {/* 根路径重定向到 Dashboard */}
          <Route path="/" element={<Navigate to="/dashboard" replace />} />

          {/* Dashboard - 主界面 */}
          <Route path="/dashboard" element={<Dashboard isDark={isDark} onToggleTheme={handleToggleTheme} />} />

          {/* PDF 阅读器页面 */}
          <Route path="/pdf" element={<PdfReader />} />
          <Route path="/pdf/:itemId" element={<PdfReader />} />
        </Routes>
      </BrowserRouter>
    </ConfigProvider>
  );
}

export default App;