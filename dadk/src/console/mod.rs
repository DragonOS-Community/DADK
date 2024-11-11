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
    /// 内核相关操作
    Kernel,
    /// 对 rootfs 进行操作
    #[command(subcommand, name = "rootfs")]
    Rootfs(RootFSCommand),
    /// 用户程序构建相关操作
    #[command(subcommand, name = "user")]
    User(UserCommand),
}
