use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

/// 检查目录是否存在
pub(super) fn check_dir_exists(path: &PathBuf) -> Result<&PathBuf> {
    if !path.exists() {
        return Err(anyhow!("Path '{}' not exists", path.display()));
    }
    if !path.is_dir() {
        return Err(anyhow!("Path '{}' is not a directory", path.display()));
    }

    Ok(path)
}

/// 获取给定路径的绝对路径
pub fn abs_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        let origin = std::env::current_dir().unwrap().join(path);
        origin.canonicalize().unwrap_or(origin)
    }
}
