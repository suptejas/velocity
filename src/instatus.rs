use crate::config::Config;
use chrono::Local;
use owo_colors::OwoColorize;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use surf::Client;

pub async fn monitor(client: Client, config: Config) {
    println!("🔍 Monitoring requests...");

    loop {
        for (name, endpoint) in config.endpoints.iter() {
            let start = Instant::now();

            if client
                .get(endpoint)
                .header("Cache-Control", "no-cache")
                .send()
                .await
                .unwrap()
                .status()
                .is_success()
            {
                let time = Local::now();

                println!(
                    "{} {} ✅ {} is up",
                    time.format("%H:%M:%S").bright_yellow(),
                    format!("{:.2} ms", start.elapsed().as_millis()).bright_black(),
                    name.bright_green()
                );
            } else {
                let time = Local::now();

                println!(
                    "{} {} ❌ {} is down",
                    time.format("%H:%M:%S").bright_yellow(),
                    format!("{:.2} ms", start.elapsed().as_millis()).bright_red(),
                    name.bright_red()
                );

                // TODO: report downtime
                // using the Instatus API
            };
        }

        sleep(Duration::from_secs(config.frequency));
    }
}
