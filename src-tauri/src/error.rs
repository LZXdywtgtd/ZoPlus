//! ZoPlus 统一错误处理模块
//!
//! 本模块实现用户友好错误消息与控制台详细日志的分离。
//! UI 层显示中文友好消息，控制台输出完整技术信息。
//!
//! # 错误类型
//! - `AppError` - 应用层错误枚举
//! - `DbError` - 数据库错误
//! - `IndexerError` - 索引错误
//! - `StorageError` - 存储错误
//!
//! # 使用方式
//! ```rust
//! // 技术性错误转换为用户友好消息
//! db_operation().map_err(|e| {
//!     eprintln!("[数据库] 查询失败: {:?}", e);  // 控制台详细日志
//!     get_user_message(&AppError::QueryFailed)   // 用户友好消息
//! })
//! ```

use thiserror::Error;

/// 应用错误类型枚举
#[derive(Error, Debug)]
pub enum AppError {
    /// 数据库连接失败
    #[error("数据库连接失败，请检查 Zotero 是否已安装")]
    DatabaseConnectionFailed,

    /// 数据库查询失败
    #[error("文献列表加载失败，请稍后重试")]
    QueryFailed,

    /// 数据库文件未找到
    #[error("未找到 Zotero 数据库，请确保 Zotero 已正确安装")]
    DatabaseNotFound,

    /// 数据库被其他进程占用
    #[error("数据库被其他程序占用，请关闭其他可能访问 Zotero 的程序后重试")]
    DatabaseLocked,

    /// 索引初始化失败
    #[error("索引初始化失败，可能被其他进程占用，请关闭其他程序后重试")]
    IndexInitFailed,

    /// 索引构建失败
    #[error("索引构建失败，请检查磁盘空间是否充足")]
    IndexBuildFailed,

    /// 索引路径未初始化
    #[error("索引未初始化，请先启动索引构建")]
    IndexNotInitialized,

    /// 搜索引擎未初始化
    #[error("搜索引擎未初始化，请重启应用")]
    SearchEngineNotInitialized,

    /// 搜索操作失败
    #[error("搜索失败，请稍后重试")]
    SearchFailed,

    /// 索引操作失败
    #[error("索引操作失败，请检查索引目录是否可写")]
    IndexOperationFailed,

    /// PDF 标注存储初始化失败
    #[error("标注存储初始化失败，请检查应用数据目录权限")]
    StorageInitFailed,

    /// PDF 标注保存失败
    #[error("标注保存失败，请确保 PDF 文件未被其他程序占用")]
    AnnotationSaveFailed,

    /// PDF 标注加载失败
    #[error("标注加载失败，文件可能已损坏")]
    AnnotationLoadFailed,

    /// PDF 标注删除失败
    #[error("标注删除失败，请稍后重试")]
    AnnotationDeleteFailed,

    /// 文件路径无效
    #[error("文件路径无效，请检查文件是否可访问")]
    InvalidPath,

    /// 系统路径获取失败
    #[error("无法获取系统路径，请检查操作系统配置")]
    SystemPathFailed,

    /// AI 服务调用失败
    #[error("AI 服务暂时不可用，请稍后重试")]
    AIServiceFailed,

    /// 云同步服务连接失败
    #[error("云同步服务连接失败，请检查网络设置")]
    SyncConnectionFailed,

    /// 内部错误（不应暴露给用户）
    #[error("内部错误: {0}")]
    Internal(String),
}

