use std::{cell::RefCell, fmt::Debug, path::PathBuf, rc::Rc};

use log::{debug, error, info};

use crate::{
    console::elements::{OptionalChoice, VecInput},
    executor::{
        cache::CacheDir,
        source::{ArchiveSource, GitSource, LocalSource},
    },
    parser::task::{
        BuildConfig, CleanConfig, CodeSource, DADKTask, Dependency, InstallConfig, PrebuiltSource,
        TaskEnv, TaskType,
    },
};

use super::{
    elements::Input,
    interactive::{InputFunc, InteractiveCommand},
    ConsoleError,
};

#[derive(Debug)]
pub struct NewConfigCommand {
    /// DADK任务配置文件所在目录
    config_dir: Option<PathBuf>,
}

impl InteractiveCommand for NewConfigCommand {
    fn run(&mut self) -> Result<(), ConsoleError> {
        // 如果没有指定配置文件输出的目录，则使用当前目录
        if self.config_dir.is_none() {
            self.config_dir = Some(PathBuf::from("./"));
        }

        println!("To create a new DADK task config, please follow the guidance below... \n");

        let mut dadk_task = self.build_dadk_task()?;
        debug!("dadk_task: {:?}", dadk_task);

        // 校验
        let check: Result<(), ConsoleError> = dadk_task.validate().map_err(|e| {
            let msg = format!("Failed to validate DADKTask: {:?}", e);
            ConsoleError::InvalidInput(msg)
        });

        if check.is_err() {
            error!("{:?}", check.unwrap_err());
        }
        // 不管校验是否通过，都写入文件
        let config_file_path = self.write_dadk_config_file(&dadk_task)?;

        info!(
            "DADK task config file created successfully! File:{}",
            config_file_path.display()
        );
        return Ok(());
    }
}

impl NewConfigCommand {
    pub fn new(config_dir: Option<PathBuf>) -> Self {
        Self { config_dir }
    }

    fn write_dadk_config_file(&self, dadk_task: &DADKTask) -> Result<PathBuf, ConsoleError> {
        let json = serde_json::to_string_pretty(&dadk_task).map_err(|e| {
            let msg = format!("Failed to serialize DADKTask to json: {:?}", e);
            error!("{}", msg);
            ConsoleError::InvalidInput(msg)
        })?;
        info!("Complete DADK task config file:\n {}", json);

        // 创建路径
        let config_dir = self.config_dir.as_ref().unwrap();
        let filename = format!("{}.dadk", dadk_task.name_version());
        let config_path = config_dir.join(filename);

        // 写入文件
        std::fs::write(&config_path, json).map_err(|e| {
            let msg = format!(
                "Failed to write config file to {}, error: {:?}",
                config_path.display(),
                e
            );
            error!("{}", msg);
            ConsoleError::InvalidInput(msg)
        })?;

        return Ok(config_path);
    }

    fn build_dadk_task(&self) -> Result<DADKTask, ConsoleError> {
        let name = self.input_name()?;
        let version = self.input_version()?;
        let description = self.input_description()?;
        debug!(
            "name: {}, version: {}, description: {}",
            name, version, description
        );

        let task_type: TaskType = TaskTypeInput::new().input()?;
        debug!("task_type: {:?}", task_type);

        let dep: Vec<Dependency> = DependencyInput::new().input()?;
        debug!("dep: {:?}", dep);
        let build_config: BuildConfig = BuildConfigInput::new().input()?;
        debug!("build_config: {:?}", build_config);
        let install_config: InstallConfig = InstallConfigInput::new().input()?;
        debug!("install_config: {:?}", install_config);
        let clean_config: CleanConfig = CleanConfigInput::new().input()?;
        debug!("clean_config: {:?}", clean_config);

        let task_env: Option<Vec<TaskEnv>> = TaskEnvInput::new().input()?;
        debug!("task_env: {:?}", task_env);

        let mut dadk: DADKTask = DADKTask::new(
            name,
            version,
            description,
            task_type,
            dep,
            build_config,
            install_config,
            clean_config,
            task_env,
        );

        dadk.trim();

        return Ok(dadk);
    }

    // 输入任务名称
    fn input_name(&self) -> Result<String, ConsoleError> {
        let name = Input::new(
            Some("Please input the [name] of the task:".to_string()),
            None,
        )
        .input()?;
        Ok(name)
    }

