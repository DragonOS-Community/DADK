use crate::{console::rootfs::RootFSCommand, context::DADKExecContext};
use anyhow::Result;
use disk_img::set_builder_version;

pub mod disk_img;
mod loopdev_v1;
mod loopdev_v2;
mod sysroot;
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuilderVersion {
    V1,
    V2,
}
impl BuilderVersion {
    fn from_str(version: &str) -> Self {
        match version {
            "v2" => BuilderVersion::V2,
            "v1" | _ => BuilderVersion::V1,
            
        }
    }
}
pub fn run(ctx: &DADKExecContext, rootfs_cmd: &RootFSCommand) -> Result<()> {
    set_builder_version(ctx);
    match rootfs_cmd {
        RootFSCommand::Create(param) => disk_img::create(ctx, param.skip_if_exists),
        RootFSCommand::Delete => disk_img::delete(ctx, false),
        RootFSCommand::DeleteSysroot => sysroot::delete(ctx),
        RootFSCommand::Mount => disk_img::mount(ctx),
        RootFSCommand::Umount => disk_img::umount(ctx),
        RootFSCommand::CheckDiskImageExists => disk_img::check_disk_image_exists(ctx),
        RootFSCommand::ShowMountPoint => disk_img::show_mount_point(ctx),
        RootFSCommand::ShowLoopDevice => disk_img::show_loop_device(ctx),
    }
}