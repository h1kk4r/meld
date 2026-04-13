use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(name = "meld", version, about = "Lua-configurable fetch/status tool")]
pub struct Cli {
    #[arg(
        long,
        value_name = "PATH",
        help = "Load init.lua from an explicit path"
    )]
    pub config: Option<PathBuf>,

    #[arg(long, help = "Print the config path that would be used and exit")]
    pub show_config_path: bool,

    #[arg(long, help = "Print config, backend, and path diagnostics and exit")]
    pub diagnostics: bool,

    #[arg(
        long,
        help = "Print a documented init.lua using the current config values and exit"
    )]
    pub print_config: bool,

    #[arg(
        long,
        help = "Log in to Spotify and cache the OAuth token next to init.lua"
    )]
    pub spotify_login: bool,
}
