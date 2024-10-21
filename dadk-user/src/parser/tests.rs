use test_base::{
    global::BaseGlobalTestContext,
    test_context::{self as test_context, test_context},
};
use tests::task::{BuildConfig, TargetArch, TaskType};

use crate::executor::source::LocalSource;

use super::*;

#[test_context(BaseGlobalTestContext)]
#[test]
fn parse_normal_v1(ctx: &mut BaseGlobalTestContext) {
    let parser = Parser::new(ctx.config_v1_dir());
    let config_file = ctx.config_v1_dir().join("app_normal_0_1_0.dadk");
    let result = parser.parse_config_file(&config_file);

    assert!(result.is_ok(), "Error: {:?}", result);

    let result = result.unwrap();

    assert_eq!(result.name, "app_normal");
    assert_eq!(result.version, "0.1.0");
    assert_eq!(result.description, "A normal app");

    let expected_task_type = TaskType::BuildFromSource(task::CodeSource::Local(LocalSource::new(
        PathBuf::from("tests/data/apps/app_normal"),
    )));

    assert_eq!(result.task_type, expected_task_type,);

    assert_eq!(result.depends.len(), 0);

    let expected_build_config: BuildConfig = BuildConfig::new(Some("bash build.sh".to_string()));
    assert_eq!(result.build, expected_build_config);

    let expected_install_config = task::InstallConfig::new(Some(PathBuf::from("/")));

    assert_eq!(result.install, expected_install_config);
    let expected_clean_config = task::CleanConfig::new(None);

    assert_eq!(result.clean, expected_clean_config);

    assert!(result.envs.is_some());
    assert_eq!(result.envs.as_ref().unwrap().len(), 0);

    assert_eq!(result.build_once, false);
    assert_eq!(result.install_once, false);
}

#[test_context(BaseGlobalTestContext)]
#[test]
fn target_arch_field_has_one_v1(ctx: &mut BaseGlobalTestContext) {
    let parser = Parser::new(ctx.config_v1_dir());
    let config_file = ctx
        .config_v1_dir()
        .join("app_target_arch_x86_64_0_1_0.dadk");
    let result = parser.parse_config_file(&config_file);

    assert!(result.is_ok(), "Error: {:?}", result);

    let result = result.unwrap();

    assert_eq!(result.name, "app_target_arch_x86_64");
    assert_eq!(result.version, "0.1.0");

    assert_eq!(result.target_arch.len(), 1);
    assert_eq!(result.target_arch[0], TargetArch::X86_64);
}

#[test_context(BaseGlobalTestContext)]
#[test]
fn target_arch_field_has_one_uppercase_v1(ctx: &mut BaseGlobalTestContext) {
    let parser = Parser::new(ctx.config_v1_dir());
    let config_file = ctx
        .config_v1_dir()
        .join("app_target_arch_x86_64_uppercase_0_1_0.dadk");
    let result = parser.parse_config_file(&config_file);

    assert!(result.is_ok(), "Error: {:?}", result);

    let result = result.unwrap();

    assert_eq!(result.name, "app_target_arch_x86_64_uppercase");
    assert_eq!(result.version, "0.1.0");

    assert_eq!(result.target_arch.len(), 1);
    assert_eq!(result.target_arch[0], TargetArch::X86_64);
}

#[test_context(BaseGlobalTestContext)]
#[test]
fn target_arch_field_empty_should_failed_v1(ctx: &mut BaseGlobalTestContext) {
    let parser = Parser::new(ctx.config_v1_dir());
    let config_file = ctx
        .config_v1_dir()
        .join("app_target_arch_empty_should_fail_0_1_0.dadk");
    let result = parser.parse_config_file(&config_file);

    assert!(
        result.is_err(),
        "parse_config_file should return error when target_arch field in config file is empty"
    );
}
