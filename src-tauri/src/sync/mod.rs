//! 阿里云同步服务模块
//!
//! 本模块负责与阿里云服务器进行数据同步。
//! 彻底摒弃 WebDAV 方案，使用自研同步协议。
//!
//! # 功能说明
//! - [x] 传输加密（HTTPS）
//! - [x] 增量同步
//! - [x] 冲突处理策略
//! - [ ] 存储加密（待后续实现）

pub mod client;
pub mod commands;
pub mod config;
pub mod scheduler;

pub use client::{SyncClient, SyncError};
pub use commands::{
    configure_sync, get_sync_config, get_sync_status, start_background_sync, stop_background_sync,
    sync_now,
};
pub use config::{ConflictStrategy, DeviceInfo, ServerStatus, SyncConfig, SyncMode, SyncResult, SyncStatus};
pub use scheduler::SyncScheduler;