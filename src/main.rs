use owo_colors::OwoColorize;

pub mod net;

fn main() {
    smol::block_on(async {
        net::pre_flight_network_test().await;
        println!("🌊 Spinning up network client");
        println!(
            "📔 Reading configuration variables from {}",
            "velocity.toml".bright_magenta().underline()
        );
    });
}
