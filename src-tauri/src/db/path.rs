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

use std::path::{PathBuf, Path};
use std::env;
use std::fs;
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
    std::env::var("ZOTERO_DB_PATH")
        .ok()
        .map(PathBuf::from)
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
fn get_default_zotero_path() -> PathBuf {
    let home_dir = env::home_dir().expect("无法获取用户主目录");

    #[cfg(target_os = "windows")]
    let db_path = {
        let mut path = home_dir;
        path.push("Zotero");
        path.push(ZOTERO_DB_NAME);
        path
    };

    #[cfg(target_os = "macos")]
    let db_path = {
        let mut path = home_dir;
        path.push("Zotero");
        path.push(ZOTERO_DB_NAME);
        path
    };

    #[cfg(target_os = "linux")]
    let db_path = {
        let mut path = home_dir;
        path.push(".zotero");
        path.push(ZOTERO_DB_NAME);
        path
    };

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let db_path = {
        // 默认使用 Linux 路径作为后备方案
        let mut path = home_dir;
        path.push(".zotero");
        path.push(ZOTERO_DB_NAME);
        path
    };

    db_path
}

/// 全磁盘扫描，查找 Zotero 数据库
///
/// # 扫描策略
/// - Windows：扫描所有盘符，递归查找 zotero.sqlite
/// - macOS：扫描常见目录 ~/Zotero, ~/.zotero, ~/Library/Application Support/Zotero
/// - Linux：扫描常见目录 ~/.zotero, ~/.local/share/Zotero, ~/Zotero
///
/// # 返回值
/// * `Option<PathBuf>` - 找到则返回路径，否则返回 None
fn scan_for_zotero_db() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        scan_windows_for_zotero_db()
    }

    #[cfg(target_os = "macos")]
    {
        scan_macos_for_zotero_db()
    }

    #[cfg(target_os = "linux")]
    {
        scan_linux_for_zotero_db()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // 未知系统，扫描用户主目录
        if let Some(home) = env::home_dir() {
            scan_directory_for_zotero_db(&home, 0)
        } else {
            None
        }
    }
}

/// Windows 平台：扫描所有盘符查找 Zotero 数据库
#[cfg(target_os = "windows")]
fn scan_windows_for_zotero_db() -> Option<PathBuf> {
    eprintln!("[Zotero路径检测] 开始 Windows 全磁盘扫描...");

    // 获取所有可用的盘符
    let drives = get_windows_drives();
    eprintln!("[Zotero路径检测] 发现 {} 个盘符: {:?}", drives.len(), drives);

    let mut candidates: Vec<(PathBuf, SystemTime)> = Vec::new();

    for drive in drives {
        eprintln!("[Zotero路径检测] 扫描盘符: {}", drive.display());
        if let Some((path, modified)) = scan_directory_for_zotero_db(&drive, 0) {
            candidates.push((path, modified));
        }
    }

    // 选择修改时间最新的数据库
    select_best_candidate(candidates)
}

/// 获取 Windows 系统所有可用的盘符
#[cfg(target_os = "windows")]
fn get_windows_drives() -> Vec<PathBuf> {
    let mut drives = Vec::new();

    // 使用 wmic 获取逻辑磁盘列表
    match Command::new("wmic").args(["logicaldisk", "get", "name"]).output() {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            eprintln!("[Zotero路径检测] wmic 输出: {}", output_str);

            // 解析输出，每行是一个盘符
            for line in output_str.lines() {
                let line = line.trim();
                // 跳过表头和空行，盘符格式如 "C:"
                if line.len() >= 2 && line.chars().last().map_or(false, |c| c == ':') {
                    let drive_letter = &line[..2];
                    let drive_path = format!("{}\\", drive_letter);
                    drives.push(PathBuf::from(drive_path));
                }
            }
        }
        Err(e) => {
            eprintln!("[Zotero路径检测] wmic 执行失败: {}，使用常见盘符作为备选", e);
            // wmic 失败时使用常见盘符作为备选
            for letter in ['C', 'D', 'E', 'F', 'G', 'H'] {
                drives.push(PathBuf::from(format!("{}:\\", letter)));
            }
        }
    }

    drives
}

