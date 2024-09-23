use actix_web::{web, App, HttpResponse, HttpServer, Responder};
mod config;
mod mkarchqemu;
use mkarchqemu::{MkArchQemu, MkArchQemuParams};

async fn set_params(
    api_data: web::Data<MkArchQemu>,
    params: web::Json<MkArchQemuParams>,
) -> impl Responder {
    log::info!("Received request to set parameters: {:?}", params);
    api_data.set_params(params.into_inner());
    HttpResponse::Ok().json(serde_json::json!({ "status": "parameters set successfully" }))
}

async fn get_params(api_data: web::Data<MkArchQemu>) -> impl Responder {
    log::info!("Received request to get parameters.");
    let params = api_data.get_params();
    log::info!("Returned parameters: {:?}", params);
    HttpResponse::Ok().json(serde_json::json!({ "params": params }))
}

async fn get_status(api_data: web::Data<MkArchQemu>) -> impl Responder {
    log::info!("Received request to get job status.");
    let status = api_data.get_status();
    log::info!("Returned job status: {:?}", status);
    HttpResponse::Ok().json(serde_json::json!({ "status": status }))
}

async fn get_last_output(api_data: web::Data<MkArchQemu>) -> impl Responder {
    log::info!("Received request to get last output.");
    let output = api_data.get_last_output();
    log::info!("Returned last output: {:?}", output);
    HttpResponse::Ok().json(serde_json::json!({ "last_output": output }))
}

async fn run_command(api_data: web::Data<MkArchQemu>) -> impl Responder {
    log::info!("Received request to run command.");
    let api_clone = api_data.clone();
    api_clone.run_command();
    log::info!("Command is being executed.");

    HttpResponse::Ok().json(serde_json::json!({ "status": "command is being executed" }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    log::info!("Starting the server...");

    let mkarchqemu = MkArchQemu::new();
    let data = web::Data::new(mkarchqemu);

    let config = config::parse_args();
    let addr = format!("{}:{}", config.bind_address, config.port);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/set_params", web::post().to(set_params))
            .route("/get_params", web::get().to(get_params))
            .route("/get_status", web::get().to(get_status))
            .route("/get_last_output", web::get().to(get_last_output))
            .route("/run_command", web::post().to(run_command))
    })
    .bind(addr)?
    .run()
    .await
}
