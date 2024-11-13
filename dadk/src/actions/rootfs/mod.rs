use crate::{console::rootfs::RootFSCommand, context::DADKExecContext};
use anyhow::Result;

mod disk_img;
mod loopdev;

pub(super) fn run(ctx: &DADKExecContext, rootfs_cmd: &RootFSCommand) -> Result<()> {
    match rootfs_cmd {
        RootFSCommand::Create => disk_img::create(ctx),
        RootFSCommand::Delete => todo!(),
        RootFSCommand::DeleteSysroot => todo!(),
        RootFSCommand::Mount => disk_img::mount(ctx),
        RootFSCommand::Umount => disk_img::umount(ctx),
    }
}
