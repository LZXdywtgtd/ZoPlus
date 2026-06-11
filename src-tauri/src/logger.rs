//! ZoPlus 日志系统模块
//!
//! 提供基于 tracing 的永久日志系统，支持文件输出和控制台输出
//! 日志文件按日期轮转，保存在用户数据目录下

use std::path::PathBuf;
use std::sync::Once;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

static INIT: Once = Once::new();

/// 初始化日志系统
///
/// 在应用启动时调用一次即可，后续调用无效
///
/// # 参数
/// * `app_name` - 应用名称，用于构建日志目录路径
pub fn init_logger(app_name: &str) {
    INIT.call_once(|| {
        // 获取用户数据目录
        let log_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(app_name)
            .join("logs");

        // 确保日志目录存在
        std::fs::create_dir_all(&log_dir).ok();

        // 创建文件 appender（按日期轮转，保留7天）
        let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "zoplus.log");

        // 非阻塞写入
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        // 存储 guard 防止被 drop（Box::leak 确保不会被释放）
        Box::leak(Box::new(_guard));

        // 设置 subscriber
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
            .with(
                fmt::layer()
                    .with_writer(non_blocking)
                    .with_ansi(false)
                    .with_target(true)
                    .with_thread_ids(true),
            )
            .with(fmt::layer().with_writer(std::io::stderr).with_ansi(true));

        tracing::subscriber::set_global_default(subscriber).ok();

        tracing::info!("日志系统初始化完成，日志目录: {:?}", log_dir);
    });
}

/// 记录信息日志的宏
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
    };
}

/// 记录警告日志的宏
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*);
    };
}

/// 记录错误日志的宏
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*);
    };
}

/// 记录调试日志的宏
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*);
    };
}