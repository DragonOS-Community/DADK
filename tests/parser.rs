
use log::error;
use test_base::{test_context::{self as test_context, test_context}, BaseTestContext};

const CONFIG_V1_DIR: &str = "tests/data/dadk_config_v1";

#[test_context(BaseTestContext)]
#[test]
fn test_parser(ctx: &mut BaseTestContext){
    let mut parser = dadk::parser::Parser::new(ctx.abs_path(CONFIG_V1_DIR));
    let result = parser.parse();
    let cwd = std::env::current_dir().unwrap();

    log::debug!("Current working directory: {:?}", cwd);
    if let Err(e) = result {
        error!("Error: {:?}", e);
        assert!(false);
    }
}