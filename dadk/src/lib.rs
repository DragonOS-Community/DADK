use console::parse_commandline_args;
use dadk_user::dadk_user_main;
use log::info;

extern crate anyhow;

mod console;

pub fn dadk_main() {
    let args = parse_commandline_args();
    info!("DADK run with args: {:?}", &args);

    dadk_user_main(args);
}
