use log::info;
use logger::init_logger;
use mbr_stats::component_stats::ComponentStats;
use mbr_stats::{LOG_CONFIG, PROMETHEUS_URL, URL_PORTAL_PROJECTS};
use mbr_stats::{SIGNER_PHRASE, URL_CHAIN};
#[tokio::main]
async fn main() {
    // Load env file
    info!("Start mbr stats");
    if dotenv::dotenv().is_err() {
        println!("Warning: Cannot load .env file");
    }

    init_logger(&String::from("ComponentStats"), Some(&*LOG_CONFIG));
    // Show env list
    info!("Envs list");
    for (key, value) in std::env::vars() {
        info!("{key}: {value}");
    }

    let mut component_stats = ComponentStats::builder(&URL_CHAIN, &SIGNER_PHRASE)
        .await
        .with_prometheus_url(&PROMETHEUS_URL)
        .await
        .with_list_project_url(URL_PORTAL_PROJECTS.to_string())
        .build();
    log::debug!("check_component: {:?}", component_stats);
    let _ = component_stats.run().await;
}
