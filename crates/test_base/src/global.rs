use std::path::PathBuf;

use test_context::TestContext;

#[derive(Debug, Clone)]
pub struct BaseGlobalTestContext {
    /// 项目的根目录
    project_base_path: PathBuf,
}

impl BaseGlobalTestContext {
    const CONFIG_V1_DIR: &'static str = "tests/data/dadk_config_v1";
    const FAKE_DRAGONOS_SYSROOT: &'static str = "tests/data/fake_dragonos_sysroot";
    const FAKE_DADK_CACHE_ROOT: &'static str = "tests/data/fake_dadk_cache_root";

    /// 获取项目的根目录
    pub fn project_base_path(&self) -> &PathBuf {
        &self.project_base_path
    }

    /// 获取项目目录下的文件的的绝对路径
    pub fn abs_path(&self, relative_path: &str) -> PathBuf {
        self.project_base_path.join(relative_path)
    }

    /// 获取`xxx.dadk`配置文件的目录
    pub fn config_v1_dir(&self) -> PathBuf {
        self.abs_path(Self::CONFIG_V1_DIR)
    }

    fn ensure_fake_dragonos_dir_exist(&self) {
        let fake_dragonos_dir = self.fake_dragonos_sysroot();
        if !fake_dragonos_dir.exists() {
            std::fs::create_dir_all(&fake_dragonos_dir).ok();
        }
    }

    fn ensure_fake_dadk_cache_root_exist(&self) {
        std::env::set_var(
            "DADK_CACHE_ROOT",
            self.fake_dadk_cache_root().to_str().unwrap(),
        );
        let fake_dadk_cache_root = self.fake_dadk_cache_root();
        if !fake_dadk_cache_root.exists() {
            std::fs::create_dir_all(&fake_dadk_cache_root).ok();
        }
    }

    pub fn fake_dadk_cache_root(&self) -> PathBuf {
        self.abs_path(Self::FAKE_DADK_CACHE_ROOT)
    }

    /// 获取假的DragonOS sysroot目录
    pub fn fake_dragonos_sysroot(&self) -> PathBuf {
        self.abs_path(Self::FAKE_DRAGONOS_SYSROOT)
    }
}

impl TestContext for BaseGlobalTestContext {
    fn setup() -> Self {
        env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

        // 获取DADK项目的根目录
        let mut project_base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        project_base_path.pop();
        project_base_path.pop();
        // 设置workdir
        std::env::set_current_dir(&project_base_path).expect("Failed to setup project_base_path");

        let r = BaseGlobalTestContext { project_base_path };
        r.ensure_fake_dragonos_dir_exist();
        r.ensure_fake_dadk_cache_root_exist();
        r
    }
}
