// Animations and logging
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use reqwest::Client;

mod api;
use api::login::Credential;
use api::Api;

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

    let cookie_jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let credential = Credential::from_username_password(

        None,
        cookie_jar,
    )
    .await?;

    println!("{:?}", credential);

    let client = Client::new();
    let mut api = Api {
        api_credential: credential.into(),
        client,
        user_id: None,
    };
    api.get_user_id().await?;
    println!("{:?}", api);
    let courses = api
        .core_enrol_get_users_courses()
        .await?
        .iter()
        .map(|c| c.id)
        .collect::<Vec<u64>>();
    println!("{:?}", api.core_course_get_contents_mult(courses).await?);

    Ok(())
}
