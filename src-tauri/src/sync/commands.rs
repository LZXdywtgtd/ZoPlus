//! 同步 Tauri 命令模块
//!
//! 提供与前端交互的 Tauri Command

use crate::error::{get_user_message, AppError};
use crate::sync::config::{SyncConfig, SyncStatus};
use crate::sync::scheduler::SyncScheduler;
use std::sync::Arc;
use tokio::sync::RwLock;

// 全局同步调度器实例
lazy_static::lazy_static! {
    static ref SYNC_SCHEDULER: Arc<RwLock<SyncScheduler>> = {
        let config = SyncConfig::default();
        Arc::new(RwLock::new(SyncScheduler::new(config)))
    };
}

/// 获取同步调度器实例
pub fn get_sync_scheduler() -> Arc<RwLock<SyncScheduler>> {
    SYNC_SCHEDULER.clone()
}

/// Tauri 命令：立即执行同步
///
/// # 返回值
/// * `Result<SyncResult, String>` - 同步结果或错误信息
#[tauri::command]
pub async fn sync_now() -> Result<crate::sync::config::SyncResult, String> {
    eprintln!("[命令] sync_now 被调用");

    let scheduler = get_sync_scheduler();
    let scheduler_guard = scheduler.read().await;

    scheduler_guard.sync().await.map_err(|e| {
        eprintln!("[同步] 同步失败: {:?}", e);
        match e {
            crate::sync::client::SyncError::AuthFailed => {
                get_user_message(&AppError::SyncConnectionFailed).to_string()
            }
            crate::sync::client::SyncError::NetworkError(_) => {
                get_user_message(&AppError::SyncConnectionFailed).to_string()
            }
            _ => format!("同步失败: {}", e),
        }
    })
}

/// Tauri 命令：获取同步状态
///
/// # 返回值
/// * `Result<SyncStatus, String>` - 同步状态或错误信息
#[tauri::command]
pub async fn get_sync_status() -> Result<SyncStatus, String> {
    eprintln!("[命令] get_sync_status 被调用");

    let scheduler = get_sync_scheduler();
    let scheduler_guard = scheduler.read().await;

    Ok(scheduler_guard.get_status().await)
}

/// Tauri 命令：配置同步
///
/// # 参数
/// * `config` - 同步配置
///
/// # 返回值
/// * `Result<(), String>` - 配置结果或错误信息
#[tauri::command]
pub async fn configure_sync(config: SyncConfig) -> Result<(), String> {
    eprintln!("[命令] configure_sync 被调用: enabled={}", config.enabled);

    let scheduler = get_sync_scheduler();
    let scheduler_guard = scheduler.read().await;

    scheduler_guard.update_config(config).await;
    Ok(())
}

/// Tauri 命令：获取同步配置
///
/// # 返回值
/// * `Result<SyncConfig, String>` - 同步配置或错误信息
#[tauri::command]
pub async fn get_sync_config() -> Result<SyncConfig, String> {
    eprintln!("[命令] get_sync_config 被调用");

    let scheduler = get_sync_scheduler();
    let scheduler_guard = scheduler.read().await;

    Ok(scheduler_guard.get_config().await)
}

/// Tauri 命令：启动后台同步
#[tauri::command]
pub async fn start_background_sync() -> Result<(), String> {
    eprintln!("[命令] start_background_sync 被调用");

    let scheduler = get_sync_scheduler();
    let scheduler_guard = scheduler.read().await;
    scheduler_guard.start_background_sync();

    Ok(())
}

/// Tauri 命令：停止后台同步
#[tauri::command]
pub async fn stop_background_sync() -> Result<(), String> {
    eprintln!("[命令] stop_background_sync 被调用");

    let scheduler = get_sync_scheduler();
    let scheduler_guard = scheduler.read().await;
    scheduler_guard.stop_background_sync();

    Ok(())
}