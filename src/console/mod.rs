pub mod clean;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use self::clean::CleanArg;

#[derive(Debug, Parser, Clone)]
#[command(author, version, about)]
pub struct CommandLineArgs {
    /// DragonOS sysroot在主机上的路径
    #[arg(short, long, value_parser = parse_check_dir_exists)]
    pub dragonos_dir: Option<PathBuf>,
    /// DADK任务配置文件所在目录
    #[arg(short, long, value_parser = parse_check_dir_exists)]
    pub config_dir: Option<PathBuf>,

    /// 要执行的操作
    #[command(subcommand)]
    pub action: Action,

    /// DADK缓存根目录
    #[arg(long, value_parser = parse_check_dir_exists)]
    pub cache_dir: Option<PathBuf>,
}

/// @brief 检查目录是否存在
fn parse_check_dir_exists(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(format!("Path '{}' not exists", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Path '{}' is not a directory", path.display()));
    }

    return Ok(path);
}

/// @brief 要执行的操作
#[derive(Debug, Subcommand, Clone, Copy)]
pub enum Action {
    /// 构建所有项目
    Build,
    /// 清理缓存
    Clean(CleanArg),
    /// 安装到DragonOS sysroot
    Install,
    /// 尚不支持
    Uninstall,
}

#[derive(Debug, Clone)]
pub enum ConsoleError {}
