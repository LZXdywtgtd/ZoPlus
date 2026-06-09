//! ZoPlus 论文管理软件 - 前端根组件
//!
//! 基于 React 18 + TypeScript + Ant Design 构建
//! 使用 React Router 实现页面路由

import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { ConfigProvider } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import Dashboard from './pages/Dashboard';
import PdfReader from './pages/PdfReader';
import './App.css';

/// App 根组件
/// 配置 Ant Design 中文语言环境，设置 React Router 路由
function App() {
  return (
    <ConfigProvider locale={zhCN}>
      <BrowserRouter>
        <Routes>
          {/* 根路径重定向到 Dashboard */}
          <Route path="/" element={<Navigate to="/dashboard" replace />} />

          {/* Dashboard - 主界面 */}
          <Route path="/dashboard" element={<Dashboard />} />

          {/* PDF 阅读器页面 */}
          <Route path="/pdf" element={<PdfReader />} />
          <Route path="/pdf/:itemId" element={<PdfReader />} />
        </Routes>
      </BrowserRouter>
    </ConfigProvider>
  );
}

export default App;
