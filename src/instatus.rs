use crate::config::Config;
use owo_colors::OwoColorize;
use std::{thread::sleep, time::Duration};
use surf::Client;

pub async fn serve(client: Client, config: Config) {
    println!("🚚 Serving requests...");

    loop {
        for (name, endpoint) in config.endpoints.iter() {
            if client
                .get(endpoint)
                .send()
                .await
                .unwrap()
                .status()
                .is_success()
            {
                println!("✅ {} is up", name.bright_green());
            } else {
                println!("🚫 {} is down", name.bright_red());
            };
        }

        sleep(Duration::from_secs(config.frequency));
    }
}
