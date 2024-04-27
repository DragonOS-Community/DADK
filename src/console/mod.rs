//! # DADK控制台
//!
//! DADK控制台能够让用户通过命令行交互的方式使用DADK。
//!
//! ## 创建配置文件
//!
//! DADK控制台提供了一个命令，用于创建一个配置文件。您可以通过以下命令创建一个配置文件：
//!
//! ```bash
//! dadk new
//! ```
//!

pub mod clean;
pub mod elements;
pub mod interactive;
pub mod new_config;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::parser::task::TargetArch;

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

    /// DADK任务并行线程数量
    #[arg(short, long)]
    pub thread: Option<usize>,

    /// 目标架构，可选： ["aarch64", "x86_64", "riscv64", "riscv32"]
    #[arg(long, value_parser = parse_target_arch)]
    pub target_arch: Option<TargetArch>,
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

fn parse_target_arch(s: &str) -> Result<TargetArch, String> {
    let x = TargetArch::try_from(s);
    if x.is_err() {
        return Err(format!("Invalid target arch: {}", s));
    }
    return Ok(x.unwrap());
}

/// @brief 要执行的操作
#[derive(Debug, Subcommand, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// 构建所有项目
    Build,
    /// 清理缓存
    Clean(CleanArg),
    /// 安装到DragonOS sysroot
    Install,
    /// 尚不支持
    Uninstall,
    /// 使用交互式命令行创建dadk任务配置文件
    New,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ConsoleError {
    CommandError(String),
    IOError(std::io::Error),
    /// 错误次数超过限制
    RetryLimitExceeded(String),
    /// 无效的输入
    InvalidInput(String),
}
