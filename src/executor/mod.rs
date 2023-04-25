use std::{
    collections::BTreeMap,
    env::Vars,
    path::PathBuf,
    process::{Command, Stdio},
    rc::Rc,
    sync::RwLock,
};

use log::{error, info, warn};

use crate::{
    console::{clean::CleanLevel, Action},
    executor::cache::CacheDir,
    parser::task::{CodeSource, PrebuiltSource, TaskEnv, TaskType},
    scheduler::{SchedEntities, SchedEntity},
    utils::stdio::StdioUtils,
};

use self::cache::CacheDirType;

pub mod cache;
pub mod source;

lazy_static! {
    // 全局环境变量的列表
    pub static ref ENV_LIST: RwLock<EnvMap> = RwLock::new(EnvMap::new());
}

#[derive(Debug, Clone)]
pub struct Executor {
    entity: Rc<SchedEntity>,
    action: Action,
    local_envs: EnvMap,
    /// 任务构建结果输出到的目录
    build_dir: CacheDir,
    /// 如果任务需要源文件缓存，则此字段为 Some(CacheDir)，否则为 None（使用本地源文件路径）
    source_dir: Option<CacheDir>,
    /// DragonOS sysroot的路径
    dragonos_sysroot: PathBuf,
}

impl Executor {
    /// # 创建执行器
    ///
    /// 用于执行一个任务
    ///
    /// ## 参数
    ///
    /// * `entity` - 任务调度实体
    ///
    /// ## 返回值
    ///
    /// * `Ok(Executor)` - 创建成功
    /// * `Err(ExecutorError)` - 创建失败
    pub fn new(
        entity: Rc<SchedEntity>,
        action: Action,
        dragonos_sysroot: PathBuf,
    ) -> Result<Self, ExecutorError> {
        let local_envs = EnvMap::new();
        let build_dir = CacheDir::new(entity.clone(), CacheDirType::Build)?;

        let source_dir = if CacheDir::need_source_cache(&entity) {
            Some(CacheDir::new(entity.clone(), CacheDirType::Source)?)
        } else {
            None
        };

        let result: Executor = Self {
            action,
            entity,
            local_envs,
            build_dir,
            source_dir,
            dragonos_sysroot,
        };

        return Ok(result);
    }

    /// # 执行任务
    ///
    /// 创建执行器后，调用此方法执行任务。
    /// 该方法会执行以下步骤：
    ///
    /// 1. 创建工作线程
    /// 2. 准备环境变量
    /// 3. 拉取数据（可选）
    /// 4. 执行构建
    pub fn execute(&mut self) -> Result<(), ExecutorError> {
        info!("Execute task: {}", self.entity.task().name_version());

        // 准备本地环境变量
        self.prepare_local_env()?;

        match self.action {
            Action::Build => {
                // 构建任务
                self.build()?;
            }
            Action::Install => {
                // 把构建结果安装到DragonOS
                self.install()?;
            }
            Action::Clean(_) => {
                // 清理构建结果
                let r = self.clean();
                if let Err(e) = r {
                    error!("Failed to clean task {}: {:?}", self.entity.task().name_version(), e);
                }
            }
            _ => {
                error!("Unsupported action: {:?}", self.action);
            }
        }
        info!("Task {} finished", self.entity.task().name_version());
        return Ok(());
    }

    /// # 执行build操作
    fn build(&mut self) -> Result<(), ExecutorError> {
        // 确认源文件就绪
        self.prepare_input()?;

        let command: Option<Command> = self.create_command()?;
        if let Some(cmd) = command {
            self.run_command(cmd)?;
        }

        // 检查构建结果，如果为空，则抛出警告
        if self.build_dir.is_empty()? {
            warn!(
                "Task {}: build result is empty, do you forget to copy the result to [${}]?",
                self.entity.task().name_version(),
                CacheDir::build_dir_env_key(&self.entity)?
            );
        }
        return Ok(());
    }

