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

#[derive(Serialize, Deserialize, Debug)]
pub struct ComponentStatus {
    id: String,
    status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncidentPost {
    name: String,
    message: String,
    components: Vec<String>,
    started: String,
    status: String,
    notify: bool,
    statuses: Vec<ComponentStatus>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComponentResponse {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Incident {
    id: String,
}

pub async fn monitor(
    page: StatusPage,
    components: Vec<ComponentResponse>,
    metrics: HashMap<String, String>,
    client: Client,
    config: Config,
) {
    println!("🔍 Monitoring requests...");

    let mut active_incidents = vec![];

    loop {
        active_incidents.clear();

        let incidents = client
            .get(format!("https://api.instatus.com/v1/{}/incidents", page.id))
            .header("Authorization", format!("Bearer {}", config.api_key))
            .recv_json::<Vec<Incident>>()
            .await
            .unwrap();

        for incident in incidents {
            active_incidents.push(incident.id);
        }

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

                        // TODO: if this is in the list of active incidents, change it to resolved again
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

                match monitor.type_ {
                    MonitorType::Uptime => {
                        println!(
                            "{}  {}{}❌  {} is down, creating incident",
                            time.format("%H:%M:%S").bright_yellow(),
                            format!("{} ms", latency).bright_black(),
                            spacing,
                            name.bright_red()
                        );

                        let mut impacted_components = vec![];

                        for component in &components {
                            if component.name == *name {
                                impacted_components.push(component.id.to_owned());
                            }
                        }

                        let mut impacted_components_statuses = vec![];

                        for component in &impacted_components {
                            impacted_components_statuses.push(ComponentStatus {
                                id: component.clone(),
                                status: "MAJOROUTAGE".to_string(),
                            });
                        }

                        let start = Instant::now();

                        for incident in active_incidents {}

                        let res = client
                            .post(format!("https://api.instatus.com/v1/{}/incidents", page.id))
                            .header("Authorization", format!("Bearer {}", config.api_key))
                            .body_json(&IncidentPost {
                                name: format!("{} Issues", name),
                                message: format!("We've identified issues with the {}. Engineers have been notified.", name),
                                components: impacted_components,
                                started: time.format("%Y-%m-%d %H:%M:%S.%3f").to_string(),
                                status: String::from("IDENTIFIED"),
                                notify: true,
                                statuses: impacted_components_statuses,
                            })
                            .unwrap()
                            .send()
                            .await
                            .unwrap_or_else(|e| {
                                eprintln!("{e}");
                                std::process::exit(1);
                            });

                        if res.status().is_success() {
                            println!(
                                "{}  {}{}🎫  {} successfully created incident",
                                time.format("%H:%M:%S").bright_yellow(),
                                format!("{} ms", start.elapsed().as_millis()).bright_black(),
                                " ".repeat(
                                    MAX_MS_TIME as usize
                                        - start.elapsed().as_millis().to_string().len() as usize
                                ),
                                name.bright_green()
                            );
                        } else {
                            println!(
                                "{}  {}{}❌  {} failed to create incident",
                                time.format("%H:%M:%S").bright_yellow(),
                                format!("{} ms", start.elapsed().as_millis()).bright_black(),
                                " ".repeat(
                                    MAX_MS_TIME as usize
                                        - start.elapsed().as_millis().to_string().len() as usize
                                ),
                                name.bright_red()
                            );
                        }
                    }
                    MonitorType::Latency => {
                        println!(
                            "{}  {}{}⚠️   Unable to measure latency for {}",
                            time.format("%H:%M:%S").bright_yellow(),
                            format!("{} ms", latency).bright_black(),
                            spacing,
                            name.bright_red()
                        );
                    }
                }
            };
        }

        sleep(Duration::from_secs(config.frequency));
    }
}
