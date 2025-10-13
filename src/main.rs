mod api;
mod config;
mod download;
mod generate_config;
mod login;
mod status_bar;
mod sync;
mod update;

use std::sync::Arc;

// Animations and logging
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{
    filter::EnvFilter, layer::SubscriberExt, reload, util::SubscriberInitExt,
};

pub use anyhow::Result;

use config::cli;
use config::sync_config::{Config, read_config};
use generate_config::generate_config;

#[tokio::main]
async fn main() -> crate::Result<()> {
    // Start logging
    let indicatif_layer = IndicatifLayer::new();

    let base_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("moo_dl=info"));
    // Create a reloadable layer that wraps the EnvFilter.
    let (reload_layer, reload_handle) = reload::Layer::new(base_filter);

    // Parse CLI arguments.
    let cli = <cli::Cli as clap::Parser>::parse();

    // Build and initialize the global subscriber.
    if cli.no_animation {
        tracing_subscriber::registry()
            .with(reload_layer)
            .with(tracing_subscriber::fmt::layer().with_ansi(false).compact())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(reload_layer)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(indicatif_layer.get_stderr_writer())
                    .compact(),
            )
            .with(indicatif_layer)
            .init();
    }

    match cli.command {
        cli::Command::Sync { config_path } => {
            let config = Arc::new(read_config(&config_path)?);

            let shutdown_config = config.clone();
            tokio::spawn(async move {
                tokio::signal::ctrl_c()
                    .await
                    .expect("failed to listen for ctrl+c");

                shutdown_config
                    .write_log_to_file(true)
                    .await
                    .expect("Failed to write log file");

                std::process::exit(130);
            });

            // Start Login
            let login_handle = Config::login_thread(config.clone()).await;
            // Spawn youtube downloader threads
            let youtube_handle = Config::create_youtube_download_threads(config.clone()).await;

            // Get download path
            let download_path = match &config.dir {
                // We can safely unwrap, as the config can't be at /
                Some(path) => &config_path.parent().unwrap().join(path),
                None => config_path.parent().unwrap(),
            };

            // Start sync
            Config::download_courses(config.clone(), &download_path).await;

            // Allow youtube downloader threads to stop gracefully
            config.youtube_queue.sender.close();
            youtube_handle.wait_for_completion().await;
            // Start chromium shutdown
            config.chromium_close().await;

            // Stop outputting more messages
            reload_handle
                .modify(|filter| *filter = EnvFilter::new("off"))
                .expect("Failed to update the filter");
            // Show Status bar
            println!("{}", config.status_bar.get_overview().await);
            config.write_log_to_file(false).await?;

            // Wait till chromium is stopped gracefully
            config.chromium_wait().await;
            // Kill tasks that are no longer needed.
            login_handle.abort();
        }
        cli::Command::Setup {} => {
            generate_config().await?;
        }
    }

    Ok(())
}
