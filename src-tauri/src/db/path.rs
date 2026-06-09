//! Zotero 数据库路径检测模块
//!
//! 本模块负责跨平台自动检测 Zotero 数据库文件路径。
//! 支持 Windows、macOS、Linux三大主流操作系统。

use std::path::PathBuf;
use std::env;

/// 检测 Zotero 数据库文件路径
///
/// #路径规则
/// - Windows: `%USERPROFILE%\Zotero\zotero.sqlite`
/// - macOS: `~/Zotero/zotero.sqlite`
/// - Linux: `~/.zotero/zotero.sqlite`
///
/// # 返回值
/// * `PathBuf` - Zotero 数据库文件的完整路径
///
/// # 示例
/// ```
/// let db_path = get_zotero_db_path();
/// println!("Zotero 数据库路径: {:?}", db_path);
/// ```
pub fn get_zotero_db_path() -> PathBuf {
    let home_dir = env::home_dir().expect("无法获取用户主目录");

    #[cfg(target_os = "windows")]
    let db_path = {
        let mut path = home_dir;
        path.push("Zotero");
        path.push("zotero.sqlite");
        path
    };

    #[cfg(target_os = "macos")]
    let db_path = {
        let mut path = home_dir;
        path.push("Zotero");
        path.push("zotero.sqlite");
        path
    };

    #[cfg(target_os = "linux")]
    let db_path = {
        let mut path = home_dir;
        path.push(".zotero");
        path.push("zotero.sqlite");
        path
    };

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let db_path = {
        // 默认使用 Linux 路径作为后备方案
        let mut path = home_dir;
        path.push(".zotero");
        path.push("zotero.sqlite");
        path
    };

    db_path
}

/// 检查 Zotero 数据库文件是否存在
///
/// # 返回值
/// * `bool` - 数据库文件是否存在
pub fn zotero_db_exists() -> bool {
    get_zotero_db_path().exists()
}