mod config;

// Animations and logging
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub use anyhow::Result;

use config::cli;
use config::sync_config::read_config;

#[tokio::main]
async fn main() -> crate::Result<()> {
    // Start logging
    let indicatif_layer = IndicatifLayer::new();
    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
    );
    let cli = <cli::Cli as clap::Parser>::parse();

    if cli.no_animation {
        subscriber
            .with(tracing_subscriber::fmt::layer().with_ansi(false).compact())
            .init();
    } else {
        subscriber
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(indicatif_layer.get_stderr_writer())
                    .compact(),
            )
            .with(indicatif_layer)
            .init();
    }

    //
    match cli.command {
        cli::Command::Sync { config_path } => {
            // TODO
            let _config = read_config(config_path)?;
        }
        cli::Command::Setup {} => {
            panic!("TODO: Implement Setup");
        }
    }

    Ok(())
}
