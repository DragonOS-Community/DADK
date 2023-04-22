use std::path::PathBuf;

use reqwest::Url;
use serde::{Deserialize, Serialize};

// 对于生成的包名和版本号，需要进行替换的字符。
pub static NAME_VERSION_REPLACE_TABLE: [(&str, &str); 3] = [(" ", "_"), ("\t", "_"), ("-", "_")];

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
            envs,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("name is empty".to_string());
        }
        if self.version.is_empty() {
            return Err("version is empty".to_string());
        }
        self.task_type.validate()?;
        self.build.validate()?;
        self.install.validate()?;
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

    pub fn name_version(&self) -> String {
        return format!("{}-{}", self.name, self.version);
    }

    pub fn name_version_env(&self) -> String {
        let mut name_version = self.name_version();
        for (src, dst) in &NAME_VERSION_REPLACE_TABLE {
            name_version = name_version.replace(src, dst);
        }
        return name_version;
    }

}

/// @brief 构建配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// 构建命令
    pub build_command: String,
}

impl BuildConfig {
    #[allow(dead_code)]
    pub fn new(build_command: String) -> Self {
        Self { build_command }
    }

    pub fn validate(&self) -> Result<(), String> {
        return Ok(());
    }

    pub fn trim(&mut self) {
        self.build_command = self.build_command.trim().to_string();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// 安装到DragonOS内的目录
    pub in_dragonos_path: PathBuf,
    /// 安装命令
    pub install_command: String,
}

impl InstallConfig {
    #[allow(dead_code)]
    pub fn new(in_dragonos_path: PathBuf, install_command: String) -> Self {
        Self {
            in_dragonos_path,
            install_command,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        return Ok(());
    }

    pub fn trim(&mut self) {
        self.install_command = self.install_command.trim().to_string();
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
    pub fn validate(&self) -> Result<(), String> {
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
    pub fn validate(&self) -> Result<(), String> {
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

/// # Git源
///
/// 从Git仓库获取源码
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSource {
    /// Git仓库地址
    url: String,
    /// 分支
    branch: String,
    /// 特定的提交的hash值（可选，如果为空，则拉取最新提交）
    revision: Option<String>,
}

impl GitSource {
    pub fn new(url: String, branch: String, revision: Option<String>) -> Self {
        Self {
            url,
            branch,
            revision,
        }
    }

    /// # 验证参数合法性
    ///
    /// 仅进行形式校验，不会检查Git仓库是否存在，以及分支是否存在、是否有权限访问等
    pub fn validate(&self) -> Result<(), String> {
        if self.url.is_empty() {
            return Err("url is empty".to_string());
        }
        if self.branch.is_empty() {
            return Err("branch is empty".to_string());
        }
        return Ok(());
    }

    pub fn trim(&mut self) {
        self.url = self.url.trim().to_string();
        self.branch = self.branch.trim().to_string();
        if let Some(revision) = &mut self.revision {
            *revision = revision.trim().to_string();
        }
    }
}

/// # 本地源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSource {
    /// 本地目录/文件的路径
    path: PathBuf,
}

impl LocalSource {
    #[allow(dead_code)]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn validate(&self, expect_file: Option<bool>) -> Result<(), String> {
        if !self.path.exists() {
            return Err(format!("path {:?} not exists", self.path));
        }

        if let Some(expect_file) = expect_file {
            if expect_file && !self.path.is_file() {
                return Err(format!("path {:?} is not a file", self.path));
            }

            if !expect_file && !self.path.is_dir() {
                return Err(format!("path {:?} is not a directory", self.path));
            }
        }

        return Ok(());
    }

    pub fn trim(&mut self) {}
}

/// # 在线压缩包源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveSource {
    /// 压缩包的URL
    url: String,
}

impl ArchiveSource {
    #[allow(dead_code)]
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.url.is_empty() {
            return Err("url is empty".to_string());
        }

        // 判断是一个网址
        if let Ok(url) = Url::parse(&self.url) {
            if url.scheme() != "http" && url.scheme() != "https" {
                return Err(format!("url {:?} is not a http/https url", self.url));
            }
        } else {
            return Err(format!("url {:?} is not a valid url", self.url));
        }
        return Ok(());
    }

    pub fn trim(&mut self) {
        self.url = self.url.trim().to_string();
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
