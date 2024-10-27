use std::path::PathBuf;

use serde::de::Error;
use toml::Value;

use super::{
    task::{Dependency, TargetArch, TaskEnv},
    InnerParserError, ParserError,
};

// DADK用户配置关键字
pub(super) enum DADKUserConfigKey {
    Name,
    Version,
    Description,
    BuildOnce,
    InstallOnce,
    RustTarget,
    TargetArch,
    TaskType,
    Type,
    Source,
    SourcePath,
    Revision,
    Branch,
    Build,
    BuildCommand,
    Install,
    InDragonosPath,
    Clean,
    CleanCommand,
    Depends,
    Envs,
    BuildFromSource,
    InstallFromPrebuilt,
    Git,
    Local,
    Archive,
}

impl Into<&str> for DADKUserConfigKey {
    fn into(self) -> &'static str {
        match self {
            DADKUserConfigKey::Name => "name",
            DADKUserConfigKey::Version => "version",
            DADKUserConfigKey::Description => "description",
            DADKUserConfigKey::BuildOnce => "build-once",
            DADKUserConfigKey::InstallOnce => "install-once",
            DADKUserConfigKey::RustTarget => "rust-target",
            DADKUserConfigKey::TargetArch => "target-arch",
            DADKUserConfigKey::TaskType => "task-type",
            DADKUserConfigKey::Type => "type",
            DADKUserConfigKey::Source => "source",
            DADKUserConfigKey::SourcePath => "source-path",
            DADKUserConfigKey::Revision => "revison",
            DADKUserConfigKey::Branch => "branch",
            DADKUserConfigKey::Build => "build",
            DADKUserConfigKey::BuildCommand => "build-command",
            DADKUserConfigKey::Install => "install",
            DADKUserConfigKey::InDragonosPath => "in-dragonos-path",
            DADKUserConfigKey::Clean => "clean",
            DADKUserConfigKey::CleanCommand => "clean-command",
            DADKUserConfigKey::Depends => "depends",
            DADKUserConfigKey::Envs => "envs",
            DADKUserConfigKey::BuildFromSource => "build_from_source",
            DADKUserConfigKey::InstallFromPrebuilt => "install_from_prebuilt",
            DADKUserConfigKey::Archive => "archive",
            DADKUserConfigKey::Git => "git",
            DADKUserConfigKey::Local => "local",
        }
    }
}

impl TryFrom<&str> for DADKUserConfigKey {
    type Error = ParserError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "name" => Ok(DADKUserConfigKey::Name),
            "version" => Ok(DADKUserConfigKey::Version),
            "description" => Ok(DADKUserConfigKey::Description),
            "build-once" => Ok(DADKUserConfigKey::BuildOnce),
            "install-once" => Ok(DADKUserConfigKey::InstallOnce),
            "rust-target" => Ok(DADKUserConfigKey::RustTarget),
            "target-arch" => Ok(DADKUserConfigKey::TargetArch),
            "task-type" => Ok(DADKUserConfigKey::TaskType),
            "type" => Ok(DADKUserConfigKey::Type),
            "source" => Ok(DADKUserConfigKey::Source),
            "source-path" => Ok(DADKUserConfigKey::SourcePath),
            "revison" => Ok(DADKUserConfigKey::Revision),
            "branch" => Ok(DADKUserConfigKey::Branch),
            "build" => Ok(DADKUserConfigKey::Build),
            "build-command" => Ok(DADKUserConfigKey::BuildCommand),
            "install" => Ok(DADKUserConfigKey::Install),
            "in-dragonos-path" => Ok(DADKUserConfigKey::InDragonosPath),
            "clean" => Ok(DADKUserConfigKey::Clean),
            "clean-command" => Ok(DADKUserConfigKey::CleanCommand),
            "depends" => Ok(DADKUserConfigKey::Depends),
            "envs" => Ok(DADKUserConfigKey::Envs),
            "build_from_source" => Ok(DADKUserConfigKey::BuildFromSource),
            "install_from_prebuilt" => Ok(DADKUserConfigKey::InstallFromPrebuilt),
            "archive" => Ok(DADKUserConfigKey::Archive),
            "git" => Ok(DADKUserConfigKey::Git),
            "local" => Ok(DADKUserConfigKey::Local),
            _ => Err(ParserError {
                config_file: None,
                error: InnerParserError::TomlError(toml::de::Error::custom(format!(
                    "Unknown dadk_user_config_key: {}",
                    value
                ))),
            }),
        }
    }
}

