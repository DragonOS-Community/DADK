use ::std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::PathBuf,
};
use std::collections::BTreeMap;

use crate::executor::{EnvMap, EnvVar};

use crate::executor::ExecutorError;

#[derive(Debug, Clone)]
pub struct Target {
    /// DADK任务名和其对应的存放在临时文件夹中的target文件路径
    pub name2tmp: BTreeMap<String, PathBuf>,
    /// DADK任务名和其对应的源target文件路径
    pub name2source: BTreeMap<String, PathBuf>,
}

impl Target {
    pub fn new() -> Target {
        let name2tmp: BTreeMap<String, PathBuf> = BTreeMap::new();
        let name2source: BTreeMap<String, PathBuf> = BTreeMap::new();
        return Target {
            name2tmp,
            name2source,
        };
    }

    // 将rust_target文件拷贝到/tmp中临时存放
    pub fn mvtotmp(
        &mut self,
        name: &str,
        rust_target: &str,
        file_path: &PathBuf,
        dragonos_dir: &PathBuf,
    ) -> Result<(), ExecutorError> {
        let dragonos_path = dragonos_dir
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string()
            .replace("/bin/sysroot", "");
        let path = self.target_path(&rust_target, &dragonos_path)?;
        let tmp_dadk = self.get_hash(file_path);
        self.name2source.insert(name.to_string(), path.clone());
        self.name2tmp
            .insert(name.to_string(), PathBuf::from(&tmp_dadk));
        self.create_and_copy(&path, &tmp_dadk)?;

        return Ok(());
    }

    // 获取rust_target文件的准确路径
    pub fn target_path(
        &self,
        rust_target: &str,
        dragonos_path: &str,
    ) -> Result<PathBuf, ExecutorError> {
        //如果是个路径，说明是用户自己的编译target文件，判断文件是否有效
        if rust_target.contains('/') | rust_target.contains('\\') {
            let path = PathBuf::from(rust_target);
            if path.exists() {
                return Ok(path);
            } else {
                return Err(ExecutorError::PrepareEnvError(
                    "Can not find the rust_target file.".to_string(),
                ));
            }
        }

        let default_target = format!("{}{}", dragonos_path, "/user/dadk/target.json");
        return Ok(PathBuf::from(default_target));
    }

    // 通过文件路径生成相应的哈希值
    pub fn get_hash(&self, file_path: &PathBuf) -> String {
        let mut hasher = DefaultHasher::new();
        let file_str = file_path.as_os_str().to_str().unwrap().to_string();
        file_str.hash(&mut hasher);
        let hash_string = format!("{:x}", hasher.finish());
        let first_six = &hash_string[0..6];
        //在/tmp文件夹下，创建当前DADK任务文件夹用于临时存放target
        let tmp_dadk = format!("/tmp/dadk{}/", first_six);
        return tmp_dadk;
    }

    // 在/tmp目录下创建临时文件夹存放各dadk对应的target文件
    pub fn create_and_copy(&self, from: &PathBuf, to: &str) -> Result<(), ExecutorError> {
        match fs::metadata(&to) {
            Ok(_) => (),
            Err(_) => {
                if let Err(e) = fs::create_dir(&to) {
                    return Err(ExecutorError::PrepareEnvError(format!("{}", e)));
                }
                let tmp_target = format!("{}{}", to, "target.json");
                if let Err(e) = fs::File::create(PathBuf::from(&tmp_target)) {
                    return Err(ExecutorError::PrepareEnvError(format!("{}", e)));
                }
                if let Err(e) = fs::copy(from, tmp_target) {
                    return Err(ExecutorError::PrepareEnvError(format!("{}", e)));
                }
            }
        }
        return Ok(());
    }

    // 设置DADK_RUST_TARGET_FILE环境变量
    pub fn set_env(&self, local_env: &mut EnvMap, name: &str) {
        let source = self
            .name2source
            .get(name)
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();
        local_env.add(EnvVar::new("DADK_RUST_TARGET_FILE".to_string(), source));
    }

    // 清理临时文件夹下的各target文件
    pub fn clean_tmpdadk(&self) -> Result<(), ExecutorError> {
        for (_, path) in self.name2tmp.iter() {
            if path.exists() {
                std::fs::remove_dir_all(path).map_err(|e| ExecutorError::IoError(e))?;
            }
        }
        return Ok(());
    }
}
