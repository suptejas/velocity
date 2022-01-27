use crate::config::{Config, MonitorType};
use chrono::Local;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use surf::Client;

/// A status page object from the Instatus API
#[derive(Serialize, Deserialize, Debug)]
pub struct StatusPage {
    /// ID of the status page
    pub id: String,
    /// Name of the status page
    pub name: String,
}

pub async fn monitor(client: Client, config: Config) {
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
                let time = Local::now();

                match monitor.type_ {
                    MonitorType::Uptime => {
                        println!(
                            "{} {} ✅ {} is up",
                            time.format("%H:%M:%S").bright_yellow(),
                            format!("{:.2} ms", start.elapsed().as_millis()).bright_black(),
                            name.bright_green()
                        );
                    }
                    MonitorType::Latency => {
                        // client.post("https://api.instatus.com")

                        // first latency should be the time taken to log the latency
                        // second latency should be the actual latency
                        println!(
                            "{} {} ✅ {} latency updated to {}",
                            time.format("%H:%M:%S").bright_yellow(),
                            format!("{:.2} ms", start.elapsed().as_millis()).bright_black(),
                            name.bright_green(),
                            format!("{:.4} ms", start.elapsed().as_millis()).bright_black(),
                        );
                    }
                }
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
