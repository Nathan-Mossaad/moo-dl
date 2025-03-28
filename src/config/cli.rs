use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about = "A fast moodle course downloader")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(about = "Synchronize the course")]
    Sync {
        #[clap(long, help = "Path to config", default_value = ".moo-dl-config.yml")]
        config_path: PathBuf,

        // Todo
        #[clap(long, help = "Disable animations")]
        no_animation: bool,
    },

    #[clap(about = "Create a config file")]
    Setup {},
}
