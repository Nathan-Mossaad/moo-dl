use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about = "A fast moodle course downloader")]
pub struct Cli {
    #[clap(long, help = "Disable animations")]
    pub no_animation: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[clap(about = "Synchronize the course")]
    Sync {
        #[clap(long, help = "Path to config", default_value = ".moo-dl-config.yml")]
        config_path: PathBuf,
    },

    #[clap(about = "Create a config file")]
    Setup {},
}
