use crate::{console::rootfs::RootFSCommand, context::DADKExecContext};
use anyhow::{anyhow, Result};
use dadk_config::rootfs::RootFSConfigFile;

mod disk_img;
mod fat;

pub(super) fn run(ctx: &DADKExecContext, rootfs_cmd: &RootFSCommand) -> Result<()> {
    match rootfs_cmd{
        RootFSCommand::Create => disk_img::create(ctx),
        RootFSCommand::Delete => todo!(),
        RootFSCommand::DeleteSysroot => todo!(),
        RootFSCommand::Mount => todo!(),
        RootFSCommand::Unmount => todo!(),
    }
}

trait FileSystemCallback {
    fn create(&self, ctx: &DADKExecContext, config: &RootFSConfigFile) -> Result<()>;
}
