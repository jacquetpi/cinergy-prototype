use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::config::AppConfig;
use crate::model::PolynomialModel;
use crate::monitor::CpuUsageTracker;

#[derive(Deserialize)]
pub struct QueryParams {
    vm: String,
}

#[derive(Serialize)]
struct PowerResponse {
    vm: String,
    cpu_usage: f64,
    vm_cores: u32,
    server_cores: u32,
    cinergy_ratio: f64,
    static_power_watts: f64,
    dynamic_power_watts: f64,
    total_power_watts: f64,
}

#[derive(Serialize)]
struct CarbonResponse {
    vm: String,
    power_watts: f64,
    pue: f64,
    emission_factor_gco2_per_kwh: f64,
    carbon_intensity_gco2_per_hour: f64,
}

pub async fn get_power_estimate(
    query: web::Query<QueryParams>,
    tracker: web::Data<Arc<Mutex<CpuUsageTracker>>>,
    polynomial: web::Data<PolynomialModel>,
    config: web::Data<AppConfig>,
) -> impl Responder {
    let vm_name = &query.vm;
    if vm_name.is_empty() {
        return HttpResponse::BadRequest().body("VM name cannot be empty");
    }

    let tracker = tracker.lock().unwrap();
    let cpu_usage = match tracker.get_last_cpu_usage_for_vm(vm_name) {
        Some(u) => u,
        None => return HttpResponse::NotFound().body(format!("No CPU usage data available for '{}'", vm_name)),
    };
    let vm_cores = tracker.get_vcpu_count_for_vm(vm_name).unwrap_or(1);
    drop(tracker);

    let estimate = polynomial.estimate_vm_power(
        cpu_usage,
        vm_cores,
        config.server_cores,
        config.cinergy_ratio,
    );

    HttpResponse::Ok().json(PowerResponse {
        vm: vm_name.clone(),
        cpu_usage,
        vm_cores,
        server_cores: config.server_cores,
        cinergy_ratio: config.cinergy_ratio,
        static_power_watts: estimate.static_power_watts,
        dynamic_power_watts: estimate.dynamic_power_watts,
        total_power_watts: estimate.total_power_watts,
    })
}

pub async fn get_carbon_emissions(
    query: web::Query<QueryParams>,
    tracker: web::Data<Arc<Mutex<CpuUsageTracker>>>,
    polynomial: web::Data<PolynomialModel>,
    config: web::Data<AppConfig>,
) -> impl Responder {
    let vm_name = &query.vm;
    if vm_name.is_empty() {
        return HttpResponse::BadRequest().body("VM name cannot be empty");
    }

    let tracker = tracker.lock().unwrap();
    let cpu_usage = match tracker.get_last_cpu_usage_for_vm(vm_name) {
        Some(u) => u,
        None => return HttpResponse::NotFound().body(format!("No CPU usage data available for '{}'", vm_name)),
    };
    let vm_cores = tracker.get_vcpu_count_for_vm(vm_name).unwrap_or(1);
    drop(tracker);

    let estimate = polynomial.estimate_vm_power(
        cpu_usage,
        vm_cores,
        config.server_cores,
        config.cinergy_ratio,
    );

    let carbon = config.carbon_intensity_gco2_per_hour(estimate.total_power_watts);

    HttpResponse::Ok().json(CarbonResponse {
        vm: vm_name.clone(),
        power_watts: estimate.total_power_watts,
        pue: config.dc_pue,
        emission_factor_gco2_per_kwh: config.emission_factor,
        carbon_intensity_gco2_per_hour: carbon,
    })
}

pub async fn run_server(
    tracker: Arc<Mutex<CpuUsageTracker>>,
    polynomial: PolynomialModel,
    config: AppConfig,
    port: String,
) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tracker.clone()))
            .app_data(web::Data::new(polynomial.clone()))
            .app_data(web::Data::new(config.clone()))
            .route("/power", web::get().to(get_power_estimate))
            .route("/carbon", web::get().to(get_carbon_emissions))
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}
