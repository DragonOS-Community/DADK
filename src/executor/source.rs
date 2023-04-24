use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use log::info;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::utils::stdio::StdioUtils;

use super::cache::CacheDir;

/// # Git源
///
/// 从Git仓库获取源码
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSource {
    /// Git仓库地址
    url: String,
    /// 分支（可选，如果为空，则拉取master）branch和revision只能同时指定一个
    branch: Option<String>,
    /// 特定的提交的hash值（可选，如果为空，则拉取branch的最新提交）
    revision: Option<String>,
}

impl GitSource {
    pub fn new(url: String, branch: Option<String>, revision: Option<String>) -> Self {
        Self {
            url,
            branch,
            revision,
        }
    }

    /// # 验证参数合法性
    ///
    /// 仅进行形式校验，不会检查Git仓库是否存在，以及分支是否存在、是否有权限访问等
    pub fn validate(&mut self) -> Result<(), String> {
        if self.url.is_empty() {
            return Err("url is empty".to_string());
        }
        // branch和revision不能同时为空
        if self.branch.is_none() && self.revision.is_none() {
            self.branch = Some("master".to_string());
        }
        // branch和revision只能同时指定一个
        if self.branch.is_some() && self.revision.is_some() {
            return Err("branch and revision are both specified".to_string());
        }

        if self.branch.is_some() {
            if self.branch.as_ref().unwrap().is_empty() {
                return Err("branch is empty".to_string());
            }
        }
        if self.revision.is_some() {
            if self.revision.as_ref().unwrap().is_empty() {
                return Err("revision is empty".to_string());
            }
        }
        return Ok(());
    }

    pub fn trim(&mut self) {
        self.url = self.url.trim().to_string();
        if let Some(branch) = &mut self.branch {
            *branch = branch.trim().to_string();
        }

        if let Some(revision) = &mut self.revision {
            *revision = revision.trim().to_string();
        }
    }

    /// # 确保Git仓库已经克隆到指定目录，并且切换到指定分支/Revision
    ///
    /// 如果目录不存在，则会自动创建
    ///
    /// ## 参数
    ///
    /// * `target_dir` - 目标目录
    ///
    /// ## 返回
    ///
    /// * `Ok(())` - 成功
    /// * `Err(String)` - 失败，错误信息
    pub fn prepare(&self, target_dir: &CacheDir) -> Result<(), String> {
        info!(
            "Preparing git repo: {}, branch: {:?}, revision: {:?}",
            self.url, self.branch, self.revision
        );

        target_dir.create().map_err(|e| {
            format!(
                "Failed to create target dir: {}, message: {e:?}",
                target_dir.path.display()
            )
        })?;

        if target_dir.is_empty().map_err(|e| {
            format!(
                "Failed to check if target dir is empty: {}, message: {e:?}",
                target_dir.path.display()
            )
        })? {
            info!("Target dir is empty, cloning repo");
            self.clone_repo(target_dir)?;
        }

        self.checkout(target_dir)?;

        self.pull(target_dir)?;

        return Ok(());
    }

    fn checkout(&self, target_dir: &CacheDir) -> Result<(), String> {
        let do_checkout = || -> Result<(), String> {
            let mut cmd = Command::new("git");
            cmd.current_dir(&target_dir.path);
            cmd.arg("checkout");

            if let Some(branch) = &self.branch {
                cmd.arg(branch);
            }
            if let Some(revision) = &self.revision {
                cmd.arg(revision);
            }

            // 强制切换分支，且安静模式
            cmd.arg("-f").arg("-q");

            // 创建子进程，执行命令
            let proc: std::process::Child = cmd
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| e.to_string())?;
            let output = proc.wait_with_output().map_err(|e| e.to_string())?;

            if !output.status.success() {
                return Err(format!(
                    "Failed to checkout {}, message: {}",
                    target_dir.path.display(),
                    String::from_utf8_lossy(&output.stdout)
                ));
            }
            return Ok(());
        };

        if let Err(_) = do_checkout() {
            // 如果切换分支失败，则尝试重新fetch
            if self.revision.is_some() {
                self.set_fetch_config(target_dir)?;
                self.unshallow(target_dir)?
            };

            self.fetch_all(target_dir).ok();
            do_checkout()?;
        }

