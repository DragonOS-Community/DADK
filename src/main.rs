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

extern crate log;
extern crate serde;
extern crate serde_json;
extern crate simple_logger;

use std::{fs, path::PathBuf, process::exit};

use clap::{Parser, Subcommand};
use log::{error, info};
use parser::task::{
    ArchiveSource, BuildConfig, CodeSource, DADKTask, Dependency, GitSource, InstallConfig,
    TaskType,
};
use simple_logger::SimpleLogger;

mod parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct CommandLineArgs {
    #[arg(short, long, value_parser = parse_check_dir_exists)]
    pub dragonos_dir: PathBuf,
    #[arg(short, long, value_parser = parse_check_dir_exists)]
    config_dir: PathBuf,

    #[command(subcommand)]
    action: Action,
}

/// @brief 要执行的操作
#[derive(Debug, Subcommand)]
enum Action {
    Build,
    Clean,
    Install,
    Uninstall,
}

/// @brief 检查目录是否存在
fn parse_check_dir_exists(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(format!("Path '{}' not exists", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("Path '{}' is not a directory", path.display()));
    }

    return Ok(path);
}

fn main() {
    SimpleLogger::new().init().unwrap();
    // generate_tmp_dadk();
    info!("DADK Starting...");
    let args = CommandLineArgs::parse();

    info!("DADK run with args: {:?}", &args);
    let dragonos_dir = args.dragonos_dir;
    let config_dir = args.config_dir;
    let action = args.action;
    info!("DragonOS dir: {}", dragonos_dir.display());
    info!("Config dir: {}", config_dir.display());
    info!("Action: {:?}", action);

    let mut parser = parser::Parser::new(config_dir);
    let r = parser.parse();
    if r.is_err() {
        error!("{:?}", r.unwrap_err());
        exit(1);
    }
    let tasks: Vec<DADKTask> = r.unwrap();
    info!("Tasks: {:?}", tasks);

}

fn generate_tmp_dadk() {
    let x = DADKTask {
        name: "test".to_string(),
        version: "0.1.0".to_string(),
        build: BuildConfig {
            build_command: "echo test".to_string(),
        },
        install: InstallConfig {
            in_dragonos_path: PathBuf::from("/bin"),
            install_command: "echo test".to_string(),
        },
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
            "master".to_string(),
            None,
        ))),
    };
    let x = serde_json::to_string(&x).unwrap();
    fs::write("test.json", x).unwrap();
}
