// Animations and logging
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod api;
use api::login::Credential;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> crate::Result<()> {
    // Start logging
    let indicatif_layer = IndicatifLayer::new();
    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
    );
    // if cli.ansi_only {
    //     subscriber
    //         .with(tracing_subscriber::fmt::layer().with_ansi(false).compact())
    //         .init();
    // } else {
    subscriber
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(indicatif_layer.get_stderr_writer())
                .compact(),
        )
        .with(indicatif_layer)
        .init();
    // }

    let credential =
        Credential::from_graphical("https://moodle.rwth-aachen.de/".to_string(), None, None)
            .await?;
    println!("{:?}", credential);

    Ok(())
}
