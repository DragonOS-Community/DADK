use std::{path::PathBuf, process::exit};

use derive_builder::Builder;
use log::error;
#[cfg(test)]
use test_base::{test_context::TestContext, BaseTestContext};

use crate::{console::Action, executor::cache::cache_root_init, scheduler::task_deque::TASK_DEQUE};

#[derive(Debug, Builder)]
#[builder(setter(into))]
pub struct DadkExecuteContext {
    /// DragonOS sysroot在主机上的路径
    sysroot_dir: Option<PathBuf>,
    /// DADK任务配置文件所在目录
    config_dir: Option<PathBuf>,
    /// 要执行的操作
    action: Action,
    /// 并行线程数量
    thread_num: Option<usize>,
    /// dadk缓存根目录
    cache_dir: Option<PathBuf>,

    #[cfg(test)]
    base_test_context: Option<BaseTestContext>,
}

impl DadkExecuteContext {
    pub fn init(&self) {
        // 初始化缓存目录
        let r: Result<(), crate::executor::ExecutorError> =
            cache_root_init(self.cache_dir().cloned());
        if r.is_err() {
            error!("Failed to init cache root: {:?}", r.unwrap_err());
            exit(1);
        }

        if let Some(thread) = self.thread_num() {
            TASK_DEQUE.lock().unwrap().set_thread(thread);
        }

        if self.action() == &Action::New {
            return;
        }

        if self.config_dir().is_none() {
            error!("Config dir is required for action: {:?}", self.action());
            exit(1);
        }

        if self.sysroot_dir().is_none() {
            error!(
                "dragonos sysroot dir is required for action: {:?}",
                self.action()
            );
            exit(1);
        }
    }
    pub fn sysroot_dir(&self) -> Option<&PathBuf> {
        self.sysroot_dir.as_ref()
    }

    pub fn config_dir(&self) -> Option<&PathBuf> {
        self.config_dir.as_ref()
    }

    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn thread_num(&self) -> Option<usize> {
        self.thread_num
    }

    pub fn cache_dir(&self) -> Option<&PathBuf> {
        self.cache_dir.as_ref()
    }
}

#[cfg(test)]
pub trait TestContextExt: TestContext {
    fn base_context(&self) -> &BaseTestContext;

    fn execute_context(&self) -> &DadkExecuteContext;
}

#[cfg(test)]
pub struct DadkExecuteContextTestBuildV1 {
    context: DadkExecuteContext,
}

#[cfg(test)]
impl TestContext for DadkExecuteContextTestBuildV1 {
    fn setup() -> Self {
        let base_context = BaseTestContext::setup();
        let context = DadkExecuteContextBuilder::default()
            .sysroot_dir(Some(base_context.fake_dragonos_sysroot()))
            .config_dir(Some(base_context.config_v1_dir()))
            .action(Action::Build)
            .thread_num(None)
            .cache_dir(Some(base_context.fake_dadk_cache_root()))
            .base_test_context(Some(base_context))
            .build()
            .expect("Failed to build DadkExecuteContextTestBuildV1");
        context.init();
        DadkExecuteContextTestBuildV1 { context }
    }
}

#[cfg(test)]
impl TestContextExt for DadkExecuteContextTestBuildV1 {
    fn base_context(&self) -> &BaseTestContext {
        self.base_test_context.as_ref().unwrap()
    }

    fn execute_context(&self) -> &DadkExecuteContext {
        &self.context
    }
}

macro_rules! impl_deref_for_test_context {
    ($context:ty) => {
        #[cfg(test)]
        impl std::ops::Deref for $context {
            type Target = DadkExecuteContext;

            fn deref(&self) -> &Self::Target {
                &self.context
            }
        }

        #[cfg(test)]
        impl std::ops::DerefMut for $context {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.context
            }
        }
    };
}

impl_deref_for_test_context!(DadkExecuteContextTestBuildV1);
