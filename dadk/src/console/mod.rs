use clap::{Parser, Subcommand};
use rootfs::RootFSCommand;
use user::UserCommand;

pub mod rootfs;
#[cfg(test)]
mod tests;
pub mod user;

#[derive(Debug, Parser, Clone)]
#[command(author, version, about)]
pub struct CommandLineArgs {
    /// 要执行的操作
    #[command(subcommand)]
    pub action: Action,

    /// dadk manifest 配置文件的路径
    #[arg(
        short = 'f',
        long = "manifest",
        default_value = "dadk-manifest.toml",
        global = true
    )]
    pub manifest_path: String,

    /// DADK 的工作目录
    #[arg(short = 'w', long = "workdir", default_value = ".", global = true)]
    pub workdir: String,
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum Action {
    Kernel,
    #[command(subcommand, name = "rootfs")]
    Rootfs(RootFSCommand),
    #[command(subcommand, name = "user")]
    User(UserCommand),
}
