use std::{fmt::Debug, path::PathBuf};

use log::error;

use crate::console::new_config::NewConfigCommand;

use super::{Action, ConsoleError};

#[derive(Debug)]
#[allow(dead_code)]
pub struct InteractiveConsole {
    /// DragonOS sysroot在主机上的路径
    dragonos_dir: Option<PathBuf>,
    /// DADK任务配置文件所在目录
    config_dir: Option<PathBuf>,
    /// 要执行的操作
    action: Action,
}

pub trait InteractiveCommand {
    fn run(&mut self) -> Result<(), ConsoleError>;
}

impl InteractiveConsole {
    pub fn new(dragonos_dir: Option<PathBuf>, config_dir: Option<PathBuf>, action: Action) -> Self {
        Self {
            dragonos_dir,
            config_dir,
            action,
        }
    }

    pub fn run(&self) -> Result<(), ConsoleError> {
        println!("\nWelcome to DADK interactive console!\n");
        match self.action {
            Action::New => {
                let mut cmd = NewConfigCommand::new(self.config_dir.clone());
                cmd.run()
            }
            _ => {
                let msg = format!(
                    "Action '{:?}' not supported in interactive console",
                    self.action
                );
                error!("{msg}");
                return Err(ConsoleError::CommandError(msg));
            }
        }
    }
}

pub trait InputFunc<T: Debug + Sized> {
    /// # 读取用户输入
    fn input(&mut self) -> Result<T, ConsoleError>;
    /// # 读取用户输入，直到返回值合法
    fn input_until_valid(&mut self) -> Result<T, ConsoleError> {
        loop {
            let task_type = self.input();
            if task_type.is_ok() {
                return task_type;
            } else {
                if let Err(ConsoleError::InvalidInput(e)) = task_type {
                    error!("{}", e);
                    continue;
                } else {
                    return task_type;
                }
            }
        }
    }

    /// # 读取用户输入，最多重试指定次数
    ///
    /// 如果重试次数超过指定次数，则返回错误Err(ConsoleError::RetryLimitExceeded)
    #[allow(dead_code)]
    fn input_with_retry(&mut self, retry: usize) -> Result<T, ConsoleError> {
        for _ in 0..retry {
            let task_type = self.input();
            if task_type.is_ok() {
                return task_type;
            } else {
                if let Err(ConsoleError::InvalidInput(e)) = task_type {
                    error!("{}", e);
                    continue;
                } else {
                    return task_type;
                }
            }
        }
        return Err(ConsoleError::RetryLimitExceeded(format!(
            "Retry limit exceeded."
        )));
    }
}
