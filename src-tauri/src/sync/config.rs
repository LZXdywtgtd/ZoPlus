//! 同步配置模块
//!
//! 定义同步功能所需的配置结构体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 同步模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SyncMode {
    /// 全量同步
    Full,
    /// 增量同步
    Incremental,
}

impl Default for SyncMode {
    fn default() -> Self {
        SyncMode::Incremental
    }
}

/// 冲突策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConflictStrategy {
    /// 本地优先
    LocalFirst,
    /// 远程优先
    RemoteFirst,
    /// 手动处理
    Manual,
}

impl Default for ConflictStrategy {
    fn default() -> Self {
        ConflictStrategy::LocalFirst
    }
}

/// 同步配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// 是否启用云同步
    pub enabled: bool,
    /// 服务器地址
    pub server_url: String,
    /// 认证令牌
    pub auth_token: String,
    /// 同步模式
    pub sync_mode: SyncMode,
    /// 冲突策略
    pub conflict_strategy: ConflictStrategy,
    /// 最后同步时间
    pub last_sync_time: Option<DateTime<Utc>>,
    /// 设备唯一ID
    pub device_id: String,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            server_url: String::from("http://121.41.223.84:4000"),
            auth_token: String::new(),
            sync_mode: SyncMode::Incremental,
            conflict_strategy: ConflictStrategy::LocalFirst,
            last_sync_time: None,
            device_id: String::new(),
        }
    }
}

impl SyncConfig {
    /// 创建新的同步配置
    pub fn new(server_url: String, auth_token: String, device_id: String) -> Self {
        Self {
            enabled: false,
            server_url,
            auth_token,
            sync_mode: SyncMode::Incremental,
            conflict_strategy: ConflictStrategy::LocalFirst,
            last_sync_time: None,
            device_id,
        }
    }

    /// 检查配置是否有效
    pub fn is_valid(&self) -> bool {
        !self.server_url.is_empty() && !self.auth_token.is_empty()
    }

    /// 生成设备ID
    pub fn generate_device_id() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 16] = rng.gen();
        hex::encode(bytes)
    }
}

/// 同步状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    /// 是否正在同步
    pub syncing: bool,
    /// 上次同步结果
    pub last_result: Option<SyncResult>,
    /// 当前配置
    pub config: SyncConfig,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self {
            syncing: false,
            last_result: None,
            config: SyncConfig::default(),
        }
    }
}

/// 同步结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// 是否成功
    pub success: bool,
    /// 上传记录数
    pub uploaded: u32,
    /// 下载记录数
    pub downloaded: u32,
    /// 冲突数量
    pub conflicts: u32,
    /// 同步时间
    pub timestamp: DateTime<Utc>,
    /// 错误信息
    pub error_message: Option<String>,
}

impl SyncResult {
    /// 创建成功结果
    pub fn success(uploaded: u32, downloaded: u32, conflicts: u32) -> Self {
        Self {
            success: true,
            uploaded,
            downloaded,
            conflicts,
            timestamp: Utc::now(),
            error_message: None,
        }
    }

    /// 创建失败结果
    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            uploaded: 0,
            downloaded: 0,
            conflicts: 0,
            timestamp: Utc::now(),
            error_message: Some(message),
        }
    }
}

/// 变更记录（用于增量同步）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// 变更类型
    pub change_type: ChangeType,
    /// 表名
    pub table_name: String,
    /// 记录ID
    pub record_id: i32,
    /// 变更数据（JSON格式）
    pub data: String,
    /// 变更时间
    pub timestamp: DateTime<Utc>,
}

/// 变更类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    /// 插入
    Insert,
    /// 更新
    Update,
    /// 删除
    Delete,
}

///设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// 设备ID
    pub device_id: String,
    /// 设备名称
    pub device_name: String,
    /// 设备类型
    pub device_type: String,
    /// 最后同步时间
    pub last_sync: Option<DateTime<Utc>>,
}

impl DeviceInfo {
    /// 创建新设备信息
    pub fn new(device_id: String, device_name: String) -> Self {
        Self {
            device_id,
            device_name,
            device_type: std::env::consts::OS.to_string(),
            last_sync: None,
        }
    }
}

/// 服务器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    /// 服务器是否可用
    pub available: bool,
    /// 最新数据库版本
    pub latest_version: Option<DateTime<Utc>>,
    /// 已注册设备数
    pub device_count: u32,
    /// 服务器时间
    pub server_time: Option<DateTime<Utc>>,
}

// 以 hex 编码依赖，需要在 Cargo.toml 中添加
mod hex {
    const HEX_CHARS: &[u8] = b"0123456789abcdef";

    pub fn encode(bytes: [u8; 16]) -> String {
        let mut s = String::with_capacity(32);
        for &b in &bytes {
            s.push(HEX_CHARS[(b >> 4) as usize] as char);
            s.push(HEX_CHARS[(b & 0xf) as usize] as char);
        }
        s
    }
}