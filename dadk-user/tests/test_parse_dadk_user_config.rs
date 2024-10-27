use std::path::PathBuf;

use dadk_user::{
    executor::source::{ArchiveSource, GitSource, LocalSource},
    parser::{
        task::{
            BuildConfig, CleanConfig, CodeSource, DADKTask, Dependency, InstallConfig,
            PrebuiltSource, TargetArch, TaskEnv, TaskType,
        },
        Parser,
    },
};
use test_base::{
    dadk_user::DadkUserTestContext,
    test_context::{self as test_context, test_context},
};

const DADK_USER_TEST_BUILD_LOCAL: &str = "build_from_source/test_local.toml";
const DADK_USER_TEST_BUILD_GIT: &str = "build_from_source/test_git.toml";
const DADK_USER_TEST_BUILD_ARCHIVE: &str = "build_from_source/test_archive.toml";
const DADK_USER_TEST_INSTALL_LOCAL: &str = "install_from_prebuilt/test_local.toml";
const DADK_USER_TEST_INSTALL_ARCHIVE: &str = "install_from_prebuilt/test_archive.toml";

/// 测试解析DADK用户配置文件
#[test_context(DadkUserTestContext)]
#[test]
fn test_parse_dadk_user_config_build_local(ctx: &mut DadkUserTestContext) {
    let config_file = ctx.templates_dir().join(DADK_USER_TEST_BUILD_LOCAL);
    assert!(config_file.exists());
    assert!(config_file.is_file());
    let r = Parser::parse_toml_file(&config_file);
    assert!(r.is_ok());
    let mut parsed_dadk_task = r.unwrap();
    let mut dadk_task = DADKTask {
        name: "test_local".to_string(),
        version: "0.2.0".to_string(),
        description: "".to_string(),
        build_once: true,
        install_once: true,
        task_type: TaskType::BuildFromSource(CodeSource::Local(LocalSource::new(PathBuf::from(
            "apps/test",
        )))),
        rust_target: None,
        depends: vec![
            Dependency {
                name: "depend1".to_string(),
                version: "0.1.1".to_string(),
            },
            Dependency {
                name: "depend2".to_string(),
                version: "0.1.2".to_string(),
            },
        ],
        build: BuildConfig::new(Some("make install".to_string())),
        install: InstallConfig::new(Some(PathBuf::from("/bin"))),
        clean: CleanConfig::new(Some("make clean".to_string())),
        envs: Some(vec![
            TaskEnv::new("PATH".to_string(), "/usr/bin".to_string()),
            TaskEnv::new("LD_LIBRARY_PATH".to_string(), "/usr/lib".to_string()),
        ]),
        target_arch: vec![TargetArch::try_from("x86_64").unwrap()],
    };

    parsed_dadk_task.target_arch.sort();
    dadk_task.target_arch.sort();
    parsed_dadk_task.depends.sort();
    dadk_task.depends.sort();
    if let Some(envs) = &mut parsed_dadk_task.envs {
        envs.sort();
    }
    if let Some(envs) = &mut dadk_task.envs {
        envs.sort();
    }
    assert_eq!(parsed_dadk_task, dadk_task)
}

#[test_context(DadkUserTestContext)]
#[test]
fn test_parse_dadk_user_config_build_git(ctx: &mut DadkUserTestContext) {
    let config_file = ctx.templates_dir().join(DADK_USER_TEST_BUILD_GIT);
    assert!(config_file.exists());
    assert!(config_file.is_file());
    let r = Parser::parse_toml_file(&config_file);
    assert!(r.is_ok());
    let mut parsed_dadk_task = r.unwrap();
    let mut dadk_task = DADKTask {
        name: "test_git".to_string(),
        version: "0.2.0".to_string(),
        description: "".to_string(),
        build_once: true,
        install_once: true,
        task_type: TaskType::BuildFromSource(CodeSource::Git(GitSource::new(
            "https://git.mirrors.dragonos.org.cn/DragonOS-Community/test_git.git".to_string(),
            Some("test".to_string()),
            Some("01cdc56863".to_string()),
        ))),
        rust_target: None,
        depends: vec![
            Dependency {
                name: "depend1".to_string(),
                version: "0.1.1".to_string(),
            },
            Dependency {
                name: "depend2".to_string(),
                version: "0.1.2".to_string(),
            },
        ],
        build: BuildConfig::new(Some("make install".to_string())),
        install: InstallConfig::new(Some(PathBuf::from("/bin"))),
        clean: CleanConfig::new(Some("make clean".to_string())),
        envs: Some(vec![
            TaskEnv::new("PATH".to_string(), "/usr/bin".to_string()),
            TaskEnv::new("LD_LIBRARY_PATH".to_string(), "/usr/lib".to_string()),
        ]),
        target_arch: vec![TargetArch::try_from("x86_64").unwrap()],
    };

    parsed_dadk_task.target_arch.sort();
    dadk_task.target_arch.sort();
    parsed_dadk_task.depends.sort();
    dadk_task.depends.sort();
    if let Some(envs) = &mut parsed_dadk_task.envs {
        envs.sort();
    }
    if let Some(envs) = &mut dadk_task.envs {
        envs.sort();
    }
    assert_eq!(parsed_dadk_task, dadk_task)
}