pub(super) struct DADKUserConfig {
    pub(super) standard_config: DADKUserStandardConfig,
    pub(super) task_type_config: DADKUserTaskType,
    pub(super) build_config: DADKUserBuildConfig,
    pub(super) install_config: DADKUserInstallConfig,
    pub(super) clean_config: DADKUserCleanConfig,
    pub(super) depends_config: DADKUserDependsConfig,
    pub(super) envs_config: DADKUserEnvsConfig,
}

impl DADKUserConfig {
    pub(super) fn parse(
        config_file: &PathBuf,
        table: &toml::value::Table,
    ) -> Result<DADKUserConfig, ParserError> {
        Ok(Self {
            standard_config: DADKUserStandardConfig::parse(config_file, table)?,
            task_type_config: DADKUserTaskType::parse(config_file, table)?,
            build_config: DADKUserBuildConfig::parse(config_file, table)?,
            install_config: DADKUserInstallConfig::parse(config_file, table)?,
            clean_config: DADKUserCleanConfig::parse(config_file, table)?,
            depends_config: DADKUserDependsConfig::parse(config_file, table)?,
            envs_config: DADKUserEnvsConfig::parse(config_file, table)?,
        })
    }
}

/// 标准信息配置
#[derive(Debug)]
pub(super) struct DADKUserStandardConfig {
    pub(super) name: String,
    pub(super) version: String,
    pub(super) description: String,
    pub(super) build_once: bool,
    pub(super) install_once: bool,
    pub(super) rust_target: Option<String>,
    pub(super) target_arch: Vec<TargetArch>,
}

impl DADKUserStandardConfig {
    fn parse(
        config_file: &PathBuf,
        table: &toml::value::Table,
    ) -> Result<DADKUserStandardConfig, ParserError> {
        let name: String =
            TomlValueParser::parse_string(config_file, table, DADKUserConfigKey::Name.into())?;
        let version =
            TomlValueParser::parse_string(config_file, table, DADKUserConfigKey::Version.into())?;
        let description = TomlValueParser::parse_string(
            config_file,
            table,
            DADKUserConfigKey::Description.into(),
        )?;
        let build_once =
            TomlValueParser::parse_bool(config_file, table, DADKUserConfigKey::BuildOnce.into())?;
        let install_once =
            TomlValueParser::parse_bool(config_file, table, DADKUserConfigKey::InstallOnce.into())?;
        let rust_target =
            TomlValueParser::parse_option_string(table, DADKUserConfigKey::RustTarget.into());
        let target_arch: Vec<TargetArch> = match TomlValueParser::parse_option_array(
            table,
            DADKUserConfigKey::TargetArch.into(),
        ) {
            Some(value_vec) => {
                let mut target_arch_vec = Vec::new();
                for value in value_vec {
                    let target_arch =
                        TargetArch::try_from(value.to_string().as_str()).map_err(|e| {
                            ParserError {
                                config_file: None,
                                error: InnerParserError::TaskError(e),
                            }
                        })?;
                    target_arch_vec.push(target_arch);
                }
                target_arch_vec
            }
            None => vec![TargetArch::X86_64],
        };

        Ok(Self {
            name,
            version,
            description,
            build_once,
            install_once,
            rust_target,
            target_arch,
        })
    }
}

/// task-type配置
#[derive(Debug)]
pub(super) struct DADKUserTaskType {
    pub(super) config_file: PathBuf,
    pub(super) task_type: String,
    pub(super) source: String,
    pub(super) source_path: String,
    // git独有
    pub(super) revision: Option<String>,
    pub(super) branch: Option<String>,
}

