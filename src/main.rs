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
//! DADK is licensed under the [GPLv2 License](https://www.gnu.org/licenses/old-licenses/gpl-2.0.html).
//!
//! ## 快速开始
//!
//! ### 安装DADK
//!
//! DADK是一个Rust程序，您可以通过Cargo来安装DADK。
//!
//! ```shell
//! # 从GitHub安装最新版
//! cargo install --git https://github.com/DragonOS-Community/DADK.git
//!
//! # 从crates.io下载
//! cargo install dadk
//!
//! ```
//!
//! ## DADK的工作原理
//!
//! DADK使用(任务名，任务版本）来标识每个构建目标。当使用DADK构建DragonOS应用时，DADK会根据用户的配置文件，自动完成以下工作：
//!
//! - 解析配置文件，生成DADK任务列表
//! - 根据DADK任务列表，进行拓扑排序。这一步会自动处理软件库的依赖关系。
//! - 收集环境变量信息，并根据DADK任务列表，设置全局环境变量、任务环境变量。
//! - 根据拓扑排序后的DADK任务列表，自动执行任务。
//!
//! ### DADK与环境变量
//!
//! 环境变量的设置是DADK能正常工作的关键因素之一，您可以在您的编译脚本中，通过引用环境变量，来获得其他软件库的编译信息。
//! 这是使得您的应用能够自动依赖其他软件库的关键一步。
//!
//! 只要您的编译脚本能够正确地引用环境变量，DADK就能够自动处理软件库的依赖关系。
//!
//! DADK会设置以下全局环境变量：
//!
//! - `DADK_CACHE_ROOT`：DADK的缓存根目录。您可以在编译脚本中，通过引用该环境变量，来获得DADK的缓存根目录。
//! - `DADK_BUILD_CACHE_DIR_任务名_任务版本`：DADK的任务构建结果缓存目录。当您要引用其他软件库的构建结果时，可以通过该环境变量来获得。
//! 同时，您也要在构建您的app时，把构建结果放到您的软件库的构建结果缓存目录（通过对应的环境变量获得）中。
//! - `DADK_SOURCE_CACHE_DIR_任务名_任务版本`：DADK的某个任务的源码目录。当您要引用其他软件库的源码目录时，可以通过该环境变量来获得。
//!
//! 同时，DADK会为每个任务设置其自身在配置文件中指定的环境变量。
//!
//! #### 全局环境变量命名格式
//!
//! 全局环境变量中的任务名和任务版本，都会被转换为大写字母，并对特殊字符进行替换。替换表如下：
//!
//! | 原字符 | 替换字符 |
//! | ------ | -------- |
//! | `.`    | `_`      |
//! | `-`    | `_`      |
//! | `\t`   | `_`      |
//! | 空格   | `_`      |
//! | `+`    | `_`      |
//! | `*`    | `_`      |
//!
//! **举例**：对于任务`libc-0.1.0`，其构建结果的全局环境变量名为`DADK_BUILD_CACHE_DIR_LIBC_0_1_0`。
//!
//!
//! ## TODO
//!
//! - 支持从在线归档文件下载源码、构建好的软件库
//! - 支持自动更新
//! - 完善clean命令的逻辑

#![feature(extract_if)]
#![feature(io_error_more)]

#[macro_use]
extern crate lazy_static;
extern crate log;
extern crate serde;
extern crate serde_json;
extern crate simple_logger;

use std::{path::PathBuf, process::exit};

use clap::Parser;

use log::{error, info};
use parser::task::DADKTask;
use simple_logger::SimpleLogger;

use crate::{
    console::{interactive::InteractiveConsole, CommandLineArgs},
    executor::cache::cache_root_init,
    scheduler::{task_deque::TASK_DEQUE, Scheduler},
};

mod console;
mod executor;
mod parser;
mod scheduler;
pub mod static_resources;
mod utils;

fn main() {
    logger_init();
    // generate_tmp_dadk();
    info!("DADK Starting...");
    let args = CommandLineArgs::parse();

    info!("DADK run with args: {:?}", &args);
    // DragonOS sysroot在主机上的路径
    let dragonos_dir = args.dragonos_dir.clone();
    let config_dir = args.config_dir.clone();
    let action = args.action;
    let thread = args.thread;
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
    info!(
        "Thread num: {}",
        thread
            .as_ref()
            .map_or_else(|| "None".to_string(), |d| d.to_string())
    );

    match action {
        console::Action::New => {
            let r = InteractiveConsole::new(dragonos_dir.clone(), config_dir.clone(), action).run();
            if r.is_err() {
                error!("Failed to run interactive console: {:?}", r.unwrap_err());
                exit(1);
            }
            exit(0);
        }
        _ => {}
    }

    if let Some(thread) = thread {
        TASK_DEQUE.lock().unwrap().set_thread(thread);
    }

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

/// 初始化日志系统
fn logger_init() {
    // 初始化日志系统，日志级别为Info
    // 如果需要调试，可以将日志级别设置为Debug
    let logger = SimpleLogger::new().with_level(log::LevelFilter::Info);

    logger.init().unwrap();
}
