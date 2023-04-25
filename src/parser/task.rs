use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::executor::source::{ArchiveSource, GitSource, LocalSource};

// 对于生成的包名和版本号，需要进行替换的字符。
pub static NAME_VERSION_REPLACE_TABLE: [(&str, &str); 6] = [
    (" ", "_"),
    ("\t", "_"),
    ("-", "_"),
    (".", "_"),
    ("+", "_"),
    ("*", "_"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DADKTask {
    /// 包名
    pub name: String,
    /// 版本
    pub version: String,
    /// 包的描述
    pub description: String,
    /// 任务类型
    pub task_type: TaskType,
    /// 依赖的包
    pub depends: Vec<Dependency>,
    /// 构建配置
    pub build: BuildConfig,
    /// 安装配置
    pub install: InstallConfig,
    /// 清理配置
    pub clean: CleanConfig,
    /// 环境变量
    pub envs: Option<Vec<TaskEnv>>,
}

impl DADKTask {
    #[allow(dead_code)]
    pub fn new(
        name: String,
        version: String,
        description: String,
        task_type: TaskType,
        depends: Vec<Dependency>,
        build: BuildConfig,
        install: InstallConfig,
        clean: CleanConfig,
        envs: Option<Vec<TaskEnv>>,
    ) -> Self {
        Self {
            name,
            version,
            description,
            task_type,
            depends,
            build,
            install,
            clean,
            envs,
        }
    }

    pub fn validate(&mut self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("name is empty".to_string());
        }
        if self.version.is_empty() {
            return Err("version is empty".to_string());
        }
        self.task_type.validate()?;
        self.build.validate()?;
        self.validate_build_type()?;
        self.install.validate()?;
        self.clean.validate()?;
        self.validate_depends()?;
        self.validate_envs()?;

        return Ok(());
    }

    pub fn trim(&mut self) {
        self.name = self.name.trim().to_string();
        self.version = self.version.trim().to_string();
        self.description = self.description.trim().to_string();
        self.task_type.trim();
        self.build.trim();
        self.install.trim();
        self.clean.trim();
        self.trim_depends();
        self.trim_envs();
    }

    fn validate_depends(&self) -> Result<(), String> {
        for depend in &self.depends {
            depend.validate()?;
        }
        return Ok(());
    }

    fn trim_depends(&mut self) {
        for depend in &mut self.depends {
            depend.trim();
        }
    }

    fn validate_envs(&self) -> Result<(), String> {
        if let Some(envs) = &self.envs {
            for env in envs {
                env.validate()?;
            }
        }
        return Ok(());
    }

    fn trim_envs(&mut self) {
        if let Some(envs) = &mut self.envs {
            for env in envs {
                env.trim();
            }
        }
    }

    /// 验证任务类型与构建配置是否匹配
    fn validate_build_type(&self) -> Result<(), String> {
        match &self.task_type {
            TaskType::BuildFromSource(_) => {
                if self.build.build_command.is_none() {
                    return Err("build command is empty".to_string());
                }
            }
            TaskType::InstallFromPrebuilt(_) => {
                if self.build.build_command.is_some() {
                    return Err(
                        "build command should be empty when install from prebuilt".to_string()
                    );
                }
            }
        }
        return Ok(());
    }

    pub fn name_version(&self) -> String {
        return format!("{}-{}", self.name, self.version);
    }

    pub fn name_version_env(&self) -> String {
        return Self::name_version_uppercase(&self.name, &self.version);
    }

    pub fn name_version_uppercase(name: &str, version: &str) -> String {
        let mut name_version = format!("{}-{}", name, version).to_ascii_uppercase();
        for (src, dst) in &NAME_VERSION_REPLACE_TABLE {
            name_version = name_version.replace(src, dst);
        }
        return name_version;
    }

    /// # 获取源码目录
    ///
    /// 如果从本地路径构建，则返回本地路径。否则返回None。
    pub fn source_path(&self) -> Option<PathBuf> {
        match &self.task_type {
            TaskType::BuildFromSource(cs) => match cs {
                CodeSource::Local(lc) => {
                    return Some(lc.path().clone());
                }
                _ => {
                    return None;
                }
            },
            TaskType::InstallFromPrebuilt(ps) => match ps {
                PrebuiltSource::Local(lc) => {
                    return Some(lc.path().clone());
                }
                _ => {
                    return None;
                }
            },
        }
    }
}

/// @brief 构建配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// 构建命令
    pub build_command: Option<String>,
}

