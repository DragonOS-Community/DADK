//! # DADK - DragonOS Application Development Kit
//! # DragonOS 应用开发工具
//!
//! ## 简介
//!
//! DADK是一个用于开发DragonOS应用的工具包，设计目的是为了让开发者能够更加方便的开发DragonOS应用。
//!
//! ### DADK做什么？
//!
//! - 自动配置libc等编译用户程序所需的环境
//! - 自动处理软件库的依赖关系
//! - 自动处理软件库的编译
//! - 一键将软件库安装到DragonOS系统中
//!
//! ### DADK不做什么？
//!
//! - DADK不会帮助开发者编写代码
//! - DADK不提供任何开发DragonOS应用所需的API。这部分工作由libc等库来完成
//!
//! ## License
//!
//! DADK is licensed under the [GPLv2 License](LICENSE).

#![feature(io_error_more)]

#[macro_use]
extern crate lazy_static;
extern crate log;
extern crate serde;
extern crate serde_json;
extern crate simple_logger;

use std::{fs, path::PathBuf, process::exit};

use clap::Parser;
use executor::source::GitSource;
use log::{error, info};
use parser::task::{
    BuildConfig, CleanConfig, CodeSource, DADKTask, Dependency, InstallConfig, TaskType,
};
use simple_logger::SimpleLogger;

use crate::{console::CommandLineArgs, executor::cache::cache_root_init, scheduler::Scheduler};

mod console;
mod executor;
mod parser;
mod scheduler;
mod utils;

fn main() {
    SimpleLogger::new().init().unwrap();
    // generate_tmp_dadk();
    info!("DADK Starting...");
    let args = CommandLineArgs::parse();

    info!("DADK run with args: {:?}", &args);
    // DragonOS sysroot在主机上的路径
    let dragonos_dir = args.dragonos_dir.as_ref();
    let config_dir = args.config_dir.as_ref();
    let action = args.action;
    info!(
        "DragonOS sysroot dir: {}",
        dragonos_dir
            .as_ref()
            .map_or_else(|| "None".to_string(), |d| d.display().to_string())
    );
    info!(
        "Config dir: {}",
        config_dir
            .as_ref()
            .map_or_else(|| "None".to_string(), |d| d.display().to_string())
    );
    info!("Action: {:?}", action);

    // 初始化缓存目录
    let r = cache_root_init(args.cache_dir);
    if r.is_err() {
        error!("Failed to init cache root: {:?}", r.unwrap_err());
        exit(1);
    }

    let config_dir = args.config_dir.unwrap_or_else(|| {
        error!("Config dir not specified");
        exit(1);
    });

    let dragonos_dir = args.dragonos_dir.unwrap_or_else(|| {
        error!("DragonOS sysroot dir not specified");
        exit(1);
    });

    let mut parser = parser::Parser::new(config_dir);
    let r = parser.parse();
    if r.is_err() {
        exit(1);
    }
    let tasks: Vec<(PathBuf, DADKTask)> = r.unwrap();
    // info!("Parsed tasks: {:?}", tasks);

    let scheduler = Scheduler::new(dragonos_dir, action, tasks);
    if scheduler.is_err() {
        exit(1);
    }

    let r = scheduler.unwrap().run();
    if r.is_err() {
        exit(1);
    }
}

#[allow(dead_code)]
fn generate_tmp_dadk() {
    let x = DADKTask {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        build: BuildConfig {
            build_command: Some("echo test".to_string()),
        },
        install: InstallConfig {
            in_dragonos_path: PathBuf::from("/bin"),
        },
        clean: CleanConfig::new(None),
        depends: vec![Dependency {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
        }],
        description: "test".to_string(),
        // task_type: TaskType::BuildFromSource(CodeSource::Archive(ArchiveSource::new(
        //     "123".to_string(),
        // ))),
        task_type: TaskType::BuildFromSource(CodeSource::Git(GitSource::new(
            "123".to_string(),
            Some("master".to_string()),
            None,
        ))),
        envs: None,
    };
    let x = serde_json::to_string(&x).unwrap();
    fs::write("test.json", x).unwrap();
}