    /// # 执行安装操作，把构建结果安装到DragonOS
    fn install(&self) -> Result<(), ExecutorError> {
        let in_dragonos_path = self.entity.task().install.in_dragonos_path.as_ref();
        // 如果没有指定安装路径，则不执行安装
        if in_dragonos_path.is_none() {
            return Ok(());
        }
        info!("Installing task: {}", self.entity.task().name_version());
        let mut in_dragonos_path = in_dragonos_path.unwrap().to_string_lossy().to_string();

        // 去除开头的斜杠
        {
            let count_leading_slashes = in_dragonos_path.chars().take_while(|c| *c == '/').count();
            in_dragonos_path = in_dragonos_path[count_leading_slashes..].to_string();
        }
        // 拼接最终的安装路径
        let install_path = self.dragonos_sysroot.join(in_dragonos_path);
        // debug!("install_path: {:?}", install_path);
        // 创建安装路径
        std::fs::create_dir_all(&install_path).map_err(|e| {
            ExecutorError::InstallError(format!("Failed to create install path: {}", e.to_string()))
        })?;

        // 拷贝构建结果到安装路径
        let build_dir: PathBuf = self.build_dir.path.clone();

        let cmd = Command::new("cp")
            .arg("-r")
            .arg(build_dir.to_string_lossy().to_string() + "/.")
            .arg(install_path)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                ExecutorError::InstallError(format!(
                    "Failed to install, error message: {}",
                    e.to_string()
                ))
            })?;

        let output = cmd.wait_with_output().map_err(|e| {
            ExecutorError::InstallError(format!(
                "Failed to install, error message: {}",
                e.to_string()
            ))
        })?;

        if !output.status.success() {
            let err_msg = StdioUtils::tail_n_str(StdioUtils::stderr_to_lines(&output.stderr), 10);
            return Err(ExecutorError::InstallError(format!(
                "Failed to install, error message: {}",
                err_msg
            )));
        }

        info!("Task {} installed.", self.entity.task().name_version());

        return Ok(());
    }

    fn clean(&self) -> Result<(), ExecutorError> {
        let level = if let Action::Clean(l) = self.action {
            l.level
        } else {
            panic!(
                "BUG: clean() called with non-clean action. executor details: {:?}",
                self
            );
        };
        info!(
            "Cleaning task: {}, level={level}",
            self.entity.task().name_version()
        );

        let r: Result<(), ExecutorError> = match level {
            CleanLevel::All => self.clean_all(),
            CleanLevel::Src => self.clean_src(),
            CleanLevel::Target => self.clean_target(),
            CleanLevel::Cache => self.clean_cache(),
        };

        if let Err(e) = r {
            error!(
                "Failed to clean task: {}, error message: {:?}",
                self.entity.task().name_version(),
                e
            );
            return Err(e);
        }

        return Ok(());
    }

    fn clean_all(&self) -> Result<(), ExecutorError> {
        // 在源文件目录执行清理
        self.clean_src()?;
        // 清理构建结果
        self.clean_target()?;
        // 清理缓存
        self.clean_cache()?;
        return Ok(());
    }

    /// 在源文件目录执行清理
    fn clean_src(&self) -> Result<(), ExecutorError> {
        let cmd: Option<Command> = self.create_command()?;
        if cmd.is_none() {
            // 如果这里没有命令，则认为用户不需要在源文件目录执行清理
            return Ok(());
        }
        info!(
            "{}: Cleaning in source directory: {:?}",
            self.entity.task().name_version(),
            self.src_work_dir()
        );

        let cmd = cmd.unwrap();
        self.run_command(cmd)?;
        return Ok(());
    }

    /// 清理构建输出目录
    fn clean_target(&self) -> Result<(), ExecutorError> {
        info!(
            "{}: Cleaning build target directory: {:?}",
            self.entity.task().name_version(),
            self.build_dir.path
        );
        return self.build_dir.remove_self_recursive();
    }

    /// 清理下载缓存
    fn clean_cache(&self) -> Result<(), ExecutorError> {
        let cache_dir = self.source_dir.as_ref();
        if cache_dir.is_none() {
            // 如果没有缓存目录，则认为用户不需要清理缓存
            return Ok(());
        }
        info!(
            "{}: Cleaning cache directory: {}",
            self.entity.task().name_version(),
            self.src_work_dir().display()
        );

        return cache_dir.unwrap().remove_self_recursive();
    }

    /// 获取源文件的工作目录
    fn src_work_dir(&self) -> PathBuf {
        if let Some(local_path) = self.entity.task().source_path() {
            return local_path;
        }

        return self.source_dir.as_ref().unwrap().path.clone();
    }

    /// 为任务创建命令
    fn create_command(&self) -> Result<Option<Command>, ExecutorError> {
        // 获取命令
        let raw_cmd = match self.entity.task().task_type {
            TaskType::BuildFromSource(_) => match self.action {
                Action::Build => self.entity.task().build.build_command.clone(),
                Action::Clean(_) => self.entity.task().clean.clean_command.clone(),
                _ => unimplemented!(
                    "create_command: Action {:?} not supported yet.",
                    self.action
                ),
            },
            _ => None,
        };

        if raw_cmd.is_none() {
            return Ok(None);
        }

        let raw_cmd = raw_cmd.unwrap();

        let mut command = Command::new("bash");
        command.current_dir(self.src_work_dir());

        // 设置参数
        command.arg("-c");
        command.arg(raw_cmd);

        // 设置环境变量
        let env_list = ENV_LIST.read().unwrap();
        for (key, value) in env_list.envs.iter() {
            // if key.starts_with("DADK") {
            //     debug!("DADK env found: {}={}", key, value.value);
            // }
            command.env(key, value.value.clone());
        }
        drop(env_list);
        for (key, value) in self.local_envs.envs.iter() {
            command.env(key, value.value.clone());
        }

        return Ok(Some(command));
    }

    /// # 准备工作线程本地环境变量
    fn prepare_local_env(&mut self) -> Result<(), ExecutorError> {
        // 设置本地环境变量
        let task_envs: Option<&Vec<TaskEnv>> = self.entity.task().envs.as_ref();
        if task_envs.is_none() {
            return Ok(());
        }

        let task_envs = task_envs.unwrap();
        for tv in task_envs.iter() {
            self.local_envs
                .add(EnvVar::new(tv.key().to_string(), tv.value().to_string()));
        }

        return Ok(());
    }

    fn prepare_input(&self) -> Result<(), ExecutorError> {
        // 拉取源文件
        if self.source_dir.is_none() {
            return Ok(());
        }
        let task = self.entity.task();
        let source_dir = self.source_dir.as_ref().unwrap();

        match &task.task_type {
            TaskType::BuildFromSource(cs) => {
                match cs {
                    CodeSource::Git(git) => {
                        git.prepare(source_dir)
                            .map_err(|e| ExecutorError::PrepareEnvError(e))?;
                    }
                    // 本地源文件，不需要拉取
                    CodeSource::Local(_) => return Ok(()),
                    // 在线压缩包，需要下载
                    CodeSource::Archive(_) => todo!(),
                }
            }
            TaskType::InstallFromPrebuilt(pb) => {
                match pb {
                    // 本地源文件，不需要拉取
                    PrebuiltSource::Local(_) => return Ok(()),
                    // 在线压缩包，需要下载
                    PrebuiltSource::Archive(_) => todo!(),
                }
            }
        }

        return Ok(());
    }

    fn run_command(&self, mut command: Command) -> Result<(), ExecutorError> {
        let mut child = command
            .stdin(Stdio::inherit())
            .spawn()
            .map_err(|e| ExecutorError::IoError(e))?;

        // 等待子进程结束
        let r = child.wait().map_err(|e| ExecutorError::IoError(e));
        if r.is_ok() {
            let r = r.unwrap();
            if r.success() {
                return Ok(());
            } else {
                // 执行失败，获取最后100行stderr输出
                let errmsg = format!(
                    "Task {} failed, exit code = {}",
                    self.entity.task().name_version(),
                    r.code().unwrap()
                );
                error!("{errmsg}");
                let command_opt = command.output();
                if command_opt.is_err() {
                    return Err(ExecutorError::TaskFailed(
                        "Failed to get command output".to_string(),
                    ));
                }
                let command_opt = command_opt.unwrap();
                let command_output = String::from_utf8_lossy(&command_opt.stderr);
                let mut last_100_outputs = command_output
                    .lines()
                    .rev()
                    .take(100)
                    .collect::<Vec<&str>>();
                last_100_outputs.reverse();
                error!("Last 100 lines msg of stderr:");
                for line in last_100_outputs {
                    error!("{}", line);
                }
                return Err(ExecutorError::TaskFailed(errmsg));
            }
        } else {
            let errmsg = format!(
                "Task {} failed, msg = {:?}",
                self.entity.task().name_version(),
                r.err().unwrap()
            );
            error!("{errmsg}");
            return Err(ExecutorError::TaskFailed(errmsg));
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnvMap {
    pub envs: BTreeMap<String, EnvVar>,
}

impl EnvMap {
    pub fn new() -> Self {
        Self {
            envs: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, env: EnvVar) {
        self.envs.insert(env.key.clone(), env);
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&EnvVar> {
        self.envs.get(key)
    }

    pub fn add_vars(&mut self, vars: Vars) {
        for (key, value) in vars {
            self.add(EnvVar::new(key, value));
        }
    }
}

/// # 环境变量
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

impl EnvVar {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

/// # 任务执行器错误枚举
#[allow(dead_code)]
#[derive(Debug)]
pub enum ExecutorError {
    /// 准备执行环境错误
    PrepareEnvError(String),
    IoError(std::io::Error),
    /// 构建执行错误
    TaskFailed(String),
    /// 安装错误
    InstallError(String),
    /// 清理错误
    CleanError(String),
}

/// # 准备全局环境变量
pub fn prepare_env(sched_entities: &SchedEntities) -> Result<(), ExecutorError> {
    info!("Preparing environment variables...");
    // 获取当前全局环境变量列表
    let mut env_list = ENV_LIST.write().unwrap();
    let envs: Vars = std::env::vars();
    env_list.add_vars(envs);

    // 为每个任务创建特定的环境变量
    for entity in sched_entities.iter() {
        // 导出任务的构建目录环境变量
        let build_dir = CacheDir::build_dir(entity.clone())?;

        let build_dir_key = CacheDir::build_dir_env_key(&entity)?;
        env_list.add(EnvVar::new(
            build_dir_key,
            build_dir.to_str().unwrap().to_string(),
        ));

        // 如果需要源码缓存目录，则导出
        if CacheDir::need_source_cache(entity) {
            let source_dir = CacheDir::source_dir(entity.clone())?;
            let source_dir_key = CacheDir::source_dir_env_key(&entity)?;
            env_list.add(EnvVar::new(
                source_dir_key,
                source_dir.to_str().unwrap().to_string(),
            ));
        }
    }

    // 查看环境变量列表
    // debug!("Environment variables:");

    // for (key, value) in env_list.envs.iter() {
    //     debug!("{}: {}", key, value.value);
    // }

    return Ok(());
}