impl BuildConfig {
    #[allow(dead_code)]
    pub fn new(build_command: Option<String>) -> Self {
        Self { build_command }
    }

    pub fn validate(&self) -> Result<(), String> {
        return Ok(());
    }

    pub fn trim(&mut self) {
        if let Some(build_command) = &mut self.build_command {
            *build_command = build_command.trim().to_string();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// 安装到DragonOS内的目录
    pub in_dragonos_path: Option<PathBuf>,
}

impl InstallConfig {
    #[allow(dead_code)]
    pub fn new(in_dragonos_path: Option<PathBuf>) -> Self {
        Self { in_dragonos_path }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.in_dragonos_path.is_none() {
            return Ok(());
        }
        if self.in_dragonos_path.as_ref().unwrap().is_relative() {
            return Err("InstallConfig: in_dragonos_path should be an Absolute path".to_string());
        }
        return Ok(());
    }

    pub fn trim(&mut self) {
        if let Some(in_dragonos_path) = &mut self.in_dragonos_path {
            *in_dragonos_path = in_dragonos_path.canonicalize().unwrap();
        }
    }
}

/// # 清理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanConfig {
    /// 清理命令
    pub clean_command: Option<String>,
}

impl CleanConfig {
    #[allow(dead_code)]
    pub fn new(clean_command: Option<String>) -> Self {
        Self { clean_command }
    }

    pub fn validate(&self) -> Result<(), String> {
        return Ok(());
    }

    pub fn trim(&mut self) {
        if let Some(clean_command) = &mut self.clean_command {
            *clean_command = clean_command.trim().to_string();
        }
    }
}

/// @brief 依赖项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}

impl Dependency {
    #[allow(dead_code)]
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("name is empty".to_string());
        }
        if self.version.is_empty() {
            return Err("version is empty".to_string());
        }
        return Ok(());
    }

    pub fn trim(&mut self) {
        self.name = self.name.trim().to_string();
        self.version = self.version.trim().to_string();
    }

    pub fn name_version(&self) -> String {
        return format!("{}-{}", self.name, self.version);
    }
}

/// # 任务类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    /// 从源码构建
    BuildFromSource(CodeSource),
    /// 从预编译包安装
    InstallFromPrebuilt(PrebuiltSource),
}

impl TaskType {
    pub fn validate(&mut self) -> Result<(), String> {
        match self {
            TaskType::BuildFromSource(source) => source.validate(),
            TaskType::InstallFromPrebuilt(source) => source.validate(),
        }
    }

    pub fn trim(&mut self) {
        match self {
            TaskType::BuildFromSource(source) => source.trim(),
            TaskType::InstallFromPrebuilt(source) => source.trim(),
        }
    }
}

/// # 代码源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeSource {
    /// 从Git仓库获取
    Git(GitSource),
    /// 从本地目录获取
    Local(LocalSource),
    /// 从在线压缩包获取
    Archive(ArchiveSource),
}

impl CodeSource {
    pub fn validate(&mut self) -> Result<(), String> {
        match self {
            CodeSource::Git(source) => source.validate(),
            CodeSource::Local(source) => source.validate(Some(false)),
            CodeSource::Archive(source) => source.validate(),
        }
    }

    pub fn trim(&mut self) {
        match self {
            CodeSource::Git(source) => source.trim(),
            CodeSource::Local(source) => source.trim(),
            CodeSource::Archive(source) => source.trim(),
        }
    }
}

/// # 预编译包源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrebuiltSource {
    /// 从在线压缩包获取
    Archive(ArchiveSource),
    /// 从本地目录/文件获取
    Local(LocalSource),
}

impl PrebuiltSource {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            PrebuiltSource::Archive(source) => source.validate(),
            PrebuiltSource::Local(source) => source.validate(None),
        }
    }

    pub fn trim(&mut self) {
        match self {
            PrebuiltSource::Archive(source) => source.trim(),
            PrebuiltSource::Local(source) => source.trim(),
        }
    }
}

/// # 任务环境变量
///
/// 任务执行时的环境变量.这个环境变量是在当前任务执行时设置的，不会影响到其他任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEnv {
    pub key: String,
    pub value: String,
}

impl TaskEnv {
    #[allow(dead_code)]
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn trim(&mut self) {
        self.key = self.key.trim().to_string();
        self.value = self.value.trim().to_string();
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.key.is_empty() {
            return Err("Env: key is empty".to_string());
        }
        return Ok(());
    }
}
