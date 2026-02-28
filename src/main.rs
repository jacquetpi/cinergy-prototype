mod config;
mod endpoint;
mod model;
mod monitor;

use std::env;
use std::sync::{Arc, Mutex};

use config::AppConfig;
use model::PolynomialModel;
use monitor::CpuUsageTracker;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let coefficients = get_float_list("REG_COEFF");
    println!("Loaded formula: {:?}", coefficients);
    let model = PolynomialModel::new(coefficients);

    let config = AppConfig::from_env();
    println!(
        "Config: cinergy_ratio={}, server_cores={}, dc_pue={}, emission_factor={}",
        config.cinergy_ratio, config.server_cores, config.dc_pue, config.emission_factor
    );

    let tracker = Arc::new(Mutex::new(CpuUsageTracker::new()));
    CpuUsageTracker::track_cpu_usage(tracker.clone());

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    println!("Starting server on port {}", port);
    endpoint::server::run_server(tracker, model, config, port).await
}

fn get_float_list(key: &str) -> Vec<f64> {
    let value = env::var(key).unwrap_or_else(|_| panic!("{} must be set", key));
    value
        .split(',')
        .map(|s| s.trim().parse::<f64>().expect("Failed to parse float"))
        .collect()
}
