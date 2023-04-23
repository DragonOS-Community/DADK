//! # 配置解析器
//!
//! 用于解析配置文件，生成任务列表
//!
//! 您需要指定一个配置文件目录，解析器会自动解析该目录下的所有配置文件。
//! 软件包的配置文件必须以`.dadk`作为后缀名，内容格式为json。
//!
//! ## 简介
//!
//! 在每个配置文件中，您需要指定软件包的名称、版本、描述、任务类型、依赖、构建配置和安装配置。DADK会根据这些信息生成任务列表。
//!
//! ## 配置文件格式
//!
//! ```json
//! {
//!     "name": "软件包名称",
//!     "version": "软件包版本",
//!     "description": "软件包描述",
//!     "task_type": {任务类型（该部分详见`TaskType`的文档）},
//!     "depends": [{依赖项（该部分详见Dependency的文档）}],
//!     "build": {构建配置（该部分详见BuildConfig的文档）},
//!     "install": {安装配置（该部分详见InstallConfig的文档）},
//!     "envs" : [{ "key": "环境变量名", "value": "环境变量值" }]
//! }
use std::{
    fmt::Debug,
    fs::{DirEntry, ReadDir},
    path::PathBuf,
};

use log::{error, info};

use self::task::DADKTask;
pub mod task;

/// # 配置解析器
///
/// 用于解析配置文件，生成任务列表
#[derive(Debug)]
pub struct Parser {
    /// 配置文件目录
    config_dir: PathBuf,
    /// 扫描到的配置文件列表
    config_files: Vec<PathBuf>,
}

pub struct ParserError {
    pub config_file: Option<PathBuf>,
    pub error: InnerParserError,
}
impl Debug for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error {
            InnerParserError::IoError(e) => {
                if let Some(config_file) = &self.config_file {
                    write!(
                        f,
                        "IO Error while parsing config file {}: {}",
                        config_file.display(),
                        e
                    )
                } else {
                    write!(f, "IO Error while parsing config files: {}", e)
                }
            }
            InnerParserError::JsonError(e) => {
                if let Some(config_file) = &self.config_file {
                    write!(
                        f,
                        "Json Error while parsing config file {}: {}",
                        config_file.display(),
                        e
                    )
                } else {
                    write!(f, "Json Error while parsing config file: {}", e)
                }
            }
            InnerParserError::TaskError(e) => {
                if let Some(config_file) = &self.config_file {
                    write!(
                        f,
                        "Error while parsing config file {}: {}",
                        config_file.display(),
                        e
                    )
                } else {
                    write!(f, "Error while parsing config file: {}", e)
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum InnerParserError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    TaskError(String),
}

impl Parser {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            config_files: Vec::new(),
        }
    }

    /// # 解析所有配置文件，生成任务列表
    ///
    /// ## 参数
    ///
    /// * `config_dir` - 配置文件所在目录
    ///
    /// ## 返回值
    ///
    /// * `Ok(Vec<(PathBuf, DADKTask)>)` - 任务列表(配置文件路径, 任务)
    /// * `Err(ParserError)` - 解析错误
    pub fn parse(&mut self) -> Result<Vec<(PathBuf, DADKTask)>, ParserError> {
        self.scan_config_files()?;
        info!("Found {} config files", self.config_files.len());
        let r: Result<Vec<(PathBuf, DADKTask)>, ParserError> = self.gen_tasks();
        if r.is_err() {
            error!("Error while parsing config files: {:?}", r);
        }
        return r;
    }

    /// # 扫描配置文件目录，找到所有配置文件
    fn scan_config_files(&mut self) -> Result<(), ParserError> {
        info!("Scanning config files in {}", self.config_dir.display());

        let mut dir_queue: Vec<PathBuf> = Vec::new();
        // 将config目录加入队列
        dir_queue.push(self.config_dir.clone());

        while !dir_queue.is_empty() {
            // 扫描目录，找到所有*.dadk文件
            let dir = dir_queue.pop().unwrap();
            let entries: ReadDir = std::fs::read_dir(&dir).map_err(|e| ParserError {
                config_file: None,
                error: InnerParserError::IoError(e),
            })?;

            for entry in entries {
                let entry: DirEntry = entry.map_err(|e| ParserError {
                    config_file: None,
                    error: InnerParserError::IoError(e),
                })?;

                let path: PathBuf = entry.path();
                if path.is_dir() {
                    dir_queue.push(path);
                } else if path.is_file() {
                    let extension: Option<&std::ffi::OsStr> = path.extension();
                    if extension.is_none() {
                        continue;
                    }
                    let extension: &std::ffi::OsStr = extension.unwrap();
                    if extension.to_ascii_lowercase() != "dadk" {
                        continue;
                    }
                    // 找到一个配置文件, 加入列表
                    self.config_files.push(path);
                }
            }
        }

        return Ok(());
    }

    /// # 解析所有配置文件，生成任务列表
    ///
    /// 一旦发生错误，立即返回
    ///
    /// ## 返回值
    ///
    /// * `Ok(Vec<DADKTask>)` - 任务列表
    /// * `Err(ParserError)` - 解析错误
    fn gen_tasks(&self) -> Result<Vec<(PathBuf, DADKTask)>, ParserError> {
        let mut result_vec = Vec::new();
        for config_file in &self.config_files {
            let task: DADKTask = self.parse_config_file(config_file)?;
            result_vec.push((config_file.clone(), task));
        }

        return Ok(result_vec);
    }

    /// # 解析单个配置文件，生成任务
    ///
    /// ## 参数
    ///
    /// * `config_file` - 配置文件路径
    ///
    /// ## 返回值
    ///
    /// * `Ok(DADKTask)` - 生成好的任务
    /// * `Err(ParserError)` - 解析错误
    fn parse_config_file(&self, config_file: &PathBuf) -> Result<DADKTask, ParserError> {
        let content = std::fs::read_to_string(config_file).map_err(|e| ParserError {
            config_file: Some(config_file.clone()),
            error: InnerParserError::IoError(e),
        })?;

        // 从json字符串中解析出DADKTask
        let mut task: DADKTask = serde_json::from_str(&content).map_err(|e| ParserError {
            config_file: Some(config_file.clone()),
            error: InnerParserError::JsonError(e),
        })?;

        // 去除字符串中的空白字符
        task.trim();

        // 校验DADKTask的参数是否合法
        task.validate().map_err(|e| ParserError {
            config_file: Some(config_file.clone()),
            error: InnerParserError::TaskError(e),
        })?;

        return Ok(task);
    }
}
