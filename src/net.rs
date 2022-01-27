use std::process::exit;

use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

use crate::{config::Config, instatus::StatusPage};

pub async fn pre_flight_network_test(config: &Config) -> StatusPage {
    let bar = ProgressBar::new(2).with_style(
        ProgressStyle::default_bar()
            .template("{msg}")
            .progress_chars("██"),
    );

    bar.set_message(format!(
        "> {} {}",
        "🔗".bright_yellow(),
        "google.com".to_string().bright_green().underline()
    ));

    surf::get("https://www.google.com".to_string())
        .recv_bytes()
        .await
        .unwrap_or_else(|error| {
            bar.abandon_with_message(format!(
                "💥 {error} for {}",
                "https://www.google.com"
                    .to_string()
                    .bright_red()
                    .underline()
            ));

            exit(1);
        });

    bar.inc(1);

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
            bar.abandon_with_message(format!("💥 Instatus API is down: {}", error));

            exit(1);
        });

    bar.inc(1);

    let mut status_page: Option<StatusPage> = None;

    for page in res {
        if config.name == page.name {
            status_page = Some(page);
        }
    }

    if status_page.is_some() {
        bar.finish_with_message("✅  All checks passed");

        status_page.unwrap()
    } else {
        bar.abandon_with_message(format!(
            "❌  Could not find relevant status page with name {}",
            config.name
        ));

        std::process::exit(1);
    }
}
