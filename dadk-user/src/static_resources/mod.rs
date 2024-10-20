use std::sync::Mutex;

use log::error;

use crate::executor::ExecutorError;

lazy_static! {
    // 全局内置target
    pub static ref INLINE_TARGETS: Mutex<InlineTargets> = Mutex::new(InlineTargets::new());
}

/// #内置target
pub struct InlineTargets {
    /// 内置target文件列表
    inline_list: Vec<(String, &'static [u8])>,
}

impl InlineTargets {
    pub fn new() -> InlineTargets {
        let mut inline_targets = InlineTargets {
            inline_list: Vec::new(),
        };
        inline_targets.init();
        return inline_targets;
    }

    pub fn init(&mut self) {
        // 后续如果有新的内置target文件，只需要在inline_list中加入(目标三元组，binary数据)
        let x86_64_unknown_dragonos: &'static [u8] =
            include_bytes!("targets/rust/x86_64-unknown-dragonos.json");
        self.inline_list.push((
            "x86_64-unknown-dragonos".to_string(),
            x86_64_unknown_dragonos,
        ));
    }

    pub fn get(&self, rust_target: &str) -> Result<&'static [u8], ExecutorError> {
        // 通过rust_target找到对应的binary数据
        for (name, data) in &self.inline_list {
            if name == rust_target {
                return Ok(data);
            }
        }

        let errmsg = format!("无效的内置target文件: {}", rust_target);
        error!("{errmsg}");
        return Err(ExecutorError::PrepareEnvError(errmsg));
    }
}
