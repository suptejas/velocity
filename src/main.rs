use std::time::Duration;

use config::Config;
use owo_colors::OwoColorize;
use surf::Client;

use tracing::Level;
use tracing_subscriber::EnvFilter;

pub mod config;
pub mod net;
pub mod velocity;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter(
            EnvFilter::try_from_env("LOG_LEVEL").unwrap_or_else(|_| EnvFilter::default()),
        )
        .without_time()
        .init();

    smol::block_on(async {
        println!(
            "📖 Reading configuration variables from {}",
            "velocity.json".bright_magenta()
        );

        let config = Config::from_file("velocity.json");

        println!("✈️  Running {} setup...", "pre-flight".bright_cyan());

        let (metrics, components, page) = net::pre_flight_setup(&config).await;

        println!("🌊 Spinning up network client");

        let client: Client = surf::Config::new()
            .set_timeout(Some(Duration::from_secs(
                config.max_connection_timeout.unwrap() as u64,
            )))
            .try_into()
            .unwrap_or_else(|err| {
                eprintln!("💥 failed to initialise network client: {}", err);

                std::process::exit(1);
            });

        velocity::monitor(page, components, metrics, client, config).await;
    });
}