    // 输入任务版本
    fn input_version(&self) -> Result<String, ConsoleError> {
        let version = Input::new(
            Some("Please input the [version] of the task:".to_string()),
            None,
        )
        .input()?;

        return Ok(version);
    }

    // 输入任务描述
    fn input_description(&self) -> Result<String, ConsoleError> {
        let description = Input::new(
            Some("Please input the [description] of the task:".to_string()),
            None,
        )
        .input()?;

        return Ok(description);
    }
}

#[derive(Debug)]
struct TaskTypeInput;

impl TaskTypeInput {
    pub fn new() -> Self {
        Self {}
    }
}

impl InputFunc<TaskType> for TaskTypeInput {
    /// # 输入任务类型
    fn input(&mut self) -> Result<TaskType, ConsoleError> {
        const TASK_TYPE_BUILD_FROM_SOURCE: &str = "src";
        const TASK_TYPE_INSTALL_FROM_PREBUILT: &str = "prebuilt";

        let mut task_type_choose =
            OptionalChoice::new(Some("Please choose the [type] of the task:".to_string()));
        task_type_choose.add_choice(
            TASK_TYPE_BUILD_FROM_SOURCE.to_string(),
            "Build from source".to_string(),
        );
        task_type_choose.add_choice(
            TASK_TYPE_INSTALL_FROM_PREBUILT.to_string(),
            "Install from prebuilt".to_string(),
        );

        // 读取用户输入
        let task_type = task_type_choose.choose_until_valid()?;

        // debug!("task type: {}", task_type);

        let mut task_type = match task_type.as_str() {
            TASK_TYPE_BUILD_FROM_SOURCE => {
                TaskType::BuildFromSource(CodeSourceInput::new().input()?)
            }
            TASK_TYPE_INSTALL_FROM_PREBUILT => {
                TaskType::InstallFromPrebuilt(PrebuiltSourceInput::new().input()?)
            }
            _ => {
                let msg = format!("Invalid task type: {}", task_type);
                return Err(ConsoleError::InvalidInput(msg));
            }
        };

        // 验证输入
        task_type.validate().map_err(|e| {
            ConsoleError::InvalidInput(format!("Invalid task type: {}", e.to_string()))
        })?;

        return Ok(task_type);
    }
}
/// # 代码源输入
#[derive(Debug)]
struct CodeSourceInput;

impl CodeSourceInput {
    pub fn new() -> Self {
        Self {}
    }

    pub fn input(&self) -> Result<CodeSource, ConsoleError> {
        const CODE_SOURCE_GIT: &str = "git";
        const CODE_SOURCE_LOCAL: &str = "local";
        const CODE_SOURCE_ARCHIVE: &str = "archive";

        let mut code_source_choose = OptionalChoice::new(Some(
            "Please choose the [code source] of the task:".to_string(),
        ));
        code_source_choose.add_choice(
            CODE_SOURCE_GIT.to_string(),
            "Build from git repository".to_string(),
        );
        code_source_choose.add_choice(
            CODE_SOURCE_LOCAL.to_string(),
            "Build from local directory".to_string(),
        );
        code_source_choose.add_choice(
            CODE_SOURCE_ARCHIVE.to_string(),
            "Build from archive file".to_string(),
        );

        // 读取用户输入
        let code_source: String = code_source_choose.choose_until_valid()?;
        // debug!("code source: {}", code_source);

        let mut code_source: CodeSource = match code_source.as_str() {
            CODE_SOURCE_GIT => CodeSource::Git(GitSourceInput::new().input_until_valid()?),
            CODE_SOURCE_LOCAL => CodeSource::Local(LocalSourceInput::new().input_until_valid()?),
            CODE_SOURCE_ARCHIVE => {
                CodeSource::Archive(ArchiveSourceInput::new().input_until_valid()?)
            }
            _ => {
                let msg = format!("Invalid code source: {}", code_source);
                return Err(ConsoleError::InvalidInput(msg));
            }
        };
        code_source.trim();
        code_source.validate().map_err(|e| {
            ConsoleError::InvalidInput(format!("Invalid code source: {}", e.to_string()))
        })?;

        return Ok(code_source);
    }
}

#[derive(Debug)]
struct PrebuiltSourceInput;

impl PrebuiltSourceInput {
    pub fn new() -> Self {
        Self {}
    }
}

