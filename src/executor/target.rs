use ::std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::PathBuf,
};
use log::error;
use std::io::Write;

use crate::{
    executor::{EnvMap, EnvVar},
    static_resources::INLINE_TARGETS,
};

use crate::executor::ExecutorError;

/// Target用于管理target文件
#[derive(Debug, Clone)]
pub struct Target {
    /// 临时target文件路径
    tmp_target_path: PathBuf,
}

impl Target {
    /// 创建target管理器
    ///
    /// ## 参数
    ///
    /// - `path` : 临时target文件路径
    ///
    /// ## 返回值
    ///
    /// target管理器
    pub fn new(path: PathBuf) -> Target {
        Target {
            tmp_target_path: path,
        }
    }

    /// 将用户的target文件或用户使用的内置target文件拷贝到临时target文件
    ///
    /// ## 参数
    ///
    /// - `rust_target` : dadk任务的rust_target字段值
    ///
    /// ## 返回值
    ///
    /// Ok(()) 拷贝成功
    /// Err(ExecutorError) 拷贝失败
    pub fn cp_to_tmp(&self, rust_target: &str) -> Result<(), ExecutorError> {
        // 创建临时target文件
        if Self::is_user_target(rust_target) {
            // 如果是用户的target文件，则从源target文件路径from拷贝
            let from = Self::user_target_path(&rust_target).unwrap();
            self.copy_to_tmp(&from)?;
        } else {
            // 如果使用的是内置target文件，则将默认的target文件写入临时target文件中
            self.write_to_tmp(rust_target)?;
        }
        return Ok(());
    }

    pub fn copy_to_tmp(&self, from: &PathBuf) -> Result<(), ExecutorError> {
        //创建临时target文件
        self.create_tmp_target()?;
        if let Err(e) = fs::copy(from, &self.tmp_target_path) {
            return Err(ExecutorError::PrepareEnvError(format!("{}", e)));
        }
        return Ok(());
    }

    pub fn write_to_tmp(&self, rust_target: &str) -> Result<(), ExecutorError> {
        // 创建临时target文件
        let file = self.create_tmp_target()?;
        let data = INLINE_TARGETS.lock().unwrap().get(rust_target)?;
        // 将target文件的二进制变量写入临时target文件中
        if file.is_some() {
            if let Err(e) = file.unwrap().write_all(&data) {
                return Err(ExecutorError::PrepareEnvError(format!("{}", e)));
            }
        }
        return Ok(());
    }

    /// 获取用户的target文件路径
    ///
    /// ## 参数
    ///
    /// - `rust_target` : dadk任务的rust_target字段值
    ///
    /// ## 返回值
    ///
    /// Ok(PathBuf) 用户target文件路径
    /// Err(ExecutorError) 用户target文件路径无效
    pub fn user_target_path(rust_target: &str) -> Result<PathBuf, ExecutorError> {
        // 如果是个路径，说明是用户自己的编译target文件，就判断文件是否有效
        let path = PathBuf::from(rust_target);
        if path.exists() {
            return Ok(path);
        } else {
            let path = path.as_path().to_str().unwrap();
            let errmsg = format!("Can not find the rust_target file: {}", path);
            error!("{errmsg}");
            return Err(ExecutorError::PrepareEnvError(errmsg));
        }
    }

    /// 通过dadk任务的路径生成相应的临时dadk目录路径
    ///
    /// ## 参数
    ///
    /// - `file_str` : dadk任务文件路径的字符串值
    ///
    /// ## 返回值
    ///
    /// 临时dadk目录路径
    pub fn tmp_dadk(file_str: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        file_str.hash(&mut hasher);
        let hash_string = format!("{:x}", hasher.finish());
        // 在/tmp文件夹下，创建当前DADK任务文件夹用于临时存放target
        let tmp_dadk = format!("/tmp/dadk{}/", hash_string);
        return PathBuf::from(tmp_dadk);
    }

    /// 创建临时target文件
    ///
    /// ## 参数
    ///
    /// - `tmp_target_path` : 临时target文件路径
    ///
    /// ## 返回值
    ///
    /// Ok(Some(fs::File)) 创建成功后的文件
    /// Ok(None) 临时target文件已经存在，不需要再创建
    /// Err(ExecutorError) 创建失败
    pub fn create_tmp_target(&self) -> Result<Option<fs::File>, ExecutorError> {
        // 先创建用于存放临时target文件的临时dadk目录
        let dir = Self::dir(&self.tmp_target_path);
        if fs::metadata(dir.clone()).is_err() {
            if let Err(e) = fs::create_dir(dir.clone()) {
                return Err(ExecutorError::PrepareEnvError(format!(
                    "{}{}",
                    dir.display(),
                    e
                )));
            }
        }

        // 如果临时target文件已经存在，则不需要返回文件，返回None即可
        if fs::metadata(&self.tmp_target_path).is_err() {
            if let Ok(file) = fs::File::create(&self.tmp_target_path) {
                return Ok(Some(file));
            }
        }

        return Ok(None);
    }

    /// 设置DADK_RUST_TARGET_FILE环境变量
    ///
    /// ## 参数
    ///
    /// - `local_envs` : 当前任务的环境变量列表
    ///
    /// ## 返回值
    ///
    /// 无
    pub fn prepare_env(&self, local_envs: &mut EnvMap) {
        let path = self
            .tmp_target_path()
            .as_path()
            .to_str()
            .unwrap()
            .to_string();
        local_envs.add(EnvVar::new("DADK_RUST_TARGET_FILE".to_string(), path));
    }

    /// 清理生成的临时dadk目录
    pub fn clean_tmpdadk(&self) -> Result<(), ExecutorError> {
        if self.tmp_target_path.exists() {
            let dir = Self::dir(&self.tmp_target_path);
            std::fs::remove_dir_all(&dir)
                .map_err(|e| ExecutorError::CleanError(format!("{}{}", dir.display(), e)))?;
        }
        return Ok(());
    }

    /// 获取文件所在的目录路径
    ///
    /// ## 参数
    ///
    /// - `path` : 文件路径
    ///
    /// ## 返回值
    ///
    /// 文件所在目录路径
    pub fn dir(path: &PathBuf) -> PathBuf {
        let path_str = path.as_path().to_str().unwrap();
        let index = path_str.rfind('/').unwrap();
        return PathBuf::from(path_str[..index + 1].to_string());
    }

    pub fn is_user_target(rust_target: &str) -> bool {
        // 如果包含.的话，说明用户使用的是自己的target文件，因为带有.json这样的字符
        return rust_target.contains('.');
    }

    pub fn tmp_target_path(&self) -> &PathBuf {
        return &self.tmp_target_path;
    }
}
