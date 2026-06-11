//! Zotero 数据库路径检测模块
//!
//! 本模块负责跨平台自动检测 Zotero 数据库文件路径。
//! 支持 Windows、macOS、Linux三大主流操作系统。
//!
//! # 功能特性
//! - 默认路径检测：优先检测各平台标准默认路径
//! - 全磁盘扫描：当默认路径不存在时，自动扫描全磁盘查找数据库
//! - 最新优先：多个数据库时自动选择最后修改时间最新的
//! - 深度限制：递归搜索限制深度，避免性能问题

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

/// Zotero 数据库文件名
const ZOTERO_DB_NAME: &str = "zotero.sqlite";

/// 递归搜索的最大深度限制，避免性能问题
const MAX_SEARCH_DEPTH: usize = 5;

/// 检测 Zotero 数据库文件路径（带缓存优化）
///
/// # 缓存机制
/// - 首次调用时执行完整路径检测（默认路径检测或全磁盘扫描）
/// - 后续调用直接返回缓存路径，避免重复扫描
/// - 当缓存路径失效（文件被删除）时，重新执行一次完整检测
///
/// # 路径规则
/// - Windows: `%USERPROFILE%\Zotero\zotero.sqlite`
/// - macOS: `~/Zotero/zotero.sqlite`
/// - Linux: `~/.zotero/zotero.sqlite`
///
/// # 选择策略
/// 1. 首先返回缓存路径（如果缓存有效）
/// 2. 如果缓存无效或不存在，检测默认路径
/// 3. 如果默认路径不存在，执行全磁盘扫描
/// 4. 如果找到多个数据库，选择最后修改时间最新的
///
/// # 返回值
/// * `PathBuf` - Zotero 数据库文件的完整路径
///
/// # 示例
/// ```
/// let db_path = get_zotero_db_path();
/// println!("Zotero 数据库路径: {:?}", db_path);
/// ```
pub fn get_zotero_database_path() -> Option<PathBuf> {
    // 第一步：检查缓存路径是否有效
    if let Some(cached) = get_cached_path() {
        if cached.exists() {
            eprintln!("[Zotero路径检测] 使用缓存路径: {:?}", cached);
            return Some(cached);
        }
        eprintln!("[Zotero路径检测] 缓存路径失效（文件不存在）: {:?}", cached);
    }

    // 第二步：尝试默认路径
    let default_path = get_default_zotero_path();
    eprintln!("[Zotero路径检测] 尝试默认路径: {:?}", default_path);

    if default_path.exists() {
        eprintln!("[Zotero路径检测] 默认路径存在，使用默认路径");
        // 更新缓存
        update_cached_path(&default_path);
        return Some(default_path);
    }

    eprintln!("[Zotero路径检测] 默认路径不存在，开始全磁盘扫描...");

    // 第三步：全磁盘扫描
    let scanned_path = scan_for_zotero_db();

    if let Some(path) = scanned_path {
        eprintln!("[Zotero路径检测] 全磁盘扫描找到数据库: {:?}", path);
        // 更新缓存
        update_cached_path(&path);
        Some(path)
    } else {
        // 第四步：兜底返回默认路径（即使不存在）
        eprintln!("[Zotero路径检测] 全磁盘扫描未找到数据库，返回默认路径作为兜底");
        Some(default_path)
    }
}

/// 获取缓存的数据库路径
///
/// # 返回值
/// * `Option<PathBuf>` - 缓存路径（如果存在）
fn get_cached_path() -> Option<PathBuf> {
    // 从环境变量或静态变量中获取缓存路径
    // 这里使用 std::env::var读取上一次缓存的路径
    std::env::var("ZOTERO_DB_PATH").ok().map(PathBuf::from)
}

/// 更新缓存的数据库路径
///
/// # 参数
/// * `path` - 要缓存的路径
fn update_cached_path(path: &PathBuf) {
    // 将路径写入环境变量，实现跨调用缓存
    std::env::set_var("ZOTERO_DB_PATH", path.to_string_lossy().as_ref());
}

