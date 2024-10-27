use crate::context::DADKExecContext;

pub mod user;

pub fn run(ctx: DADKExecContext) {
    match &ctx.command.action {
        crate::console::Action::Kernel => {
            unimplemented!("kernel command has not implemented for run yet.")
        }
        crate::console::Action::Rootfs(_rootfs_command) => {
            unimplemented!("rootfs command has not implemented for run yet.")
        }
        crate::console::Action::User(user_command) => {
            user::run(&ctx, user_command).expect("Run user action error.")
        }
    }
}
