use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use crate::model::PolynomialModel;
use crate::monitor::CpuUsageTracker;

#[derive(Deserialize)]
pub struct QueryParams {
    vm: String,
}

pub async fn get_cpu_usage_estimate(
    query: web::Query<QueryParams>,
    tracker: web::Data<Arc<Mutex<CpuUsageTracker>>>,
    polynomial: web::Data<PolynomialModel>,
) -> impl Responder {
    let vm_name = &query.vm;
    if vm_name.is_empty() {return HttpResponse::BadRequest().body("VM name cannot be empty");}
    //println!("Received request for VM: {}", vm_name);
    
    let cpu_usage = tracker.lock().unwrap().get_last_cpu_usage_for_vm(&vm_name);
    match cpu_usage {
        Some(usage) => {
            let estimated_value = polynomial.estimate(usage); // Apply the polynomial estimate using the CPU usage as input
            HttpResponse::Ok().json(estimated_value) // Return as JSON
        }
        None => HttpResponse::NotFound().body(format!("No CPU usage data available for '{}'", vm_name)),
    }
}

pub async fn run_server(tracker: Arc<Mutex<CpuUsageTracker>>, polynomial: PolynomialModel, port: String) -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "debug");
    // env_logger::init();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tracker.clone()))
            .app_data(web::Data::new(polynomial.clone()))
            .route("/power", web::get().to(get_cpu_usage_estimate))
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}