/// 获取默认的 Zotero 数据库路径（不检查是否存在）
/// 简化：Windows/macOS 共用 "Zotero" 子目录，Linux 使用 ".zotero"
fn get_default_zotero_path() -> PathBuf {
    let home_dir = env::home_dir().expect("无法获取用户主目录");
    let subdir = if cfg!(target_os = "linux") {
        ".zotero"
    } else {
        "Zotero"
    };

    let mut path = home_dir;
    path.push(subdir);
    path.push(ZOTERO_DB_NAME);
    path
}

/// 全磁盘扫描，查找 Zotero 数据库（简化：合并各平台扫描逻辑）
///
/// # 扫描策略
/// - Windows：扫描所有盘符，递归查找 zotero.sqlite
/// - macOS：扫描常见目录 ~/Zotero, ~/.zotero, ~/Library/Application Support/Zotero
/// - Linux：扫描常见目录 ~/.zotero, ~/.local/share/Zotero, ~/Zotero
///
/// # 返回值
/// * `Option<PathBuf>` - 找到则返回路径，否则返回 None
fn scan_for_zotero_db() -> Option<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "windows")] {
            scan_windows_for_zotero_db()
        } else if #[cfg(target_os = "macos")] {
            scan_macos_for_zotero_db()
        } else if #[cfg(target_os = "linux")] {
            scan_linux_for_zotero_db()
        } else {
            // 未知系统，扫描用户主目录
            // 注意：scan_directory_for_zotero_db 返回 (PathBuf, SystemTime)，只取 PathBuf
            env::home_dir()
                .and_then(|home| scan_directory_for_zotero_db(&home, 0))
                .map(|(path, _)| path)
        }
    }
}

/// Windows 平台：扫描所有盘符查找 Zotero 数据库
/// 简化：合并扫描逻辑，统一收集候选者
#[cfg(target_os = "windows")]
fn scan_windows_for_zotero_db() -> Option<PathBuf> {
    eprintln!("[Zotero路径检测] 开始 Windows 全磁盘扫描...");

    let drives = get_windows_drives();
    eprintln!(
        "[Zotero路径检测] 发现 {} 个盘符: {:?}",
        drives.len(),
        drives
    );

    // 收集所有盘符中的候选数据库
    let candidates: Vec<(PathBuf, SystemTime)> = drives
        .iter()
        .filter_map(|drive| {
            eprintln!("[Zotero路径检测] 扫描盘符: {}", drive.display());
            scan_directory_for_zotero_db(drive, 0)
        })
        .collect();

    select_best_candidate(candidates)
}

/// 获取 Windows 系统所有可用的盘符
/// 简化：提取常见盘符备选方案，减少重复逻辑
#[cfg(target_os = "windows")]
fn get_windows_drives() -> Vec<PathBuf> {
    let output = Command::new("wmic")
        .args(["logicaldisk", "get", "name"])
        .output();

    match output {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            eprintln!("[Zotero路径检测] wmic 输出: {}", output_str);

            // 解析输出，每行是一个盘符，盘符格式如 "C:"
            output_str
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    let len = line.len();
                    if len >= 2 && line.chars().last()? == ':' {
                        Some(PathBuf::from(format!("{}\\", &line[..2])))
                    } else {
                        None
                    }
                })
                .collect()
        }
        Err(e) => {
            eprintln!(
                "[Zotero路径检测] wmic 执行失败: {}，使用常见盘符作为备选",
                e
            );
            // wmic 失败时使用常见盘符作为备选
            ['C', 'D', 'E', 'F', 'G', 'H']
                .iter()
                .map(|&letter| PathBuf::from(format!("{}:\\", letter)))
                .collect()
        }
    }
}

/// macOS 平台：扫描常见目录查找 Zotero 数据库
/// 简化：提取公共扫描逻辑，与 Linux 共用
#[cfg(target_os = "macos")]
fn scan_macos_for_zotero_db() -> Option<PathBuf> {
    eprintln!("[Zotero路径检测] 开始 macOS 目录扫描...");

    let home = env::home_dir()?;
    let search_paths = vec![
        home.join("Zotero"),
        home.join(".zotero"),
        home.join("Library/Application Support/Zotero"),
        home.join("Library/Application Support/Zotero/Profiles"),
    ];

    scan_common_locations(search_paths)
}

