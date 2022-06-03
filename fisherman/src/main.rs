use anyhow::{anyhow, Error};
use common::job_manage::{Job, JobResult};
use common::logger::init_logger;
use common::worker::{WorkerInfo, WorkerRegisterResult};
use std::collections::HashMap;

use fisherman::models::job::JobBuffer;
use fisherman::server_builder::WebServerBuilder;
use fisherman::server_config::AccessControl;
use fisherman::services::{JobExecution, JobResultReporter, WebServiceBuilder};
use fisherman::state::WorkerState;
use fisherman::{ENVIRONMENT, FISHERMAN_ENDPOINT, SCHEDULER_ENDPOINT};
use futures_util::future::{join, join3};
use log::{debug, info, warn};
use reqwest::StatusCode;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    // Load env file
    dotenv::dotenv().ok();
    // Init logger
    let _res = init_logger(&String::from("Fisherman-worker"));
    // Create job queue
    //Call to scheduler to register worker
    if let Ok(WorkerRegisterResult {
        report_callback,
        worker_id,
    }) = try_register().await
    {
        log::info!("Successfully register worker {:?}", &worker_id);
        let (sender, mut receiver): (Sender<JobResult>, Receiver<JobResult>) = channel(1024);
        let job_buffer = Arc::new(Mutex::new(JobBuffer::new()));
        let mut reporter = JobResultReporter::new(receiver, report_callback);

        let mut execution = JobExecution::new(sender, job_buffer.clone());
        let socket_addr = FISHERMAN_ENDPOINT.as_str();
        let service = WebServiceBuilder::new().build();
        let access_control = AccessControl::default();
        // Create job process thread
        //let task_process_job = create_job_process_thread(receiver);
        let server = WebServerBuilder::default()
            .with_entry_point(socket_addr)
            .with_access_control(access_control)
            .with_worker_state(WorkerState::new(job_buffer.clone()))
            .build(service);
        let task_execution = task::spawn(async move { execution.run().await });
        let task_reporter = task::spawn(async move { reporter.run().await });
        info!("Start fisherman service ");
        let task_serve = server.serve();
        join3(task_serve, task_execution, task_reporter).await;
    }
}
/*
fn create_job_process_thread(mut receiver: Receiver<Job>) -> JoinHandle<()> {
    // Run thread verify
    let task_job = tokio::spawn(async move {
        info!("spawn thread!");
        // Process each socket concurrently.
        loop {
            info!("Wait for new job... ");
            let job = receiver.recv().await;
            // Process the job
            info!("Process job: {:?}", job);

            if let Some(job) = job {
                // Init job result
                let job_result = job.process().await;
                if let Ok(job_result) = job_result {
                    info!("Send job result: {:?}", job_result);
                    let res = job_result.send().await;
                    info!("Response Send job result: {:?}", res);
                }
            }

            sleep(Duration::from_millis(100));
        }
        warn!("Job queue is dead!");
    });
    task_job
}
*/

async fn try_register() -> Result<WorkerRegisterResult, anyhow::Error> {
    loop {
        let res = register().await;
        info!("Register result: {:?}", res);
        match res {
            Ok(res) => return Ok(res),
            Err(err) => {
                info!("error: {:?}", err);
                if &*ENVIRONMENT == "local" {
                    break;
                }
            }
        }
        sleep(Duration::from_millis(1000));
    }
    Err(anyhow!("Cannot register worker"))
}

async fn register() -> Result<WorkerRegisterResult, anyhow::Error> {
    let client_builder = reqwest::ClientBuilder::new();
    let client = client_builder.danger_accept_invalid_certs(true).build()?;
    let worker_info = WorkerInfo::new();
    let scheduler_url = SCHEDULER_ENDPOINT.as_str();
    let request_builder = client
        .post(scheduler_url.to_string())
        .header("content-type", "application/json")
        .body(serde_json::to_string(&worker_info)?);
    debug!("request_builder: {:?}", request_builder);
    let response = request_builder.send().await?;
    match response.status() {
        StatusCode::OK => match response.json::<WorkerRegisterResult>().await {
            Ok(parsed) => Ok(parsed),
            Err(err) => Err(anyhow!(format!("{:?}", &err))),
        },
        _ => Err(anyhow!("Cannot register worker")),
    }
}