/// macOS 平台：扫描常见目录查找 Zotero 数据库
#[cfg(target_os = "macos")]
fn scan_macos_for_zotero_db() -> Option<PathBuf> {
    eprintln!("[Zotero路径检测] 开始 macOS 目录扫描...");

    let home = env::home_dir()?;
    let mut candidates: Vec<(PathBuf, SystemTime)> = Vec::new();

    // macOS 常见 Zotero 数据目录
    let search_paths = vec![
        home.join("Zotero"),
        home.join(".zotero"),
        home.join("Library/Application Support/Zotero"),
        home.join("Library/Application Support/Zotero/Profiles"),
    ];

    for search_path in search_paths {
        eprintln!("[Zotero路径检测] 扫描路径: {:?}", search_path);
        if let Some((path, modified)) = scan_directory_for_zotero_db(&search_path, 0) {
            candidates.push((path, modified));
        }
    }

    select_best_candidate(candidates)
}

/// Linux 平台：扫描常见目录查找 Zotero 数据库
#[cfg(target_os = "linux")]
fn scan_linux_for_zotero_db() -> Option<PathBuf> {
    eprintln!("[Zotero路径检测] 开始 Linux 目录扫描...");

    let home = env::home_dir()?;
    let mut candidates: Vec<(PathBuf, SystemTime)> = Vec::new();

    // Linux 常见 Zotero 数据目录
    let search_paths = vec![
        home.join(".zotero"),
        home.join("Zotero"),
        home.join(".local/share/Zotero"),
        home.join(".local/share/zotero"),
    ];

    for search_path in search_paths {
        eprintln!("[Zotero路径检测] 扫描路径: {:?}", search_path);
        if let Some((path, modified)) = scan_directory_for_zotero_db(&search_path, 0) {
            candidates.push((path, modified));
        }
    }

    select_best_candidate(candidates)
}

/// 在指定目录中递归搜索 Zotero 数据库
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

    // 检查目录是否存在且可读
    let metadata = match fs::metadata(dir) {
        Ok(m) => m,
        Err(_e) => {
            // 跳过无法访问的目录
            return None;
        }
    };

    // 确保是目录
    if !metadata.is_dir() {
        return None;
    }

    // 检查当前目录下是否有 zotero.sqlite
    let db_path = dir.join(ZOTERO_DB_NAME);
    if db_path.is_file() {
        // 获取文件修改时间
        if let Ok(db_meta) = fs::metadata(&db_path) {
            if let Ok(modified) = db_meta.modified() {
                eprintln!("[Zotero路径检测] 在 {:?} 找到数据库，修改时间: {:?}", db_path, modified);
                return Some((db_path, modified));
            }
        }
    }

    // 递归扫描子目录（限制深度）
    if depth < MAX_SEARCH_DEPTH {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                // 只处理目录
                if let Ok(meta) = fs::metadata(&path) {
                    if meta.is_dir() {
                        // 跳过系统目录，减少搜索范围
                        let dir_name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");

                        // 跳过常见的不需要扫描的目录
                        if should_skip_directory(dir_name) {
                            continue;
                        }

                        // 递归搜索
                        if let Some(result) = scan_directory_for_zotero_db(&path, depth + 1) {
                            return Some(result);
                        }
                    }
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
///
/// # 参数
/// * `candidates` - 候选数据库列表，每项为 (路径, 修改时间)
///
/// # 返回值
/// * `Option<PathBuf>` - 最佳数据库路径
fn select_best_candidate(candidates: Vec<(PathBuf, SystemTime)>) -> Option<PathBuf> {
    if candidates.is_empty() {
        return None;
    }

    // 按修改时间降序排序，选择最新的
    let mut sorted = candidates;
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let best = sorted.into_iter().next().unwrap().0;
    eprintln!("[Zotero路径检测] 选择最新数据库: {:?}", best);
    Some(best)
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