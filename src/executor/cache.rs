use std::{path::PathBuf, rc::Rc};

use log::info;

use crate::{
    parser::task::{CodeSource, DADKTask, TaskType},
    scheduler::SchedEntity,
    utils::lazy_init::Lazy,
};

use super::ExecutorError;

pub static CACHE_ROOT: Lazy<PathBuf> = Lazy::new();

/// # 初始化缓存根目录
///
/// ## 参数
///
/// - `path` 缓存根目录的路径
pub fn cache_root_init(path: Option<PathBuf>) -> Result<(), ExecutorError> {
    let cache_root: String;
    if path.is_none() {
        // 查询环境变量，是否有设置缓存根目录
        let env = std::env::var("DADK_CACHE_ROOT");
        if env.is_ok() {
            cache_root = env.unwrap();
        } else {
            // 如果没有设置环境变量，则使用默认值
            // 默认值为当前目录下的.cache目录
            let cwd = std::env::current_dir().map_err(|e| ExecutorError::IoError(e))?;
            let cwd = cwd.to_str();

            if cwd.is_none() {
                return Err(ExecutorError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Current dir is not a valid unicode string",
                )));
            }
            let cwd = cwd.unwrap();

            cache_root = format!("{}/.cache", cwd);
        }
    } else {
        // 如果有设置缓存根目录，则使用设置的值
        let path = path.unwrap();
        let x = path
            .to_str()
            .ok_or(ExecutorError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cache root dir is not a valid unicode string",
            )))?;
        cache_root = x.to_string();
    }

    let cache_root = PathBuf::from(cache_root);

    // 如果缓存根目录不存在，则创建
    if !cache_root.exists() {
        info!("Cache root dir not exists, create it: {:?}", cache_root);
        std::fs::create_dir_all(&cache_root).map_err(|e| ExecutorError::IoError(e))?;
    } else if !cache_root.is_dir() {
        // 如果缓存根目录不是目录，则报错
        return Err(ExecutorError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            format!("Cache root dir is not a directory: {:?}", cache_root),
        )));
    }

    // 初始化缓存根目录
    CACHE_ROOT.init(cache_root);

    // 设置环境变量
    std::env::set_var("DADK_CACHE_ROOT", CACHE_ROOT.get().to_str().unwrap());
    info!("Cache root dir: {:?}", CACHE_ROOT.get());
    return Ok(());
}

#[derive(Debug, Clone)]
pub struct CacheDir {
    #[allow(dead_code)]
    entity: Rc<SchedEntity>,
    pub path: PathBuf,
    pub cache_type: CacheDirType,
}

#[derive(Debug, Clone, Copy)]
pub enum CacheDirType {
    Build,
    Source,
}

impl CacheDir {
    pub const DADK_BUILD_CACHE_DIR_ENV_KEY_PREFIX: &'static str = "DADK_BUILD_CACHE_DIR";
    pub const DADK_SOURCE_CACHE_DIR_ENV_KEY_PREFIX: &'static str = "DADK_SOURCE_CACHE_DIR";
    pub fn new(entity: Rc<SchedEntity>, cache_type: CacheDirType) -> Result<Self, ExecutorError> {
        let task = entity.task();
        let path = Self::get_path(task, cache_type);

        let result = Self {
            entity,
            path,
            cache_type,
        };

        result.create()?;

        return Ok(result);
    }

    fn get_path(task: &DADKTask, cache_type: CacheDirType) -> PathBuf {
        let cache_root = CACHE_ROOT.get();
        let name_version = task.name_version();
        let cache_dir = match cache_type {
            CacheDirType::Build => {
                format!("{}/build/{}", cache_root.to_str().unwrap(), name_version)
            }
            CacheDirType::Source => {
                format!("{}/source/{}", cache_root.to_str().unwrap(), name_version)
            }
        };

        return PathBuf::from(cache_dir);
    }

    pub fn build_dir(entity: Rc<SchedEntity>) -> Result<PathBuf, ExecutorError> {
        return Ok(Self::new(entity, CacheDirType::Build)?.path);
    }

    pub fn source_dir(entity: Rc<SchedEntity>) -> Result<PathBuf, ExecutorError> {
        return Ok(Self::new(entity, CacheDirType::Source)?.path);
    }

    pub fn build_dir_env_key(entity: &Rc<SchedEntity>) -> Result<String, ExecutorError> {
        let name_version_env = entity.task().name_version_env();
        return Ok(format!(
            "{}_{}",
            Self::DADK_BUILD_CACHE_DIR_ENV_KEY_PREFIX,
            name_version_env
        ));
    }

    pub fn source_dir_env_key(entity: &Rc<SchedEntity>) -> Result<String, ExecutorError> {
        let name_version_env = entity.task().name_version_env();
        return Ok(format!(
            "{}_{}",
            Self::DADK_SOURCE_CACHE_DIR_ENV_KEY_PREFIX,
            name_version_env
        ));
    }

    pub fn need_source_cache(entity: &Rc<SchedEntity>) -> bool {
        let task_type = &entity.task().task_type;

        if let TaskType::BuildFromSource(cs) = task_type {
            match cs {
                CodeSource::Git(_) | CodeSource::Archive(_) => {
                    return true;
                }
                CodeSource::Local(_) => {
                    return false;
                }
            }
        } else if let TaskType::InstallFromPrebuilt(ps) = task_type {
            match ps {
                crate::parser::task::PrebuiltSource::Archive(_) => return false,
                crate::parser::task::PrebuiltSource::Local(_) => return false,
            }
        }
        unimplemented!("Not fully implemented task type: {:?}", task_type);
    }

    pub fn create(&self) -> Result<(), ExecutorError> {
        if !self.path.exists() {
            info!("Cache dir not exists, create it: {:?}", self.path);
            std::fs::create_dir_all(&self.path).map_err(|e| ExecutorError::IoError(e))?;
            info!("Cache dir: [{:?}] created.", self.path);
        } else if !self.path.is_dir() {
            // 如果路径类别不是目录，则报错
            return Err(ExecutorError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotADirectory,
                format!("Cache dir is not a directory: {:?}", self.path),
            )));
        }

        return Ok(());
    }

    /// 判断缓存目录是否为空
    pub fn is_empty(&self) -> Result<bool, ExecutorError> {
        let x = self
            .path
            .read_dir()
            .map_err(|e| ExecutorError::IoError(e))?;
        for _ in x {
            return Ok(false);
        }

        return Ok(true);
    }

    /// # 递归删除自身目录
    /// 递归删除自身目录，如果目录不存在，则忽略
    ///
    /// 请注意，这会删除整个目录，包括目录下的所有文件和子目录
    pub fn remove_self_recursive(&self) -> Result<(), ExecutorError> {
        let path = &self.path;
        if path.exists() {
            std::fs::remove_dir_all(path).map_err(|e| ExecutorError::IoError(e))?;
        }
        return Ok(());
    }
}
