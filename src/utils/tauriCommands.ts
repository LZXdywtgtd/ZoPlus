//! Tauri 命令调用封装模块
//!
//! 本模块封装所有与 Rust 后端的 IPC 通信，确保前端不直接访问数据库

import { invoke } from '@tauri-apps/api/core';
import type { ItemInfo } from '../store/appStore';

/// 调用 Rust 后端获取所有文献列表
///
/// @returns Promise<ItemInfo[]> 文献列表
/// @throws 数据库连接失败或查询错误时抛出异常
export async function getItems(): Promise<ItemInfo[]> {
  return await invoke<ItemInfo[]>('get_items');
}

/// 根据 ID 获取单条文献信息
///
/// @param itemId - 文献 ID
/// @returns Promise<ItemInfo | null> 文献信息，不存在时返回 null
/// @throws 数据库连接失败或查询错误时抛出异常
export async function getItem(itemId: number): Promise<ItemInfo | null> {
  return await invoke<ItemInfo | null>('get_item', { item_id: itemId });
}

/// 检查 Zotero 数据库状态
///
/// @returns Promise<boolean> 数据库是否存在
export async function checkDbStatus(): Promise<boolean> {
  return await invoke<boolean>('check_db_status');
}