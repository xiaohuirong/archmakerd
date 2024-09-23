use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone)]
pub struct MkArchQemuParams {
    pub out_dir: String,
    pub work_dir: String,
    pub img_size: Option<String>,
    pub swap: Option<String>,
    pub profile_dir: String,
}

#[derive(Serialize, Clone)]
pub enum JobStatus {
    Waiting,
    Running,
    Finished,
    Error(String),
}

#[derive(Clone)]
pub struct MkArchQemu {
    pub params: Arc<Mutex<Option<MkArchQemuParams>>>,
    pub status: Arc<Mutex<JobStatus>>,
    pub last_output: Arc<Mutex<Option<String>>>,
}

impl MkArchQemu {
    pub fn new() -> Self {
        MkArchQemu {
            params: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(JobStatus::Waiting)),
            last_output: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_params(&self, params: MkArchQemuParams) {
        let mut stored_params = self.params.lock().unwrap();
        *stored_params = Some(params);
    }

    pub fn get_params(&self) -> Option<MkArchQemuParams> {
        let params = self.params.lock().unwrap();
        params.clone()
    }

    pub fn get_status(&self) -> JobStatus {
        let status = self.status.lock().unwrap();
        status.clone()
    }

    pub fn get_last_output(&self) -> Option<String> {
        let output = self.last_output.lock().unwrap();
        output.clone()
    }

    pub fn run_command(&self) {
        let stored_params = self.params.lock().unwrap();
        if let Some(params) = &*stored_params {
            let mut status = self.status.lock().unwrap();
            *status = JobStatus::Running;

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

            match cmd.output() {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let combined_output = format!("stdout: {}\nstderr: {}", stdout, stderr);

                    let mut last_output = self.last_output.lock().unwrap();
                    *last_output = Some(combined_output);

                    let mut status = self.status.lock().unwrap();
                    *status = JobStatus::Finished;
                }
                Err(e) => {
                    let mut status = self.status.lock().unwrap();
                    *status = JobStatus::Error(e.to_string());
                }
            }
        } else {
            println!("Parameters not set.");
        }
    }
}

