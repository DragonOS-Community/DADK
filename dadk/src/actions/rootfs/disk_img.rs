use crate::context::DADKExecContext;
use anyhow::{anyhow, Result};
pub(super) fn create(ctx: &DADKExecContext) -> Result<()> {
    // 判断是否需要分区？
    if ctx.rootfs().partition.should_create_partitioned_image() {
        return create_partitioned_image(ctx);
    } else {
        return create_unpartitioned_image(ctx);
    }
}

fn create_partitioned_image(ctx: &DADKExecContext) -> Result<()> {
    unimplemented!("Not implemented: create_partitioned_image")
}

fn create_unpartitioned_image(ctx: &DADKExecContext) -> Result<()> {
    unimplemented!("Not implemented: create_unpartitioned_image")
}
