use std::{collections::BTreeMap, env::Vars, rc::Rc, sync::RwLock};

use log::{debug, info};

use crate::{
    executor::cache::CacheDir,
    scheduler::{SchedEntities, SchedEntity},
};

use self::cache::CacheDirType;

pub mod cache;

lazy_static! {
    // 全局环境变量的列表
    pub static ref ENV_LIST: RwLock<EnvMap> = RwLock::new(EnvMap::new());
}

#[derive(Debug)]
pub struct Executor {
    entity: Rc<SchedEntity>,
    local_envs: EnvMap,
    build_dir: CacheDir,
    source_dir: Option<CacheDir>,
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
    pub fn new(entity: Rc<SchedEntity>) -> Result<Self, ExecutorError> {
        let local_envs = EnvMap::new();
        let build_dir = CacheDir::new(entity.clone(), CacheDirType::Build)?;

        let source_dir = if CacheDir::need_source_cache(&entity) {
            Some(CacheDir::new(entity.clone(), CacheDirType::Source)?)
        } else {
            None
        };

        let result: Executor = Self {
            entity,
            local_envs,
            build_dir,
            source_dir,
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
    pub fn execute(&self) -> Result<(), ExecutorError> {
        // todo!("Execute task: {:?}", self.entity.task());
        info!("Execute task: {}", self.entity.task().name_version());

        return Ok(());
    }
}

#[derive(Debug)]
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

    pub fn get(&self, key: &str) -> Option<&EnvVar> {
        self.envs.get(key)
    }

    pub fn add_vars(&mut self, vars: Vars) {
        for (key, value) in vars {
            self.add(EnvVar::new(key, value));
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

impl EnvVar {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

#[derive(Debug)]
pub enum ExecutorError {
    /// # 准备环境变量错误
    PrepareEnvError,
    IoError(std::io::Error),
}

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

        let build_dir_key = CacheDir::build_dir_env_key(entity.clone())?;
        env_list.add(EnvVar::new(
            build_dir_key,
            build_dir.to_str().unwrap().to_string(),
        ));

        // 如果需要源码缓存目录，则导出
        if CacheDir::need_source_cache(entity) {
            let source_dir = CacheDir::source_dir(entity.clone())?;
            let source_dir_key = CacheDir::source_dir_env_key(entity.clone())?;
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