/// 获取用户友好的错误消息
///
/// # 参数
/// * `err` - 错误引用
///
/// # 返回值
/// * `&'static str` - 用户友好的中文错误消息
pub fn get_user_message(err: &AppError) -> &'static str {
    match err {
        AppError::DatabaseConnectionFailed => "数据库连接失败，请检查 Zotero 是否已安装",
        AppError::QueryFailed => "文献列表加载失败，请稍后重试",
        AppError::DatabaseNotFound => "未找到 Zotero 数据库，请确保 Zotero 已正确安装",
        AppError::DatabaseLocked => "数据库被其他程序占用，请关闭其他可能访问 Zotero 的程序后重试",
        AppError::IndexInitFailed => "索引初始化失败，可能被其他进程占用，请关闭其他程序后重试",
        AppError::IndexBuildFailed => "索引构建失败，请检查磁盘空间是否充足",
        AppError::IndexNotInitialized => "索引未初始化，请先启动索引构建",
        AppError::SearchEngineNotInitialized => "搜索引擎未初始化，请重启应用",
        AppError::SearchFailed => "搜索失败，请稍后重试",
        AppError::IndexOperationFailed => "索引操作失败，请检查索引目录是否可写",
        AppError::StorageInitFailed => "标注存储初始化失败，请检查应用数据目录权限",
        AppError::AnnotationSaveFailed => "标注保存失败，请确保 PDF 文件未被其他程序占用",
        AppError::AnnotationLoadFailed => "标注加载失败，文件可能已损坏",
        AppError::AnnotationDeleteFailed => "标注删除失败，请稍后重试",
        AppError::InvalidPath => "文件路径无效，请检查文件是否可访问",
        AppError::SystemPathFailed => "无法获取系统路径，请检查操作系统配置",
        AppError::AIServiceFailed => "AI 服务暂时不可用，请稍后重试",
        AppError::SyncConnectionFailed => "云同步服务连接失败，请检查网络设置",
        AppError::Internal(_) => "操作失败，请稍后重试",
    }
}

/// 将技术性错误转换为用户友好的错误消息字符串
///
/// # 参数
/// * `err` - 错误引用
///
/// # 返回值
/// * `String` - 用户友好的错误消息字符串
pub fn to_user_message(err: &AppError) -> String {
    get_user_message(err).to_string()
}

/// 输出详细错误日志到控制台（不暴露给用户）
///
/// # 参数
/// * `module` - 模块名称（如 "数据库"、"索引"）
/// * `operation` - 操作名称（如 "查询"、"构建"）
/// * `err` - 错误引用
pub fn log_error(module: &str, operation: &str, err: &AppError) {
    eprintln!("[{}] {}失败: {:?}", module, operation, err);
    if let AppError::Internal(detail) = err {
        eprintln!("[{}] 详细错误信息: {}", module, detail);
    }
}

/// 将数据库错误转换为应用错误
///
/// # 参数
/// * `db_err` - 数据库错误字符串
///
/// # 返回值
/// * `AppError` - 应用错误
pub fn from_db_error(db_err: &str) -> AppError {
    if db_err.contains("locked") || db_err.contains("Lockfile") {
        AppError::DatabaseLocked
    } else if db_err.contains("not found") || db_err.contains("不存在") {
        AppError::DatabaseNotFound
    } else {
        AppError::QueryFailed
    }
}

/// 将索引错误转换为应用错误
///
/// # 参数
/// * `index_err` - 索引错误字符串
///
/// # 返回值
/// * `AppError` - 应用错误
pub fn from_index_error(index_err: &str) -> AppError {
    if index_err.contains("locked") || index_err.contains("占用") {
        AppError::IndexInitFailed
    } else if index_err.contains("path") || index_err.contains("路径") {
        AppError::IndexNotInitialized
    } else if index_err.contains("build") || index_err.contains("构建") {
        AppError::IndexBuildFailed
    } else {
        AppError::IndexOperationFailed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message() {
        assert_eq!(get_user_message(&AppError::DatabaseConnectionFailed),
            "数据库连接失败，请检查 Zotero 是否已安装");
        assert_eq!(get_user_message(&AppError::QueryFailed),
            "文献列表加载失败，请稍后重试");
    }

    #[test]
    fn test_from_db_error() {
        assert!(matches!(from_db_error("database locked"), AppError::DatabaseLocked));
        assert!(matches!(from_db_error("not found"), AppError::DatabaseNotFound));
    }
}