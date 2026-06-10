//! 应用状态管理模块
//!
//! 使用 Zustand 管理全局状态，包括数据库状态和文献列表

import { create } from 'zustand';
import type { ArticleSummary } from '../utils/tauriCommands';

/// 文献信息结构体（对应 Rust 端的 ItemInfo）
export interface ItemInfo {
  item_id: number;
  title: string;
  authors: string;
  year: string;
}

/// 数据库状态
export interface DbStatus {
  isConnected: boolean;
  isChecking: boolean;
  error: string | null;
}

/// 摘要状态
export interface SummaryStatus {
  isGenerating: boolean;
  currentItemId: number | null;
  error: string | null;
}

/// 应用状态类型
interface AppState {
  // 数据库状态
  dbStatus: DbStatus;

  // 文献列表
  items: ItemInfo[];
  itemsLoading: boolean;
  itemsError: string | null;

  // 侧边栏折叠状态
  siderCollapsed: boolean;

  // 摘要状态
  summaryStatus: SummaryStatus;
  currentSummary: ArticleSummary | null;

  // 设置数据库状态
  setDbStatus: (status: Partial<DbStatus>) => void;

  // 设置文献列表
  setItems: (items: ItemInfo[]) => void;

  // 设置加载状态
  setItemsLoading: (loading: boolean) => void;

  // 设置错误信息
  setItemsError: (error: string | null) => void;

  // 切换侧边栏
  toggleSider: () => void;

  // 摘要相关操作
  setSummaryGenerating: (generating: boolean, itemId?: number | null) => void;
  setSummaryError: (error: string | null) => void;
  setCurrentSummary: (summary: ArticleSummary | null) => void;
}

/// 创建全局状态存储
const useAppStore = create<AppState>((set) => ({
  // 初始数据库状态
  dbStatus: {
    isConnected: false,
    isChecking: true,
    error: null,
  },

  // 初始文献列表
  items: [],
  itemsLoading: false,
  itemsError: null,

  // 初始侧边栏状态
  siderCollapsed: false,

  // 初始摘要状态
  summaryStatus: {
    isGenerating: false,
    currentItemId: null,
    error: null,
  },
  currentSummary: null,

  // 设置数据库状态
  setDbStatus: (status) =>
    set((state) => ({
      dbStatus: { ...state.dbStatus, ...status },
    })),

  // 设置文献列表
  setItems: (items) => set({ items }),

  // 设置加载状态
  setItemsLoading: (loading) => set({ itemsLoading: loading }),

  // 设置错误信息
  setItemsError: (error) => set({ itemsError: error }),

  // 切换侧边栏
  toggleSider: () =>
    set((state) => ({
      siderCollapsed: !state.siderCollapsed,
    })),

  // 摘要相关操作
  setSummaryGenerating: (generating, itemId = null) =>
    set((state) => ({
      summaryStatus: {
        ...state.summaryStatus,
        isGenerating: generating,
        currentItemId: itemId,
      },
    })),

  setSummaryError: (error) =>
    set((state) => ({
      summaryStatus: {
        ...state.summaryStatus,
        error,
      },
    })),

  setCurrentSummary: (summary) =>
    set({ currentSummary: summary }),
}));

export default useAppStore;