/// Linux 平台：扫描常见目录查找 Zotero 数据库
/// 简化：提取公共扫描逻辑，与 macOS 共用
#[cfg(target_os = "linux")]
fn scan_linux_for_zotero_db() -> Option<PathBuf> {
    eprintln!("[Zotero路径检测] 开始 Linux 目录扫描...");

    let home = env::home_dir()?;
    let search_paths = vec![
        home.join(".zotero"),
        home.join("Zotero"),
        home.join(".local/share/Zotero"),
        home.join(".local/share/zotero"),
    ];

    scan_common_locations(search_paths)
}

/// 公共目录扫描逻辑：收集多个路径下的候选数据库
#[allow(dead_code)]
fn scan_common_locations(search_paths: Vec<PathBuf>) -> Option<PathBuf> {
    let candidates: Vec<(PathBuf, SystemTime)> = search_paths
        .iter()
        .filter_map(|search_path| {
            eprintln!("[Zotero路径检测] 扫描路径: {:?}", search_path);
            scan_directory_for_zotero_db(search_path, 0)
        })
        .collect();

    select_best_candidate(candidates)
}

/// 在指定目录中递归搜索 Zotero 数据库（简化：使用早期返回减少嵌套）
///
/// # 参数
/// * `dir` - 要搜索的目录
/// * `depth` - 当前递归深度
///
/// # 返回值
/// * `Option<(PathBuf, SystemTime)>` - 找到则返回 (数据库路径, 修改时间)
fn scan_directory_for_zotero_db(dir: &Path, depth: usize) -> Option<(PathBuf, SystemTime)> {
    // 超过最大深度限制，不再继续递归
    if depth > MAX_SEARCH_DEPTH {
        return None;
    }

    // 检查目录是否存在且可读（跳过无法访问的目录）
    let metadata = fs::metadata(dir).ok()?;
    if !metadata.is_dir() {
        return None;
    }

    // 检查当前目录下是否有 zotero.sqlite
    let db_path = dir.join(ZOTERO_DB_NAME);
    if db_path.is_file() {
        let db_meta = fs::metadata(&db_path).ok()?;
        let modified = db_meta.modified().ok()?;
        eprintln!(
            "[Zotero路径检测] 在 {:?} 找到数据库，修改时间: {:?}",
            db_path, modified
        );
        return Some((db_path, modified));
    }

    // 递归扫描子目录（限制深度）
    if depth >= MAX_SEARCH_DEPTH {
        return None;
    }

    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        // 只处理目录，跳过系统目录
        if let Ok(meta) = fs::metadata(&path) {
            if meta.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if should_skip_directory(dir_name) {
                    continue;
                }
                // 递归搜索，找到则立即返回
                if let Some(result) = scan_directory_for_zotero_db(&path, depth + 1) {
                    return Some(result);
                }
            }
        }
    }

    None
}

/// 判断目录是否应该跳过（避免扫描系统目录，提升性能）
fn should_skip_directory(name: &str) -> bool {
    // 跳过常见的系统目录和不需要扫描的目录
    let skip_dirs = [
        "node_modules",
        ".git",
        ".svn",
        "__pycache__",
        "target",
        "build",
        ".gradle",
        ".idea",
        ".vscode",
        "Windows",
        "Program Files",
        "Program Files (x86)",
        "ProgramData",
        "Recovery",
        "$Recycle.Bin",
        "System Volume Information",
        "PerfLogs",
        "MSOCache",
        "Config.Msi",
    ];

    skip_dirs.contains(&name)
}

/// 从多个候选数据库中选择最佳的一个（修改时间最新的）
/// 简化：使用 max_by 替代全排序，降低时间复杂度 O(n log n) -> O(n)
fn select_best_candidate(candidates: Vec<(PathBuf, SystemTime)>) -> Option<PathBuf> {
    // 使用 max_by 直接找最大元素，避免全排序
    candidates
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1))
        .map(|(path, _)| {
            eprintln!("[Zotero路径检测] 选择最新数据库: {:?}", path);
            path
        })
}

/// 检查 Zotero 数据库文件是否存在
///
/// # 返回值
/// * `bool` - 数据库文件是否存在
pub fn zotero_db_exists() -> bool {
    get_zotero_database_path()
        .map(|p| p.exists())
        .unwrap_or(false)
}
