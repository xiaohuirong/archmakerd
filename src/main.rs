use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
mod config;

#[derive(Serialize, Deserialize, Clone)]
struct MkArchQemuParams {
    out_dir: String,
    work_dir: String,
    img_size: Option<String>,
    swap: Option<String>,
    profile_dir: String,
}

#[derive(Serialize, Clone)]
enum JobStatus {
    Waiting,
    Running,
    Finished,
    Error(String),
}

#[derive(Clone)]
struct MkArchQemu {
    params: Arc<Mutex<Option<MkArchQemuParams>>>,
    status: Arc<Mutex<JobStatus>>,
    last_output: Arc<Mutex<Option<String>>>,
}

impl MkArchQemu {
    fn new() -> Self {
        MkArchQemu {
            params: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(JobStatus::Waiting)),
            last_output: Arc::new(Mutex::new(None)),
        }
    }

    // update parameters
    fn set_params(&self, params: MkArchQemuParams) {
        let mut stored_params = self.params.lock().unwrap();
        *stored_params = Some(params);
    }

    // get current parameters
    fn get_params(&self) -> Option<MkArchQemuParams> {
        let params = self.params.lock().unwrap();
        params.clone()
    }

    // get current status
    fn get_status(&self) -> JobStatus {
        let status = self.status.lock().unwrap();
        status.clone()
    }

    // get last output
    fn get_last_output(&self) -> Option<String> {
        let output = self.last_output.lock().unwrap();
        output.clone()
    }

    // run build command
    fn run_command(&self) {
        let stored_params = self.params.lock().unwrap();
        if let Some(params) = &*stored_params {
            // set status to Running
            {
                let mut status = self.status.lock().unwrap();
                *status = JobStatus::Running;
            }

            // generate build command
            let mut cmd = Command::new("/usr/bin/mkarchqemu");
            cmd.arg("-o").arg(&params.out_dir);
            cmd.arg("-w").arg(&params.work_dir);

            if let Some(img_size) = &params.img_size {
                cmd.arg("-s").arg(img_size);
            }

            if let Some(swap) = &params.swap {
                cmd.arg(format!("--swap={}", swap));
            }

            cmd.arg(&params.profile_dir);

            // print command
            let command_string = format!(
                "/usr/bin/mkarchqemu -o {} -w {} -s {} {} {}",
                params.out_dir,
                params.work_dir,
                params.img_size.as_deref().unwrap_or_default(),
                params
                    .swap
                    .as_deref()
                    .map_or("".to_string(), |s| format!("--swap={}", s)),
                params.profile_dir
            );
            println!("Executing command: {}", command_string);

            // run build commnad
            match cmd.output() {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let combined_output = format!("stdout: {}\nstderr: {}", stdout, stderr);

                    // update last log
                    {
                        let mut last_output = self.last_output.lock().unwrap();
                        *last_output = Some(combined_output);
                    }

                    // set status to Finished
                    {
                        let mut status = self.status.lock().unwrap();
                        *status = JobStatus::Finished;
                    }
                }
                Err(e) => {
                    // set status to Error
                    {
                        let mut status = self.status.lock().unwrap();
                        *status = JobStatus::Error(e.to_string());
                    }
                }
            }
        } else {
            println!("Parameters not set.");
        }
    }
}

async fn set_params(
    api_data: web::Data<MkArchQemu>,
    params: web::Json<MkArchQemuParams>,
) -> impl Responder {
    api_data.set_params(params.into_inner());
    HttpResponse::Ok().json(serde_json::json!({ "status": "parameters set successfully" }))
}

async fn get_params(api_data: web::Data<MkArchQemu>) -> impl Responder {
    let params = api_data.get_params();
    HttpResponse::Ok().json(serde_json::json!({ "params": params }))
}

async fn get_status(api_data: web::Data<MkArchQemu>) -> impl Responder {
    let status = api_data.get_status();
    HttpResponse::Ok().json(serde_json::json!({ "status": status }))
}

async fn get_last_output(api_data: web::Data<MkArchQemu>) -> impl Responder {
    let output = api_data.get_last_output();
    HttpResponse::Ok().json(serde_json::json!({ "last_output": output }))
}

async fn run_command(api_data: web::Data<MkArchQemu>) -> impl Responder {
    let api_clone = api_data.clone();
    thread::spawn(move || {
        api_clone.run_command();
    });

    HttpResponse::Ok().json(serde_json::json!({ "status": "command is being executed" }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
