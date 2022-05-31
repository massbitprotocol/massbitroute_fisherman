use anyhow::Error;
use common::job_manage::{Job, JobResult};
use common::logger::init_logger;
use common::worker::{WorkerInfo, WorkerRegisterResult};
use fisherman::job::job_process::JobProcess;
use fisherman::server_builder::FishermanServerBuilder;
use fisherman::server_config::AccessControl;
use fisherman::service::FishermanServiceBuilder;
use fisherman::{ENVIRONMENT, FISHERMAN_ENDPOINT, SCHEDULER_ENDPOINT};
use futures_util::future::join;
use log::{debug, info, warn};
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    // Load env file
    dotenv::dotenv().ok();
    // Init logger
    let _res = init_logger(&String::from("Fisherman-worker"));

    // Create job queue
    let (sender, mut receiver): (Sender<Job>, Receiver<Job>) = channel(1024);
    info!("sender: {:?}", sender);
    info!("receiver: {:?}", receiver);

    let socket_addr = FISHERMAN_ENDPOINT.as_str();
    let service = FishermanServiceBuilder::new(sender).build();
    let access_control = AccessControl::default();

    //Call to scheduler to register worker
    try_register();

    // Create job process thread
    let task_process_job = create_job_process_thread(receiver);
    let server = FishermanServerBuilder::default()
        .with_entry_point(socket_addr)
        .with_access_control(access_control)
        .build(service);

    info!("Start fisherman service ");
    let task_serve = server.serve();
    join(task_process_job, task_serve).await;
}

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

async fn try_register() {
    loop {
        let res = register().await;
        info!("Register result: {:?}", res);
        match res {
            Ok(res) => {
                break;
            }
            Err(err) => {
                info!("error: {:?}", err);
                if &*ENVIRONMENT == "local" {
                    break;
                }
            }
        }
        sleep(Duration::from_millis(1000));
    }
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
    let result = request_builder.send().await?.text().await?;

    let result: WorkerRegisterResult = serde_json::from_str(result.as_str())?;
    Ok(result)
}
