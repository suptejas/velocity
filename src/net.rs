use std::{collections::HashMap, process::exit};

use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

use crate::{
    config::{Config, MonitorType},
    instatus::{Metric, StatusPage},
};

pub async fn pre_flight_network_test(config: &Config) -> (HashMap<String, String>, StatusPage) {
    let bar = ProgressBar::new(2).with_style(
        ProgressStyle::default_bar()
            .template("{msg}")
            .progress_chars("██"),
    );

    bar.set_message(format!(
        "> {} {}",
        "🔗".bright_yellow(),
        "api.instatus.com".to_string().bright_green().underline()
    ));

    let res: Vec<StatusPage> = surf::get("https://api.instatus.com/v1/pages".to_string())
        .header("Authorization", format!("Bearer {}", config.api_key))
        .recv_json()
        .await
        .unwrap_or_else(|error| {
            bar.abandon_with_message(format!("💥 could not connect to Instatus API: {}", error));

            exit(1);
        });

    let mut status_page: Option<StatusPage> = None;

    for page in res {
        if config.name == page.name {
            status_page = Some(page);
        }
    }

    if status_page.is_some() {
        bar.inc(1);

        let status_page = status_page.unwrap();

        let mut metric_loggers = vec![];

        for monitor in &config.monitors {
            if let MonitorType::Latency = monitor.1.type_ {
                metric_loggers.push(monitor.0);
            }
        }

        bar.set_message(format!(
            "> {} {}",
            "🔗".bright_yellow(),
            "api.instatus.com".to_string().bright_green().underline()
        ));

        let res: Vec<Metric> = surf::get(format!(
            "https://api.instatus.com/v1/{}/metrics",
            status_page.id
        ))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .recv_json()
        .await
        .unwrap_or_else(|error| {
            bar.abandon_with_message(format!("💥 could not connect to Instatus API: {}", error));

            exit(1);
        });

        let mut metrics = HashMap::new();

        for metric in res {
            if metric_loggers.contains(&&metric.name) {
                metrics.insert(metric.name, metric.id);
            }
        }

        bar.finish_with_message("✅  All checks passed");

        (metrics, status_page)
    } else {
        bar.abandon_with_message(format!(
            "❌  Could not find relevant status page with name {}",
            config.name
        ));

        std::process::exit(1);
    }
}
