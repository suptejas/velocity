use std::process::exit;

use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

pub async fn pre_flight_network_test() {
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

    surf::get("https://api.instatus.com".to_string())
        .recv_bytes()
        .await
        .unwrap_or_else(|error| {
            bar.abandon_with_message(format!("💥 Instatus API is down: {}", error));

            exit(1);
        });

    bar.inc(1);

    bar.finish_with_message(format!("✅  All checks passed"));
}
