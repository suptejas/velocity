use crate::config::{Config, MonitorType};
use chrono::Local;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    thread::sleep,
    time::{Duration, Instant},
};
use surf::Client;

const MAX_MS_TIME: u8 = 5;

/// A status page object from the Instatus API
#[derive(Serialize, Deserialize, Debug)]
pub struct StatusPage {
    /// ID of the status page
    pub id: String,
    /// Name of the status page
    pub name: String,
}

/// A metric object from the Instatus API
#[derive(Serialize, Deserialize, Debug)]
pub struct Metric {
    /// ID of the metric
    pub id: String,
    /// Name of the metric
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LatencyPost {
    timestamp: u64,
    value: u128,
}

pub async fn monitor(
    page: StatusPage,
    metrics: HashMap<String, String>,
    client: Client,
    config: Config,
) {
    println!("🔍 Monitoring requests...");

    loop {
        for (name, monitor) in config.monitors.iter() {
            let start = Instant::now();

            if client
                .get(&monitor.url)
                .header("Cache-Control", "no-cache, no-store, must-revalidate")
                .header("Pragma", "no-cache")
                .header("Expires", "0")
                .send()
                .await
                .unwrap()
                .status()
                .is_success()
            {
                // latency for the request
                let latency = start.elapsed().as_millis();
                // current time
                let time = Local::now();

                // calculate spacing
                let spacing = " ".repeat(MAX_MS_TIME as usize - latency.to_string().len() as usize);

                match monitor.type_ {
                    MonitorType::Uptime => {
                        println!(
                            "{}  {}{}✅  {} is up",
                            time.format("%H:%M:%S").bright_yellow(),
                            format!("{} ms", latency).bright_black(),
                            spacing,
                            name.bright_green()
                        );
                    }
                    MonitorType::Latency => {
                        let start = Instant::now();

                        client
                            .post(format!(
                                "https://api.instatus.com/v1/{}/metrics/{}",
                                page.id, metrics[name]
                            ))
                            .header("Authorization", format!("Bearer {}", config.api_key))
                            .body_json(&LatencyPost {
                                timestamp: time.timestamp() as u64,
                                value: latency,
                            })
                            .unwrap()
                            .await
                            .unwrap()
                            .body_string()
                            .await
                            .unwrap();

                        println!(
                            "{}  {}{}📡  {} latency updated to {}",
                            time.format("%H:%M:%S").bright_yellow(),
                            format!("{} ms", start.elapsed().as_millis()).bright_black(),
                            " ".repeat(
                                MAX_MS_TIME as usize
                                    - start.elapsed().as_millis().to_string().len() as usize
                            ),
                            name.bright_green(),
                            format!("{} ms", latency).bright_black(),
                        );
                    }
                }
            } else {
                // latency for the request
                let latency = start.elapsed().as_millis();
                // current time
                let time = Local::now();

                // calculate spacing
                let spacing = " ".repeat(MAX_MS_TIME as usize - latency.to_string().len() as usize);

                println!(
                    "{}  {}{}❌  {} is down",
                    time.format("%H:%M:%S").bright_yellow(),
                    format!("{} ms", latency).bright_black(),
                    spacing,
                    name.bright_green()
                );

                // TODO: report downtime
                // using the Instatus API
            };
        }

        sleep(Duration::from_secs(config.frequency));
    }
}
