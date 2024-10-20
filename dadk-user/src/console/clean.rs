use std::{fmt::Display, str::FromStr};

use clap::{Args, Subcommand};

/// 清理缓存的级别
#[derive(Debug, Args, Clone, Copy, PartialEq, Eq)]
pub struct CleanArg {
    #[arg(default_value = "src")]
    /// 清理缓存的级别
    ///
    /// all：清理所有缓存
    ///
    /// src：在源码目录内运行clean命令
    ///
    /// target：清理DADK输出目录
    ///
    /// cache：清理DADK缓存目录（下载的源码、编译好的库等）
    pub level: CleanLevel,
}

#[derive(Debug, Subcommand, Clone, Copy, PartialEq, Eq)]
pub enum CleanLevel {
    /// 清理所有缓存
    All,
    /// 在源码目录内运行clean命令
    Src,
    /// 清理DADK输出目录
    Target,
    /// 清理DADK缓存目录（下载的源码、编译好的库等）
    Cache,
}

impl FromStr for CleanLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "all" => Ok(CleanLevel::All),
            "src" => Ok(CleanLevel::Src),
            "target" => Ok(CleanLevel::Target),
            "cache" => Ok(CleanLevel::Cache),
            _ => Err(format!("Unknown clean level: {}", s)),
        }
    }
}

impl Display for CleanLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CleanLevel::All => write!(f, "all"),
            CleanLevel::Src => write!(f, "src"),
            CleanLevel::Target => write!(f, "target"),
            CleanLevel::Cache => write!(f, "cache"),
        }
    }
}
