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
pub struct IncidentUpdate {
    message: String,
    components: Vec<String>,
    started: String,
    status: String,
    notify: bool,
    statuses: Vec<ComponentStatus>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComponentResponse {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Incident {
    id: String,
    started: String,
    status: String,
    components: Vec<ComponentResponse>,
}

pub async fn set_incident_status(
    client: Client,
    page_id: String,
    incident: Incident,
    status: String,
) {
    match status.as_str() {
        "MONITORING" => {
            // todo: mark the incident as monitoring
            client
                .post(
                    format!(
                        "https://api.instatus.com/v1/{}/incidents/{}/incident-updates",
                        page_id, incident.id
                    )
                    .as_str(),
                )
                .body_json(&IncidentUpdate {
                    message: "A fix has been implemented. We are monitoring the service closely."
                        .to_string(),
                    components: incident
                        .components
                        .iter()
                        .map(|v| v.id.clone())
                        .collect::<Vec<String>>(),
                    started: incident.started,
                    status,
                    notify: true,
                    statuses: incident
                        .components
                        .iter()
                        .map(|v| ComponentStatus {
                            id: v.id.clone(),
                            status: "OPERATIONAL".to_string(),
                        })
                        .collect::<Vec<ComponentStatus>>(),
                })
                .unwrap()
                .await
                .unwrap();
        }
        "RESOLVED" => {
            // todo: mark the incident as resolved
            // todo: mark the components as online
        }
        &_ => {}
    }
}

pub async fn monitor(
    page: StatusPage,
    components: Vec<ComponentResponse>,
    metrics: HashMap<String, String>,
    client: Client,
    config: Config,
) {
    println!("🔍 Monitoring requests...");

    let mut active_incidents: Vec<Incident> = vec![];

    let mut active_monitoring_incidents: Vec<Incident> = vec![];

    let mut monitoring_elapsed: HashMap<String, u64> = HashMap::new();

    loop {
        active_incidents.clear();

        // get a list of incidents
        let incidents = client
            .get(format!("https://api.instatus.com/v1/{}/incidents", page.id))
            .header("Authorization", format!("Bearer {}", config.api_key))
            .recv_json::<Vec<Incident>>()
            .await
            .unwrap();

        // if the incident is still valid / active append it to the array of active incidents
        for incident in incidents {
            // an incident is still valid if the status is still one that is not resolved
            if incident.status == "IDENTIFIED" || incident.status == "MONITORING" {
                // if the status is still active
                active_incidents.push(incident.clone());

                if incident.status == "MONITORING" {
                    active_monitoring_incidents.push(incident.clone());
                }
            }
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

                if let MonitorType::Uptime = monitor.type_ {
                    println!(
                        "{}  {}{}✅  {} is up",
                        time.format("%H:%M:%S").bright_yellow(),
                        format!("{} ms", latency).bright_black(),
                        spacing,
                        name.bright_green()
                    );

                    // TODO: if this is in the list of active incidents and the monitoring time has elapsed, change it to resolved again
                    for incident in active_incidents.iter() {
                        if incident.status == "MONITORING" {
                            if monitoring_elapsed[&incident.id] == 0 {
                                set_incident_status(
                                    client.clone(),
                                    page.id.clone(),
                                    incident.clone(),
                                    "RESOLVED".to_string(),
                                )
                                .await;

                                println!("✅  {} marked as resolved", name.bright_green());
                            } else {
                                *monitoring_elapsed.get_mut(&incident.id).unwrap() -= 1;
                            }
                        } else {
                            // if the status is not monitoring, then change this incident to MONITORING
                            if incident.status.as_str() == "IDENTIFIED" {
                                monitoring_elapsed.insert(
                                    incident.id.clone(),
                                    config.incident_monitoring_time.unwrap(),
                                );

                                set_incident_status(
                                    client.clone(),
                                    page.id.clone(),
                                    incident.clone(),
                                    "MONITORING".to_string(),
                                )
                                .await;
                            }
                        }
                        // otherwise, if it is MONITORING and it's passed it's monitoring time, move it to resolved
                        // while marking things as resolved, ensure that all the components related to it are marked as resolved as well
                    }
                } else {
                    let start = Instant::now();

                    client
                        .post(format!(
                            "https://api.instatus.com/v1/{}/metrics/{}",
                            page.id,
                            metrics.get(name).unwrap_or_else(|| {
                                println!("Could not detect any metrics corresponding to {}", name,);

                                std::process::exit(1);
                            })
                        ))
                        .header("Authorization", format!("Bearer {}", config.api_key))
                        .body_json(&LatencyPost {
                            timestamp: time.timestamp_millis() as u64,
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
                            "{}  {}{}❌  {} is down",
                            time.format("%H:%M:%S").bright_yellow(),
                            format!("{} ms", latency).bright_black(),
                            spacing,
                            name.bright_red()
                        );

                        let start = Instant::now();

                        // check if the incident needs to be created
                        // if there's already an incident with the same name, we can skip it
                        let mut create_report = true;

                        for incident in active_incidents.iter() {
                            if incident
                                .components
                                .iter()
                                .map(|v| v.name.clone())
                                .any(|x| x == *name)
                            {
                                create_report = false;
                            }
                        }

                        if create_report {
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
                                    "{}  {}{}🎫  Successfully created incident for {} ",
                                    time.format("%H:%M:%S").bright_yellow(),
                                    format!("{} ms", start.elapsed().as_millis()).bright_black(),
                                    " ".repeat(
                                        MAX_MS_TIME as usize
                                            - start.elapsed().as_millis().to_string().len()
                                                as usize
                                    ),
                                    name.bright_green()
                                );
                            } else {
                                println!(
                                    "{}  {}{}❌  Failed to create incident for {} ",
                                    time.format("%H:%M:%S").bright_yellow(),
                                    format!("{} ms", start.elapsed().as_millis()).bright_black(),
                                    " ".repeat(
                                        MAX_MS_TIME as usize
                                            - start.elapsed().as_millis().to_string().len()
                                                as usize
                                    ),
                                    name.bright_red()
                                );
                            }
                        }
                    }
                    MonitorType::Latency => {
                        println!(
                            "{}  {}{}⚠️   Unable to measure latency for {}",
                            time.format("%H:%M:%S").bright_yellow(),
                            format!("{} ms", latency).bright_black(),
                            spacing,
                            name.bright_yellow()
                        );
                    }
                }
            };
        }

        sleep(Duration::from_secs(config.frequency));
    }
}