impl InputFunc<PrebuiltSource> for PrebuiltSourceInput {
    fn input(&mut self) -> Result<PrebuiltSource, ConsoleError> {
        const PREBUILT_SOURCE_LOCAL: &str = "local";
        const PREBUILT_SOURCE_ARCHIVE: &str = "archive";

        let mut prebuilt_source_choose = OptionalChoice::new(Some(
            "Please choose the [prebuilt source] of the task:".to_string(),
        ));

        prebuilt_source_choose.add_choice(
            PREBUILT_SOURCE_LOCAL.to_string(),
            "Install from local directory".to_string(),
        );
        prebuilt_source_choose.add_choice(
            PREBUILT_SOURCE_ARCHIVE.to_string(),
            "Install from archive file".to_string(),
        );

        // 读取用户输入
        let prebuilt_source: String = prebuilt_source_choose.choose_until_valid()?;
        // debug!("prebuilt source: {}", prebuilt_source);

        let mut prebuilt_source: PrebuiltSource = match prebuilt_source.as_str() {
            PREBUILT_SOURCE_LOCAL => {
                PrebuiltSource::Local(LocalSourceInput::new().input_until_valid()?)
            }
            PREBUILT_SOURCE_ARCHIVE => {
                PrebuiltSource::Archive(ArchiveSourceInput::new().input_until_valid()?)
            }
            _ => {
                let msg = format!("Invalid prebuilt source: {}", prebuilt_source);
                return Err(ConsoleError::InvalidInput(msg));
            }
        };
        prebuilt_source.trim();
        prebuilt_source.validate().map_err(|e| {
            ConsoleError::InvalidInput(format!("Invalid prebuilt source: {}", e.to_string()))
        })?;

        return Ok(prebuilt_source);
    }
}

#[derive(Debug)]
struct GitSourceInput;

impl InputFunc<GitSource> for GitSourceInput {
    fn input(&mut self) -> Result<GitSource, ConsoleError> {
        let url = self.input_url()?;

        // 选择分支还是指定的commit
        const GIT_SOURCE_BRANCH: &str = "branch";
        const GIT_SOURCE_REVISION: &str = "revision";

        let mut git_source_choose = OptionalChoice::new(Some(
            "Please choose the [git source] of the task:".to_string(),
        ));
        git_source_choose.add_choice(GIT_SOURCE_BRANCH.to_string(), "branch name".to_string());
        git_source_choose.add_choice(GIT_SOURCE_REVISION.to_string(), "revision hash".to_string());

        // 读取用户输入
        let git_source = git_source_choose.choose_until_valid()?;
        // debug!("git source: {}", git_source);

        let mut git_source: GitSource = match git_source.as_str() {
            GIT_SOURCE_BRANCH => {
                let branch = self.input_branch()?;
                GitSource::new(url, Some(branch), None)
            }
            GIT_SOURCE_REVISION => {
                let revision = self.input_revision()?;
                GitSource::new(url, None, Some(revision))
            }
            _ => {
                let msg = format!("Invalid git source: {}", git_source);
                return Err(ConsoleError::InvalidInput(msg));
            }
        };
        git_source.trim();
        // 验证输入
        git_source.validate().map_err(|e| {
            ConsoleError::InvalidInput(format!("Invalid git source: {}", e.to_string()))
        })?;

        return Ok(git_source);
    }
}

impl GitSourceInput {
    pub fn new() -> Self {
        Self {}
    }

    fn input_url(&self) -> Result<String, ConsoleError> {
        let url = Input::new(
            Some("Please input the [url] of the git repository:".to_string()),
            None,
        )
        .input()?;
        return Ok(url);
    }

    fn input_branch(&self) -> Result<String, ConsoleError> {
        let branch = Input::new(
            Some("Please input the [branch name] of the git repository:".to_string()),
            None,
        )
        .input()?;
        return Ok(branch);
    }

    fn input_revision(&self) -> Result<String, ConsoleError> {
        let revision = Input::new(
            Some("Please input the [revision hash] of the git repository:".to_string()),
            None,
        )
        .input()?;
        return Ok(revision);
    }
}

#[derive(Debug)]
struct LocalSourceInput;

impl LocalSourceInput {
    pub fn new() -> Self {
        Self {}
    }

