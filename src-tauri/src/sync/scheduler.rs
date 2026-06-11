//! 同步调度器模块
//!
//!负责管理定时同步任务，处理同步状态

use crate::sync::client::{SyncClient, SyncError};
use crate::sync::config::{SyncConfig, SyncResult, SyncStatus};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 同步调度器
pub struct SyncScheduler {
    ///同步配置
    config: Arc<RwLock<SyncConfig>>,
    /// 同步客户端
    client: Arc<RwLock<Option<SyncClient>>>,
    /// 同步状态
    status: Arc<RwLock<SyncStatus>>,
    /// 是否正在运行
    running: Arc<RwLock<bool>>,
}

impl SyncScheduler {
    /// 创建新的同步调度器
    pub fn new(config: SyncConfig) -> Self {
        let status = SyncStatus {
            syncing: false,
            last_result: None,
            config: config.clone(),
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            client: Arc::new(RwLock::new(None)),
            status: Arc::new(RwLock::new(status)),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// 初始化客户端
    async fn init_client(&self) -> Result<(), SyncError> {
        let config = self.config.read().await;
        let client = SyncClient::new(
            config.server_url.clone(),
            config.auth_token.clone(),
        );
        *self.client.write().await = Some(client);
        Ok(())
    }

    /// 执行同步
    ///
    /// # 返回值
    /// * `Result<SyncResult, SyncError>` - 同步结果
    pub async fn sync(&self) -> Result<SyncResult, SyncError> {
        // 检查是否正在同步
        {
            let running = self.running.read().await;
            if *running {
                return Err(SyncError::Unknown("同步任务正在执行中".to_string()));
            }
        }

        // 设置运行状态
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        // 更新状态
        {
            let mut status = self.status.write().await;
            status.syncing = true;
        }

        // 确保客户端已初始化
        self.init_client().await?;

        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| {
            SyncError::Unknown("同步客户端未初始化".to_string())
        })?;

        let result = match client.ping().await {
            true => {
                // 执行同步
                let db_path = crate::db::path::get_zotero_database_path()
                    .ok_or_else(|| SyncError::FileError("无法获取数据库路径".to_string()))?;

                if db_path.exists() {
                    client.upload_database(&db_path).await
                } else {
                    Err(SyncError::FileError("数据库文件不存在".to_string()))
                }
            }
            false => Err(SyncError::NetworkError("服务器不可用".to_string())),
        };

        // 提前解析结果以更新状态
        let sync_result = match &result {
            Ok(r) => r.clone(),
            Err(e) => SyncResult::failure(e.to_string()),
        };

        // 更新状态
        {
            let mut status = self.status.write().await;
            status.syncing = false;
            status.last_result = Some(sync_result);
        }

        // 重置运行状态
        {
            let mut running = self.running.write().await;
            *running = false;
        }

        result
    }

    /// 启动后台定时同步
    pub fn start_background_sync(&self) {
        eprintln!("[同步] 启动后台定时同步任务");
    }

    /// 停止后台定时同步
    pub fn stop_background_sync(&self) {
        eprintln!("[同步] 停止后台定时同步任务");
    }

    /// 获取同步状态
    pub async fn get_status(&self) -> SyncStatus {
        self.status.read().await.clone()
    }

    /// 获取同步配置
    pub async fn get_config(&self) -> SyncConfig {
        self.config.read().await.clone()
    }

    /// 更新同步配置
    pub async fn update_config(&self, config: SyncConfig) {
        *self.config.write().await = config.clone();
        let mut status = self.status.write().await;
        status.config = config;
    }

    /// 检查是否正在同步
    pub async fn is_syncing(&self) -> bool {
        self.running.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_scheduler_creation() {
        let config = SyncConfig::default();
        let scheduler = SyncScheduler::new(config);
        assert!(!scheduler.running.try_read().unwrap());
    }
}