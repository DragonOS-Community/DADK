use std::{cell::OnceCell, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use dadk_config::{
    common::target_arch::TargetArch, manifest::DadkManifestFile, rootfs::RootFSConfigFile,
};
use derive_builder::Builder;
use manifest::parse_manifest;

use crate::{
    console::CommandLineArgs,
    utils::{abs_path, check_dir_exists},
};

mod manifest;

/// DADK的执行上下文
#[derive(Debug, Clone, Builder)]
pub struct DADKExecContext {
    pub command: CommandLineArgs,
    /// DADK manifest file
    pub manifest: DadkManifestFile,

    /// RootFS config file
    rootfs: OnceCell<RootFSConfigFile>,
}

pub fn build_exec_context() -> Result<DADKExecContext> {
    let mut builder = DADKExecContextBuilder::create_empty();
    builder.command(CommandLineArgs::parse());
    builder.rootfs(OnceCell::new());
    parse_manifest(&mut builder).expect("Failed to parse manifest");
    let ctx = builder.build()?;
    ctx.setup_workdir().expect("Failed to setup workdir");
    Ok(ctx)
}

impl DADKExecContext {
    /// 获取工作目录的绝对路径
    pub fn workdir(&self) -> PathBuf {
        abs_path(&PathBuf::from(&self.command.workdir))
    }

    /// 设置进程的工作目录
    fn setup_workdir(&self) -> Result<()> {
        std::env::set_current_dir(&self.workdir()).expect("Failed to set current directory");
        Ok(())
    }
    /// Get rootfs configuration
    pub fn rootfs(&self) -> &RootFSConfigFile {
        self.rootfs.get_or_init(|| {
            RootFSConfigFile::load(&self.manifest.metadata.rootfs_config)
                .expect("Failed to load rootfs config")
        })
    }

    /// Get sysroot directory
    ///
    /// If the directory does not exist, or the path is not a folder, an error is returned
    pub fn sysroot_dir(&self) -> Result<PathBuf> {
        check_dir_exists(&self.manifest.metadata.sysroot_dir)
            .map(|p| p.clone())
            .map_err(|e| anyhow::anyhow!("Failed to get sysroot dir: {}", e))
    }

    /// Get cache root directory
    ///
    /// If the directory does not exist, or the path is not a folder, an error is returned
    pub fn cache_root_dir(&self) -> Result<PathBuf> {
        check_dir_exists(&self.manifest.metadata.cache_root_dir)
            .map(|p| p.clone())
            .map_err(|e| anyhow::anyhow!("Failed to get cache root dir: {}", e))
    }

    #[deprecated]
    pub fn user_config_dir(&self) -> Result<PathBuf> {
        check_dir_exists(&self.manifest.metadata.user_config_dir)
            .map(|p| p.clone())
            .map_err(|e| anyhow::anyhow!("Failed to get user config dir: {}", e))
    }

    pub fn target_arch(&self) -> TargetArch {
        self.manifest.metadata.arch
    }
}
