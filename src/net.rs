use std::process::exit;

use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

static DOMAINS: &'static [&'static str] = &[
    "https://www.google.com",
    "https://www.amazon.com",
    "https://stackoverflow.com",
];

pub async fn pre_flight_network_test() {
    println!("Running {} checks...", "pre-flight".bright_cyan());

    let bar = ProgressBar::new(3).with_style(
        ProgressStyle::default_bar()
            .template("{bar:20.cyan/black} {pos}/{len} > {msg}")
            .progress_chars("██"),
    );

    for domain in DOMAINS {
        bar.set_message(format!(
            "{} {}",
            "🔗".bright_yellow(),
            domain.to_string().bright_green().underline()
        ));

        surf::get(domain.to_string())
            .recv_bytes()
            .await
            .unwrap_or_else(|error| {
                bar.abandon_with_message(format!(
                    "💥 {error} for {}",
                    domain.to_string().bright_red().underline()
                ));

                exit(1);
            });

        bar.inc(1);
    }

    bar.finish_with_message(format!("✅ all checks passed"));
}
