use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

use crate::common::target_arch::TargetArch;

use std::fs;
use toml;

#[derive(Debug, Clone, Deserialize)]
pub struct Metadata {
    /// Target processor architecture
    pub arch: TargetArch,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DadkManifest {
    pub metadata: Metadata,
}

impl DadkManifest {
    pub fn load(path: &PathBuf) -> Result<Self> {
        // 读取文件内容
        let content = fs::read_to_string(path)?;
        Self::do_load(&content)
    }

    fn do_load(content: &str) -> Result<Self> {
        // 解析TOML内容
        let manifest_toml: DadkManifest = toml::from_str(&content)?;

        Ok(manifest_toml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_success() -> Result<()> {
        let toml_content = r#"
            [metadata]
            arch = "x86_64"
        "#;

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(toml_content.as_bytes())?;

        let path = temp_file.path().to_path_buf();
        let manifest = DadkManifest::load(&path)?;

        assert_eq!(manifest.metadata.arch, TargetArch::X86_64);

        Ok(())
    }

    #[test]
    fn test_load_file_not_found() {
        let path = PathBuf::from("non_existent_file.toml");
        let result = DadkManifest::load(&path);

        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_toml() -> Result<()> {
        let invalid_toml_content = r#"
            [metadata
            arch = "x86_64"
        "#;

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(invalid_toml_content.as_bytes())?;

        let path = temp_file.path().to_path_buf();
        let result = DadkManifest::load(&path);

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_load_invalid_arch_toml() -> Result<()> {
        // Invalid arch value
        let invalid_toml_content = r#"
            [metadata]
            arch = "abcde"
        "#;

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(invalid_toml_content.as_bytes())?;

        let path = temp_file.path().to_path_buf();
        let result = DadkManifest::load(&path);

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_load_missing_required_fields() -> Result<()> {
        let toml_content = r#"
            [metadata]
            # arch field is missing
        "#;

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(toml_content.as_bytes())?;

        let path = temp_file.path().to_path_buf();
        let result = DadkManifest::load(&path);

        assert!(result.is_err());

        Ok(())
    }
}
