use dadk_config::{self, manifest::DadkManifest};
use test_base::{
    dadk_config::DadkConfigTestContext,
    test_context::{self as test_context, test_context},
};

const TEMPLATES_DIR: &str = "templates";
const DADK_MANIFEST_FILE_NAME: &str = "dadk-manifest.toml";

/// 测试加载模板目录中的 dadk-manifest.toml 文件，验证它能被加载成功，并且已经包含了所有字段
#[test_context(DadkConfigTestContext)]
#[test]
fn test_load_dadk_manifest_template(ctx: &DadkConfigTestContext) {
    let manifest_path = ctx.abs_path(&format!("{TEMPLATES_DIR}/{DADK_MANIFEST_FILE_NAME}"));
    assert_eq!(manifest_path.exists(), true);
    assert_eq!(manifest_path.is_file(), true);
    let manifest = DadkManifest::load(&manifest_path).expect("Failed to load manifest");
    assert_eq!(manifest.used_default, false);
}
