//! ZoPlus 论文管理软件 - 前端根组件
//!
//! 基于 React 18 + TypeScript + Ant Design 构建

import { ConfigProvider } from 'antd';
import zhCN from 'antd/locale/zh_CN';
import Dashboard from './pages/Dashboard';
import './App.css';

/// App 根组件
/// 配置 Ant Design 中文语言环境，渲染主界面布局
function App() {
  return (
    <ConfigProvider locale={zhCN}>
      <Dashboard />
    </ConfigProvider>
  );
}

export default App;