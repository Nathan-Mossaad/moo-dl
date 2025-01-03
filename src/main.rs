// Animations and logging
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod api;
mod downloader;

use api::login::{
    from_params::{CredentialFromRawParams, LoginParams},
    ApiCredential,
};
use api::rest_api::course_modules::Download;
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

    // TODO remove test credentials
    let cookie_jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let api_credential = ApiCredential {
    };
    let login_params = LoginParams::Raw(CredentialFromRawParams {
    });

    println!("{:?}", api_credential);
    println!("{:?}", login_params);

    let api = Api::builder()
        .api_credential(api_credential)
        .login_params(login_params)
        .cookie_jar(cookie_jar)
        .build()?;

    let course = api.core_course_get_contents(40121).await?;
    println!("{:?}", course);
    println!("\n\n\n");
    course
        .download(&api, &std::path::Path::new("./target/Course"))
        .await?;

    api.close_browser().await?;

    Ok(())
}
