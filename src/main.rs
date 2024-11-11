mod monitor;
mod model;
mod endpoint;

use std::sync::{Arc, Mutex};
use std::env;

use monitor::CpuUsageTracker;
use model::PolynomialModel;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    dotenv::dotenv().ok();
    let coefficients = get_float_list("REG_COEFF");
    println!("Loaded formula: {:?}", coefficients);
    let model = PolynomialModel::new(coefficients);

    let tracker = Arc::new(Mutex::new(CpuUsageTracker::new()));
    CpuUsageTracker::track_cpu_usage(tracker.clone()); // Start the CPU usage tracking in a separate thread

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    println!("Starting server on port {}", port);
    endpoint::server::run_server(tracker, model, port).await
    
}

fn get_float_list(key: &str) -> Vec<f64> {
    let value = env::var(key).expect(&format!("{} must be set", key));
    value
        .split(',')
        .map(|s| s.trim().parse::<f64>().expect("Failed to parse float"))
        .collect()
}