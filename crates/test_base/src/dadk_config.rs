use std::path::PathBuf;

use test_context::TestContext;

#[derive(Debug, Clone)]
pub struct DadkConfigTestContext {
    /// 项目的根目录
    test_base_path: PathBuf,
}

impl DadkConfigTestContext {
    /// 获取项目的根目录
    pub fn test_base_path(&self) -> &PathBuf {
        &self.test_base_path
    }

    /// 获取项目目录下的文件的的绝对路径
    pub fn abs_path(&self, relative_path: &str) -> PathBuf {
        self.test_base_path.join(relative_path)
    }
}

impl TestContext for DadkConfigTestContext {
    fn setup() -> Self {
        env_logger::try_init_from_env(env_logger::Env::default().default_filter_or("info")).ok();

        // 获取dadk-config包的根目录
        let mut test_base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_base_path.pop();
        test_base_path.pop();
        test_base_path.push("dadk-config");
        log::debug!(
            "DadkConfigTestContext setup: project_base_path={:?}",
            test_base_path
        );
        // 设置workdir
        std::env::set_current_dir(&test_base_path).expect("Failed to setup test base path");

        let r = DadkConfigTestContext { test_base_path };

        r
    }
}