    fn input_path(&self) -> Result<String, ConsoleError> {
        let path = Input::new(
            Some("Please input the [path] of the local directory:".to_string()),
            None,
        )
        .input()?;
        return Ok(path);
    }
}
impl InputFunc<LocalSource> for LocalSourceInput {
    fn input(&mut self) -> Result<LocalSource, ConsoleError> {
        let path = self.input_path()?;
        let path = PathBuf::from(path);
        let mut local_source = LocalSource::new(path);

        local_source.trim();
        // 验证输入
        local_source.validate(None).map_err(|e| {
            ConsoleError::InvalidInput(format!("Invalid local source: {}", e.to_string()))
        })?;

        return Ok(local_source);
    }
}

#[derive(Debug)]
struct ArchiveSourceInput;

impl ArchiveSourceInput {
    pub fn new() -> Self {
        Self {}
    }

    fn input_url(&self) -> Result<String, ConsoleError> {
        let url = Input::new(
            Some("Please input the [url] of the archive file:".to_string()),
            None,
        )
        .input()?;
        return Ok(url);
    }
}

impl InputFunc<ArchiveSource> for ArchiveSourceInput {
    fn input(&mut self) -> Result<ArchiveSource, ConsoleError> {
        let url = self.input_url()?;
        let mut archive_source = ArchiveSource::new(url);

        archive_source.trim();
        // 验证输入
        archive_source.validate().map_err(|e| {
            ConsoleError::InvalidInput(format!("Invalid archive source: {}", e.to_string()))
        })?;

        return Ok(archive_source);
    }
}

#[derive(Debug)]
struct DependencyInput;

impl DependencyInput {
    pub fn new() -> Self {
        Self {}
    }
}

impl InputFunc<Vec<Dependency>> for DependencyInput {
    fn input(&mut self) -> Result<Vec<Dependency>, ConsoleError> {
        const TIPS: &str = "Please input the [dependencies] of the task:";
        println!();
        println!("Please input the [dependencies] of the task:");
        let dependency_reader: Rc<RefCell<DependencyInputOne>> =
            Rc::new(RefCell::new(DependencyInputOne::new()));
        let mut vecinput = VecInput::new(Some(TIPS.to_string()), dependency_reader);
        vecinput.input()?;
        return Ok(vecinput.results()?.clone());
    }
}

/// 读取一个dependency的读取器
#[derive(Debug)]
struct DependencyInputOne;

impl InputFunc<Dependency> for DependencyInputOne {
    fn input(&mut self) -> Result<Dependency, ConsoleError> {
        return self.input_one();
    }
}

impl DependencyInputOne {
    pub fn new() -> Self {
        Self {}
    }

    fn input_name(&self) -> Result<String, ConsoleError> {
        let name = Input::new(
            Some("Please input the [name] of the dependency:".to_string()),
            None,
        )
        .input()?;
        return Ok(name);
    }

    fn input_version(&self) -> Result<String, ConsoleError> {
        let version = Input::new(
            Some("Please input the [version] of the dependency:".to_string()),
            None,
        )
        .input()?;
        return Ok(version);
    }

    fn input_one(&self) -> Result<Dependency, ConsoleError> {
        let name = self.input_name()?;
        let version = self.input_version()?;
        let mut dependency = Dependency::new(name, version);

        dependency.trim();
        // 验证输入
        dependency.validate().map_err(|e| {
            ConsoleError::InvalidInput(format!("Invalid dependency: {}", e.to_string()))
        })?;

        return Ok(dependency);
    }
}

#[derive(Debug)]
pub struct BuildConfigInput;

impl BuildConfigInput {
    pub fn new() -> Self {
        Self {}
    }

    fn input_command(&self) -> Result<String, ConsoleError> {
        println!("Please input the [build command] of the task:");
        let tips = format!("\nNote:
\t1. The command will be executed in the root directory of the source code.
\t2. After the command is executed, all files need to install to DragonOS should be placed in: [{}_TASKNAME_VERSION]\n",
 CacheDir::DADK_BUILD_CACHE_DIR_ENV_KEY_PREFIX);
        println!("{}", tips);
        let mut command = Input::new(Some("Build Command:".to_string()), None).input()?;
        command = command.trim().to_string();

        return Ok(command);
    }
}