        return Ok(());
    }

    pub fn clone_repo(&self, cache_dir: &CacheDir) -> Result<(), String> {
        let path: &PathBuf = &cache_dir.path;
        let mut cmd = Command::new("git");
        cmd.arg("clone").arg(&self.url).arg(".").arg("--recursive");

        if let Some(branch) = &self.branch {
            cmd.arg("--branch").arg(branch).arg("--depth").arg("1");
        }

        // 对于克隆，如果指定了revision，则直接克隆整个仓库，稍后再切换到指定的revision

        // 设置工作目录
        cmd.current_dir(path);

        // 创建子进程，执行命令
        let proc: std::process::Child = cmd
            .stderr(Stdio::piped())
            .stdout(Stdio::inherit())
            .spawn()
            .map_err(|e| e.to_string())?;
        let output = proc.wait_with_output().map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(format!(
                "clone git repo failed, status: {:?},  stderr: {:?}",
                output.status,
                StdioUtils::tail_n_str(StdioUtils::stderr_to_lines(&output.stderr), 5)
            ));
        }
        return Ok(());
    }

    /// 设置fetch所有分支
    fn set_fetch_config(&self, target_dir: &CacheDir) -> Result<(), String> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&target_dir.path);
        cmd.arg("config")
            .arg("remote.origin.fetch")
            .arg("+refs/heads/*:refs/remotes/origin/*");

        // 创建子进程，执行命令
        let proc: std::process::Child = cmd
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        let output = proc.wait_with_output().map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(format!(
                "Failed to set fetch config {}, message: {}",
                target_dir.path.display(),
                StdioUtils::tail_n_str(StdioUtils::stderr_to_lines(&output.stderr), 5)
            ));
        }
        return Ok(());
    }

    /// # 把浅克隆的仓库变成深克隆
    fn unshallow(&self, target_dir: &CacheDir) -> Result<(), String> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&target_dir.path);
        cmd.arg("fetch").arg("--unshallow");

        // 安静模式
        cmd.arg("-f").arg("-q");

        // 创建子进程，执行命令
        let proc: std::process::Child = cmd
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        let output = proc.wait_with_output().map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(format!(
                "Failed to unshallow {}, message: {}",
                target_dir.path.display(),
                StdioUtils::tail_n_str(StdioUtils::stderr_to_lines(&output.stderr), 5)
            ));
        }
        return Ok(());
    }

    fn fetch_all(&self, target_dir: &CacheDir) -> Result<(), String> {
        self.set_fetch_config(target_dir)?;
        let mut cmd = Command::new("git");
        cmd.current_dir(&target_dir.path);
        cmd.arg("fetch").arg("--all");

        // 安静模式
        cmd.arg("-f").arg("-q");

        // 创建子进程，执行命令
        let proc: std::process::Child = cmd
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        let output = proc.wait_with_output().map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(format!(
                "Failed to fetch all {}, message: {}",
                target_dir.path.display(),
                StdioUtils::tail_n_str(StdioUtils::stderr_to_lines(&output.stderr), 5)
            ));
        }

        return Ok(());
    }

    fn pull(&self, target_dir: &CacheDir) -> Result<(), String> {
        // 如果没有指定branch，则不执行pull
        if !self.branch.is_some() {
            return Ok(());
        }
        info!("git pulling: {}", target_dir.path.display());

        let mut cmd = Command::new("git");
        cmd.current_dir(&target_dir.path);
        cmd.arg("pull");

        // 安静模式
        cmd.arg("-f").arg("-q");

        // 创建子进程，执行命令
        let proc: std::process::Child = cmd
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        let output = proc.wait_with_output().map_err(|e| e.to_string())?;

        // 如果pull失败，且指定了branch，则报错
        if !output.status.success() {
            return Err(format!(
                "Failed to pull {}, message: {}",
                target_dir.path.display(),
                StdioUtils::tail_n_str(StdioUtils::stderr_to_lines(&output.stderr), 5)
            ));
        }

        return Ok(());
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

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
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