impl DADKUserTaskType {
    fn parse(
        config_file: &PathBuf,
        table: &toml::value::Table,
    ) -> Result<DADKUserTaskType, ParserError> {
        let task_type_table =
            TomlValueParser::parse_table(config_file, table, DADKUserConfigKey::TaskType.into())?;
        let task_type = TomlValueParser::parse_string(
            config_file,
            &task_type_table,
            DADKUserConfigKey::Type.into(),
        )?;
        let source = TomlValueParser::parse_string(
            config_file,
            &task_type_table,
            DADKUserConfigKey::Source.into(),
        )?;

        let source_path = TomlValueParser::parse_string(
            config_file,
            &task_type_table,
            DADKUserConfigKey::SourcePath.into(),
        )?;

        let (branch, revision) =
            if source.to_lowercase().trim() == Into::<&str>::into(DADKUserConfigKey::Git) {
                let branch = TomlValueParser::parse_option_string(
                    &task_type_table,
                    DADKUserConfigKey::Branch.into(),
                );
                let revision = TomlValueParser::parse_option_string(
                    &task_type_table,
                    DADKUserConfigKey::Revision.into(),
                );
                (branch, revision)
            } else {
                (None, None)
            };

        Ok(Self {
            config_file: config_file.clone(),
            task_type,
            source,
            source_path,
            revision,
            branch,
        })
    }
}

/// build配置
#[derive(Debug)]
pub(super) struct DADKUserBuildConfig {
    pub(super) build_command: Option<String>,
}

impl DADKUserBuildConfig {
    fn parse(
        config_file: &PathBuf,
        table: &toml::value::Table,
    ) -> Result<DADKUserBuildConfig, ParserError> {
        let build_table =
            TomlValueParser::parse_table(config_file, table, DADKUserConfigKey::Build.into())?;
        let build_command = TomlValueParser::parse_option_string(
            &build_table,
            DADKUserConfigKey::BuildCommand.into(),
        );
        Ok(Self { build_command })
    }
}

/// install配置
#[derive(Debug)]
pub(super) struct DADKUserInstallConfig {
    pub(super) in_dragonos_path: Option<PathBuf>,
}

impl DADKUserInstallConfig {
    fn parse(
        config_file: &PathBuf,
        table: &toml::value::Table,
    ) -> Result<DADKUserInstallConfig, ParserError> {
        let install_table =
            TomlValueParser::parse_table(config_file, table, DADKUserConfigKey::Install.into())?;
        let in_dragonos_path = TomlValueParser::parse_option_string(
            &install_table,
            DADKUserConfigKey::InDragonosPath.into(),
        )
        .map(|path| PathBuf::from(path));

        Ok(Self { in_dragonos_path })
    }
}

/// clean配置
#[derive(Debug)]
pub(super) struct DADKUserCleanConfig {
    pub(super) clean_command: Option<String>,
}

impl DADKUserCleanConfig {
    fn parse(
        config_file: &PathBuf,
        table: &toml::value::Table,
    ) -> Result<DADKUserCleanConfig, ParserError> {
        let clean_table =
            TomlValueParser::parse_table(config_file, table, DADKUserConfigKey::Clean.into())?;
        let clean_command = TomlValueParser::parse_option_string(
            &clean_table,
            DADKUserConfigKey::CleanCommand.into(),
        );
        Ok(Self { clean_command })
    }
}

/// depends配置
#[derive(Debug)]
pub(super) struct DADKUserDependsConfig {
    pub(super) depends: Vec<Dependency>,
}

impl DADKUserDependsConfig {
    fn parse(
        config_file: &PathBuf,
        table: &toml::value::Table,
    ) -> Result<DADKUserDependsConfig, ParserError> {
        let depends_table =
            TomlValueParser::parse_table(config_file, table, DADKUserConfigKey::Depends.into())?;
        let depends = depends_table
            .iter()
            .map(|(key, value)| Dependency {
                name: key.clone(),
                version: value.to_string(),
            })
            .collect::<Vec<Dependency>>();
        Ok(Self { depends })
    }
}
/// envs配置
#[derive(Debug)]
pub(super) struct DADKUserEnvsConfig {
    pub(super) envs: Option<Vec<TaskEnv>>,
}

