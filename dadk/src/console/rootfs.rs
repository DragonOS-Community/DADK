use clap::{Parser, ValueEnum};

// 定义一个枚举类型 RootFSCommand，表示根文件系统操作命令
#[derive(Debug, Parser, Clone, PartialEq, Eq, ValueEnum)]
pub enum RootFSCommand {
    /// 创建根文件系统（磁盘镜像）
    Create,
    /// 删除根文件系统（磁盘镜像）
    Delete,
    /// 删除系统根目录（sysroot文件夹）
    DeleteSysroot,
    /// 挂载根文件系统（磁盘镜像）
    Mount,
    /// 卸载根文件系统（磁盘镜像）
    Umount,
}
