#[deny(clippy::all)]
pub mod common;
pub mod hypervisor;
pub mod manifest;
pub mod rootfs;
pub mod user;

extern crate anyhow;