impl DADKUserEnvsConfig {
    fn parse(
        config_file: &PathBuf,
        table: &toml::value::Table,
    ) -> Result<DADKUserEnvsConfig, ParserError> {
        let envs_table: toml::map::Map<String, Value> =
            TomlValueParser::parse_table(config_file, table, DADKUserConfigKey::Envs.into())?;
        let envs_vec = if !envs_table.is_empty() {
            Some(
                envs_table
                    .iter()
                    .map(|(key, value)| TaskEnv {
                        key: key.clone(),
                        value: value.to_string(),
                    })
                    .collect::<Vec<TaskEnv>>(),
            )
        } else {
            None
        };

        Ok(DADKUserEnvsConfig { envs: envs_vec })
    }
}

struct TomlValueParser;

impl TomlValueParser {
    // 解析String类型的值
    fn parse_string(
        config_file: &PathBuf,
        table: &toml::value::Table,
        key: &'static str,
    ) -> Result<String, ParserError> {
        let value = table.get(key).ok_or(ParserError {
            config_file: Some(config_file.clone()),
            error: InnerParserError::TomlError(toml::de::Error::missing_field(key)),
        })?;
        Ok(value.to_string())
    }

    // 解析Option<String>类型的值
    fn parse_option_string(table: &toml::value::Table, key: &'static str) -> Option<String> {
        let value = table.get(key);
        value.map(|v| v.to_string())
    }

    // 解析Table类型的值
    fn parse_table(
        config_file: &PathBuf,
        table: &toml::value::Table,
        key: &'static str,
    ) -> Result<toml::value::Table, ParserError> {
        let value = table.get(key).ok_or(ParserError {
            config_file: Some(config_file.clone()),
            error: InnerParserError::TomlError(toml::de::Error::missing_field(key)),
        })?;
        let table = value.as_table().ok_or(ParserError {
            config_file: Some(config_file.clone()),
            error: InnerParserError::TomlError(toml::de::Error::custom(format!(
                "{} is not a table",
                key
            ))),
        })?;
        Ok(table.clone())
    }

    #[allow(dead_code)]
    // 解析Array类型的值
    fn parse_array(
        config_file: &PathBuf,
        table: &toml::value::Table,
        key: &'static str,
    ) -> Result<Vec<toml::Value>, ParserError> {
        let value = table.get(key).ok_or(ParserError {
            config_file: Some(config_file.clone()),
            error: InnerParserError::TomlError(toml::de::Error::missing_field(key)),
        })?;
        let array = value.as_array().ok_or(ParserError {
            config_file: Some(config_file.clone()),
            error: InnerParserError::TomlError(toml::de::Error::custom(format!(
                "{} is not an array",
                key
            ))),
        })?;
        Ok(array.clone())
    }

    // 解析Option<Array>类型的值
    fn parse_option_array(
        table: &toml::value::Table,
        key: &'static str,
    ) -> Option<Vec<toml::Value>> {
        let value = table.get(key);
        value.map(|v| v.as_array().unwrap().clone())
    }

    // 解析Boolean类型的值
    fn parse_bool(
        config_file: &PathBuf,
        table: &toml::value::Table,
        key: &'static str,
    ) -> Result<bool, ParserError> {
        let value = table.get(key).ok_or(ParserError {
            config_file: Some(config_file.clone()),
            error: InnerParserError::TomlError(toml::de::Error::missing_field(key)),
        })?;
        let boolean = value.as_bool().ok_or(ParserError {
            config_file: Some(config_file.clone()),
            error: InnerParserError::TomlError(toml::de::Error::custom(format!(
                "{} is not a boolean",
                key
            ))),
        })?;
        Ok(boolean)
    }
}
