use std::time::Duration;

use config::Config;
use owo_colors::OwoColorize;
use surf::Client;

pub mod config;
pub mod instatus;
pub mod net;

fn main() {
    smol::block_on(async {
        println!(
            "📖 Reading configuration variables from {}",
            "velocity.json".bright_magenta()
        );

        let config = Config::from_file("velocity.json");

        println!("✈️  Running {} checks...", "pre-flight".bright_cyan());

        let page = net::pre_flight_network_test(&config).await;

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

        instatus::monitor(client, config).await;
    });
}
