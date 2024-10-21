use console::CommandLineArgs;
use dadk_user::dadk_user_main;

extern crate anyhow;

mod console;

pub fn dadk_main() {
    let args = CommandLineArgs::parse();
    dadk_user_main();
}
