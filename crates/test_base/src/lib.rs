pub extern crate test_context;

use std::path::PathBuf;

use simple_logger::SimpleLogger;
use test_context::TestContext;

pub struct BaseTestContext {
    /// 项目的根目录
    project_base_path: PathBuf,
}

impl BaseTestContext {
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
        self.abs_path("tests/data/dadk_config_v1")
    }
}

impl TestContext for BaseTestContext {
    fn setup() -> Self {
        let logger = SimpleLogger::new().with_level(log::LevelFilter::Debug);

        logger.init().unwrap();
        // 获取DADK项目的根目录
        let mut project_base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        project_base_path.pop();
        project_base_path.pop();
        BaseTestContext { project_base_path }
    }
}
