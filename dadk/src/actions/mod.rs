use crate::context::DADKExecContext;

pub mod profile;
pub mod rootfs;
pub mod user;

pub fn run(ctx: DADKExecContext) {
    match &ctx.command.action {
        crate::console::Action::Kernel => {
            unimplemented!("kernel command has not implemented for run yet.")
        }
        crate::console::Action::Rootfs(rootfs_command) => {
            match ctx.manifest().metadata.builder_version.as_str() {
                "v2" => {
                    // v2版本的rootfs命令
                    rootfs::v2::run(&ctx, rootfs_command).expect("Run rootfs action error.")
                }
                "v1" | _ => {
                    // v1版本的rootfs命令
                    rootfs::v1::run(&ctx, rootfs_command).expect("Run rootfs action error.")
                }
            }
        }
        crate::console::Action::User(user_command) => {
            user::run(&ctx, user_command).expect("Run user action error.")
        }
        crate::console::Action::Profile(profile_command) => {
            profile::run(&ctx, profile_command).expect("Run profile action error.")
        }
    }
}
