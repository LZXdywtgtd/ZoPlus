//! 同步客户端模块
//!
//! 实现与阿里云同步服务器的 HTTP 通信

use crate::sync::config::{
    Change, DeviceInfo, ServerStatus, SyncResult,
};
use reqwest::Client;
use std::path::Path;
use thiserror::Error;

///同步错误类型
#[derive(Error, Debug)]
pub enum SyncError {
    /// 网络连接失败
    #[error("网络连接失败: {0}")]
    NetworkError(String),

    /// 服务器返回错误
    #[error("服务器错误: {0}")]
    ServerError(String),

    /// 认证失败
    #[error("认证失败，请检查同步令牌是否正确")]
    AuthFailed,

    /// 文件操作失败
    #[error("文件操作失败: {0}")]
    FileError(String),

    /// 超时
    #[error("请求超时")]
    Timeout,

    /// 未知错误
    #[error("未知错误: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for SyncError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            SyncError::Timeout
        } else if err.is_connect() {
            SyncError::NetworkError(err.to_string())
        } else {
            SyncError::Unknown(err.to_string())
        }
    }
}

/// 同步客户端
pub struct SyncClient {
    /// 服务器地址
    server_url: String,
    /// HTTP 客户端
    client: Client,
    /// 认证令牌
    auth_token: String,
}

impl SyncClient {
    /// 创建新的同步客户端
    pub fn new(server_url: String, auth_token: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("创建同步客户端失败");

        Self {
            server_url,
            client,
            auth_token,
        }
    }

    /// 获取认证头
    fn auth_header(&self) -> String {
        format!("Bearer {}", self.auth_token)
    }

    /// 上传数据库备份
    ///
    /// # 参数
    /// * `db_path` - 数据库文件路径
    ///
    /// # 返回值
    /// * `Result<SyncResult, SyncError>` - 同步结果
    pub async fn upload_database(&self, db_path: &Path) -> Result<SyncResult, SyncError> {
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;

        let file_name = db_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("zotero.sqlite")
            .to_string();

        let mut file = File::open(db_path).await.map_err(|e| {
            SyncError::FileError(format!("无法打开数据库文件: {}", e))
        })?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await.map_err(|e| {
            SyncError::FileError(format!("无法读取数据库文件: {}", e))
        })?;

        let part = reqwest::multipart::Part::bytes(buffer)
            .file_name(file_name)
            .mime_str("application/octet-stream")
            .map_err(|e| SyncError::Unknown(e.to_string()))?;

        let form = reqwest::multipart::Form::new().part("database", part);

        let url = format!("{}/api/sync/upload", self.server_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .multipart(form)
            .send()
            .await?;

        if response.status() == 401 {
            return Err(SyncError::AuthFailed);
        }

        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap_or_default();
            return Err(SyncError::ServerError(error_msg));
        }

        let result = response.json::<SyncResult>().await.map_err(|e| {
            SyncError::Unknown(format!("解析服务器响应失败: {}", e))
        })?;

        Ok(result)
    }

    /// 下载最新数据库
    ///
    /// # 返回值
    /// * `Result<PathBuf, SyncError>` - 下载的数据库文件路径
    pub async fn download_database(&self) -> Result<std::path::PathBuf, SyncError> {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("zoplus_download.sqlite");

        let url = format!("{}/api/sync/download", self.server_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if response.status() == 401 {
            return Err(SyncError::AuthFailed);
        }

        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap_or_default();
            return Err(SyncError::ServerError(error_msg));
        }

        let bytes = response.bytes().await.map_err(|e| {
            SyncError::NetworkError(format!("下载数据库失败: {}", e))
        })?;

        tokio::fs::write(&temp_path, bytes).await.map_err(|e| {
            SyncError::FileError(format!("保存下载文件失败: {}", e))
        })?;

        Ok(temp_path)
    }

    /// 增量同步
    ///
    /// # 参数
    /// * `changes` - 变更记录列表
    ///
    /// # 返回值
    /// * `Result<SyncResult, SyncError>` - 同步结果
    pub async fn sync_incremental(&self, changes: Vec<Change>) -> Result<SyncResult, SyncError> {
        let url = format!("{}/api/sync/incremental", self.server_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&changes)
            .send()
            .await?;

        if response.status() == 401 {
            return Err(SyncError::AuthFailed);
        }

        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap_or_default();
            return Err(SyncError::ServerError(error_msg));
        }

        let result = response.json::<SyncResult>().await.map_err(|e| {
            SyncError::Unknown(format!("解析服务器响应失败: {}", e))
        })?;

        Ok(result)
    }

    /// 获取服务器状态
    ///
    /// # 返回值
    /// * `Result<ServerStatus, SyncError>` - 服务器状态
    pub async fn get_status(&self) -> Result<ServerStatus, SyncError> {
        let url = format!("{}/api/sync/status", self.server_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if response.status() == 401 {
            return Err(SyncError::AuthFailed);
        }

        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap_or_default();
            return Err(SyncError::ServerError(error_msg));
        }

        let status = response.json::<ServerStatus>().await.map_err(|e| {
            SyncError::Unknown(format!("解析服务器响应失败: {}", e))
        })?;

        Ok(status)
    }

    /// 注册设备
    ///
    /// # 参数
    /// * `device_info` - 设备信息
    ///
    /// # 返回值
    /// * `Result<(), SyncError>` - 注册结果
    pub async fn register_device(&self, device_info: DeviceInfo) -> Result<(), SyncError> {
        let url = format!("{}/api/sync/device", self.server_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&device_info)
            .send()
            .await?;

        if response.status() == 401 {
            return Err(SyncError::AuthFailed);
        }

        if !response.status().is_success() {
            let error_msg = response.text().await.unwrap_or_default();
            return Err(SyncError::ServerError(error_msg));
        }

        Ok(())
    }

    /// 检查服务器是否可用
    pub async fn ping(&self) -> bool {
        let url = format!("{}/api/health", self.server_url);
        match self.client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}