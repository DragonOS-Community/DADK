use ::std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::PathBuf,
};
use log::error;
use std::io::Write;

use crate::executor::{EnvMap, EnvVar};

use crate::executor::ExecutorError;

use super::TARGET_BINARY;

/// TargetManager用于管理生成的临时target文件
#[derive(Debug, Clone)]
pub struct TargetManager {
    /// 临时target文件路径
    tmp_target_path: PathBuf,
}

impl TargetManager {
    /// 创建target管理器
    ///
    /// ## 参数
    ///
    /// - `path` : 临时target文件路径
    ///
    /// ## 返回值
    ///
    /// target管理器
    pub fn new(path: PathBuf) -> TargetManager {
        TargetManager {
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
    pub fn mv_to_tmp(&self, rust_target: &str) -> Result<(), ExecutorError> {
        if Self::is_user_target(rust_target) {
            // 如果是用户的target文件，则从源target文件路径from拷贝
            let from = Self::user_target_path(&rust_target).unwrap();
            self.copy_to_tmp(&from)?;
        } else {
            // 如果使用的是内置target文件，则将默认的target文件写入临时target文件中
            self.write_to_tmp()?;
        }
        return Ok(());
    }

    pub fn copy_to_tmp(&self, from: &PathBuf) -> Result<(), ExecutorError> {
        if let Err(e) = fs::copy(from, &self.tmp_target_path) {
            return Err(ExecutorError::PrepareEnvError(format!("{}", e)));
        }
        return Ok(());
    }

    pub fn write_to_tmp(&self) -> Result<(), ExecutorError> {
        match fs::File::create(&self.tmp_target_path) {
            Ok(mut file) => {
                // 将target文件的二进制变量写入临时target文件中
                if let Err(e) = file.write_all(&TARGET_BINARY) {
                    return Err(ExecutorError::PrepareEnvError(format!("{}", e)));
                }
            }
            Err(e) => {
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
        let first_six = &hash_string[0..6];
        // 在/tmp文件夹下，创建当前DADK任务文件夹用于临时存放target
        let tmp_dadk = format!("/tmp/dadk{}/", first_six);
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
    /// Ok(()) 创建成功
    /// Err(ExecutorError) 创建失败
    pub fn create_tmp_target(tmp_target_path: &PathBuf) -> Result<(), ExecutorError> {
        // 先创建用于存放临时target文件的临时dadk目录
        let dir = Self::dir(&tmp_target_path);
        if let Err(e) = fs::create_dir(dir) {
            return Err(ExecutorError::PrepareEnvError(format!("{}", e)));
        }
        if let Err(e) = fs::File::create(&tmp_target_path) {
            return Err(ExecutorError::PrepareEnvError(format!("{}", e)));
        }
        return Ok(());
    }

    /// 设置DADK_RUST_TARGET_FILE环境变量
    ///
    /// ## 参数
    ///
    /// - `local_envs` : 当前任务的环境变量列表
    /// - `path` : 环境变量值，即target文件路径
    ///
    /// ## 返回值
    ///
    /// 无
    pub fn set_env(&self, local_envs: &mut EnvMap, path: &PathBuf) {
        let path = path.as_path().to_str().unwrap().to_string();
        local_envs.add(EnvVar::new("DADK_RUST_TARGET_FILE".to_string(), path));
    }

    /// 清理生成的临时dadk目录
    pub fn clean_tmpdadk(&self) -> Result<(), ExecutorError> {
        if self.tmp_target_path.exists() {
            std::fs::remove_dir_all(Self::dir(&self.tmp_target_path))
                .map_err(|e| ExecutorError::IoError(e))?;
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
        // 如果包含目录层次的话，说明用户使用的是自己的target文件
        return rust_target.contains('/') || rust_target.contains('\\');
    }

    pub fn tmp_target_path(&self) -> &PathBuf {
        return &self.tmp_target_path;
    }
}
