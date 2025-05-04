use std::path::{Path, PathBuf};

/// 获取给定路径的绝对路径
pub fn abs_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap().join(path)
    }
}
