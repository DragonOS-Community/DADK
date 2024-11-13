use crate::{console::rootfs::RootFSCommand, context::DADKExecContext};
use anyhow::Result;

mod disk_img;
mod loopdev;
mod sysroot;

pub(super) fn run(ctx: &DADKExecContext, rootfs_cmd: &RootFSCommand) -> Result<()> {
    match rootfs_cmd {
        RootFSCommand::Create => disk_img::create(ctx, false),
        RootFSCommand::Delete => disk_img::delete(ctx, false),
        RootFSCommand::DeleteSysroot => sysroot::delete(ctx),
        RootFSCommand::Mount => disk_img::mount(ctx),
        RootFSCommand::Umount => disk_img::umount(ctx),
    }
}
