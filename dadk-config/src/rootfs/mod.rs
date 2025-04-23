pub mod fstype;
pub mod partition;

mod utils;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use fstype::FsType;
use partition::PartitionConfig;
use serde::Deserialize;

/// rootfs配置文件
#[derive(Debug, Clone, Deserialize)]
pub struct RootFSConfigFile {
    pub metadata: RootFSMeta,
    #[serde(default)]
    pub partition: PartitionConfig,
}

impl RootFSConfigFile {
    pub const LBA_SIZE: usize = 512;
    pub fn load(path: &Path) -> Result<Self> {
        // 读取文件内容
        let content = fs::read_to_string(path)?;
        Self::load_from_str(&content)
    }

    pub fn load_from_str(content: &str) -> Result<Self> {
        let config: RootFSConfigFile = toml::from_str(content)?;

        Ok(config)
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct RootFSMeta {
    /// rootfs文件系统类型
    pub fs_type: FsType,
    /// rootfs磁盘大小（至少要大于这个值）
    /// 单位：字节
    #[serde(deserialize_with = "utils::size::deserialize_size")]
    pub size: usize,
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_from_valid_file() {
        let config_content = r#"
            [metadata]
            fs_type = "fat32"
            size = "1024M"
        "#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(config_content.as_bytes())
            .expect("Failed to write to temp file");

        let config_path = PathBuf::from(temp_file.path());
        let config = RootFSConfigFile::load(&config_path).expect("Failed to load config");

        assert_eq!(config.metadata.fs_type, FsType::Fat32);
        assert_eq!(config.metadata.size, 1024 * 1024 * 1024); // Assuming `deserialize_size` converts MB to Bytes
    }

    #[test]
    fn test_load_from_valid_str() {
        let config_content = r#"
            [metadata]
            fs_type = "fat32"
            size = "512M"
        "#;

        let config = RootFSConfigFile::load_from_str(config_content)
            .expect("Failed to load config from str");

        assert_eq!(config.metadata.fs_type, FsType::Fat32);
        assert_eq!(config.metadata.size, 512 * 1024 * 1024); // Assuming `deserialize_size` converts MB to Bytes
    }
    #[test]
    fn test_load_from_invalid_fs_type() {
        let config_content = r#"
            [metadata]
            fs_type = "ABCDE"
            size = "512M"
        "#;
        assert!(RootFSConfigFile::load_from_str(config_content).is_err());
    }

    /// 测试size为int类型的字节大小
    #[test]
    fn test_load_from_valid_str_size_integer() {
        let config_content = r#"
            [metadata]
            fs_type = "fat32"
            size = 1048576
        "#;

        let config = RootFSConfigFile::load_from_str(config_content)
            .expect("Failed to load config from str");

        assert_eq!(config.metadata.fs_type, FsType::Fat32);
        assert_eq!(config.metadata.size, 1048576); // Assuming `deserialize_size` converts MB to Bytes
    }
    #[test]
    fn test_load_from_valid_str_size_bytes_str() {
        let config_content = r#"
            [metadata]
            fs_type = "fat32"
            size = "1048576"
        "#;

        let config = RootFSConfigFile::load_from_str(config_content)
            .expect("Failed to load config from str");

        assert_eq!(config.metadata.fs_type, FsType::Fat32);
        assert_eq!(config.metadata.size, 1048576); // Assuming `deserialize_size` converts MB to Bytes
    }

    #[test]
    fn test_load_from_invalid_file() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config_path = PathBuf::from(temp_file.path());

        assert!(RootFSConfigFile::load(&config_path).is_err());
    }

    /// Parse from an incorrect size field (string)
    #[test]
    fn test_load_from_invalid_size_str() {
        let invalid_config_content = r#"
            [metadata]
            fs_type = "fat32"
            size = "not_a_size"
        "#;

        assert!(RootFSConfigFile::load_from_str(invalid_config_content).is_err());
    }

    /// Parse from an incorrect size field (array)
    #[test]
    fn test_load_from_invalid_size_array() {
        // The 'size' field should not be an array
        let invalid_config_content = r#"
            [metadata]
            fs_type = "fat32"
            size = ["not_a_size"]
        "#;

        assert!(RootFSConfigFile::load_from_str(invalid_config_content).is_err());
    }
}
