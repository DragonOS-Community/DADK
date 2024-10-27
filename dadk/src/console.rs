use dadk_user::clap::Parser;
use dadk_user::console::CommandLineArgs;

pub fn parse_commandline_args() -> CommandLineArgs {
    CommandLineArgs::parse()
}