impl InputFunc<BuildConfig> for BuildConfigInput {
    fn input(&mut self) -> Result<BuildConfig, ConsoleError> {
        println!("\nPlease input the [build_config] of the task:");

        // 读取build_config
        let command = self.input_command()?;
        let command = if command.is_empty() {
            None
        } else {
            Some(command)
        };
        let build_config = BuildConfig::new(command);
        return Ok(build_config);
    }
}

#[derive(Debug)]
struct InstallConfigInput;

impl InstallConfigInput {
    pub fn new() -> Self {
        Self {}
    }

    fn input_install_dir(&self) -> Result<Option<PathBuf>, ConsoleError> {
        let install_dir = Input::new(
            Some("Please input the [dir to install in DragonOS] of the task:".to_string()),
            None,
        )
        .input()?;
        let install_dir = install_dir.trim().to_string();
        let install_dir = if install_dir.is_empty() {
            None
        } else {
            Some(PathBuf::from(install_dir))
        };
        return Ok(install_dir);
    }
}

impl InputFunc<InstallConfig> for InstallConfigInput {
    fn input(&mut self) -> Result<InstallConfig, ConsoleError> {
        println!("\nPlease input the [install_config] of the task:");

        // 读取install dir
        let install_dir = self.input_install_dir()?;
        let mut install_config = InstallConfig::new(install_dir);
        install_config.trim();
        return Ok(install_config);
    }
}

#[derive(Debug)]
struct CleanConfigInput;

impl CleanConfigInput {
    pub fn new() -> Self {
        Self {}
    }

    fn input_clean_command(&self) -> Result<Option<String>, ConsoleError> {
        let clean_command = Input::new(
            Some("Please input the [clean command] of the task:".to_string()),
            None,
        )
        .input()?;
        let clean_command = clean_command.trim().to_string();
        let clean_command = if clean_command.is_empty() {
            None
        } else {
            Some(clean_command)
        };
        return Ok(clean_command);
    }
}

impl InputFunc<CleanConfig> for CleanConfigInput {
    fn input(&mut self) -> Result<CleanConfig, ConsoleError> {
        println!("\nPlease configure the [clean_config] of the task:");

        // 读取clean command
        let clean_command = self.input_clean_command()?;
        let mut clean_config = CleanConfig::new(clean_command);
        clean_config.trim();
        return Ok(clean_config);
    }
}

#[derive(Debug)]
struct TaskEnvInput;

impl TaskEnvInput {
    pub fn new() -> Self {
        Self {}
    }
}

impl InputFunc<Option<Vec<TaskEnv>>> for TaskEnvInput {
    fn input(&mut self) -> Result<Option<Vec<TaskEnv>>, ConsoleError> {
        const TIPS: &str = "Please configure the [ environment variables ] of the task:";
        println!();
        println!("{TIPS}");
        let env_reader: Rc<RefCell<TaskEnvInputOne>> =
            Rc::new(RefCell::new(TaskEnvInputOne::new()));
        let mut vecinput: VecInput<TaskEnv> = VecInput::new(Some(TIPS.to_string()), env_reader);
        vecinput.input()?;
        let result = vecinput.results()?.clone();
        // 不管是否有输入，都返回Some
        return Ok(Some(result));
    }
}

#[derive(Debug)]
struct TaskEnvInputOne;

impl TaskEnvInputOne {
    pub fn new() -> Self {
        Self {}
    }

    fn input_name(&self) -> Result<String, ConsoleError> {
        let name = Input::new(
            Some("Please input the [name] of the env:".to_string()),
            None,
        )
        .input()?;
        return Ok(name);
    }

    fn input_value(&self) -> Result<String, ConsoleError> {
        let value = Input::new(
            Some("Please input the [value] of the env:".to_string()),
            None,
        )
        .input()?;
        return Ok(value);
    }

    fn input_one(&self) -> Result<TaskEnv, ConsoleError> {
        let name = self.input_name()?;
        let value = self.input_value()?;
        let mut env = TaskEnv::new(name, value);

        env.trim();
        // 验证输入
        env.validate()
            .map_err(|e| ConsoleError::InvalidInput(format!("Invalid env: {}", e.to_string())))?;

        return Ok(env);
    }
}

impl InputFunc<TaskEnv> for TaskEnvInputOne {
    fn input(&mut self) -> Result<TaskEnv, ConsoleError> {
        let env = self.input_one()?;
        return Ok(env);
    }
}
