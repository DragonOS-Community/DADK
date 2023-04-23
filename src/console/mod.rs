use clap::Subcommand;

/// @brief 要执行的操作
#[derive(Debug, Subcommand, Clone, Copy)]
pub enum Action {
    Build,
    Clean,
    Install,
    Uninstall,
}