#[test_context(DadkUserTestContext)]
#[test]
fn test_parse_dadk_user_config_build_archive(ctx: &mut DadkUserTestContext) {
    let config_file = ctx.templates_dir().join(DADK_USER_TEST_BUILD_ARCHIVE);
    assert!(config_file.exists());
    assert!(config_file.is_file());
    let r = Parser::parse_toml_file(&config_file);
    assert!(r.is_ok());
    let mut parsed_dadk_task = r.unwrap();
    let mut dadk_task = DADKTask {
        name: "test_archive".to_string(),
        version: "0.2.0".to_string(),
        description: "".to_string(),
        build_once: true,
        install_once: true,
        task_type: TaskType::BuildFromSource(CodeSource::Archive(ArchiveSource::new(
            "https://url".to_string(),
        ))),
        rust_target: None,
        depends: vec![
            Dependency {
                name: "depend1".to_string(),
                version: "0.1.1".to_string(),
            },
            Dependency {
                name: "depend2".to_string(),
                version: "0.1.2".to_string(),
            },
        ],
        build: BuildConfig::new(Some("make install".to_string())),
        install: InstallConfig::new(Some(PathBuf::from("/bin"))),
        clean: CleanConfig::new(Some("make clean".to_string())),
        envs: Some(vec![
            TaskEnv::new("PATH".to_string(), "/usr/bin".to_string()),
            TaskEnv::new("LD_LIBRARY_PATH".to_string(), "/usr/lib".to_string()),
        ]),
        target_arch: vec![TargetArch::try_from("x86_64").unwrap()],
    };

    parsed_dadk_task.target_arch.sort();
    dadk_task.target_arch.sort();
    parsed_dadk_task.depends.sort();
    dadk_task.depends.sort();
    if let Some(envs) = &mut parsed_dadk_task.envs {
        envs.sort();
    }
    if let Some(envs) = &mut dadk_task.envs {
        envs.sort();
    }
    assert_eq!(parsed_dadk_task, dadk_task)
}

#[test_context(DadkUserTestContext)]
#[test]
fn test_parse_dadk_user_config_install_local(ctx: &mut DadkUserTestContext) {
    let config_file = ctx.templates_dir().join(DADK_USER_TEST_INSTALL_LOCAL);
    assert!(config_file.exists());
    assert!(config_file.is_file());
    let r = Parser::parse_toml_file(&config_file);
    assert!(r.is_ok());
    let mut parsed_dadk_task = r.unwrap();
    let mut dadk_task = DADKTask {
        name: "test_local".to_string(),
        version: "0.2.0".to_string(),
        description: "".to_string(),
        build_once: true,
        install_once: true,
        task_type: TaskType::InstallFromPrebuilt(PrebuiltSource::Local(LocalSource::new(
            PathBuf::from("/home/dev/demo"),
        ))),
        rust_target: None,
        depends: vec![
            Dependency {
                name: "depend1".to_string(),
                version: "0.1.1".to_string(),
            },
            Dependency {
                name: "depend2".to_string(),
                version: "0.1.2".to_string(),
            },
        ],
        build: BuildConfig::new(Some("make install".to_string())),
        install: InstallConfig::new(Some(PathBuf::from("/bin"))),
        clean: CleanConfig::new(Some("make clean".to_string())),
        envs: Some(vec![
            TaskEnv::new("PATH".to_string(), "/usr/bin".to_string()),
            TaskEnv::new("LD_LIBRARY_PATH".to_string(), "/usr/lib".to_string()),
        ]),
        target_arch: vec![TargetArch::try_from("x86_64").unwrap()],
    };

    parsed_dadk_task.target_arch.sort();
    dadk_task.target_arch.sort();
    parsed_dadk_task.depends.sort();
    dadk_task.depends.sort();
    if let Some(envs) = &mut parsed_dadk_task.envs {
        envs.sort();
    }
    if let Some(envs) = &mut dadk_task.envs {
        envs.sort();
    }
    assert_eq!(parsed_dadk_task, dadk_task)
}

#[test_context(DadkUserTestContext)]
#[test]
fn test_parse_dadk_user_config_install_archive(ctx: &mut DadkUserTestContext) {
    let config_file = ctx.templates_dir().join(DADK_USER_TEST_INSTALL_ARCHIVE);
    assert!(config_file.exists());
    assert!(config_file.is_file());
    let r = Parser::parse_toml_file(&config_file);
    assert!(r.is_ok());
    let mut parsed_dadk_task = r.unwrap();
    let mut dadk_task = DADKTask {
        name: "test_archive".to_string(),
        version: "0.2.0".to_string(),
        description: "".to_string(),
        build_once: true,
        install_once: true,
        task_type: TaskType::InstallFromPrebuilt(PrebuiltSource::Archive(ArchiveSource::new(
            "https://url".to_string(),
        ))),
        rust_target: None,
        depends: vec![
            Dependency {
                name: "depend1".to_string(),
                version: "0.1.1".to_string(),
            },
            Dependency {
                name: "depend2".to_string(),
                version: "0.1.2".to_string(),
            },
        ],
        build: BuildConfig::new(Some("make install".to_string())),
        install: InstallConfig::new(Some(PathBuf::from("/bin"))),
        clean: CleanConfig::new(Some("make clean".to_string())),
        envs: Some(vec![
            TaskEnv::new("PATH".to_string(), "/usr/bin".to_string()),
            TaskEnv::new("LD_LIBRARY_PATH".to_string(), "/usr/lib".to_string()),
        ]),
        target_arch: vec![TargetArch::try_from("x86_64").unwrap()],
    };

    parsed_dadk_task.target_arch.sort();
    dadk_task.target_arch.sort();
    parsed_dadk_task.depends.sort();
    dadk_task.depends.sort();
    if let Some(envs) = &mut parsed_dadk_task.envs {
        envs.sort();
    }
    if let Some(envs) = &mut dadk_task.envs {
        envs.sort();
    }
    assert_eq!(parsed_dadk_task, dadk_task)
}
