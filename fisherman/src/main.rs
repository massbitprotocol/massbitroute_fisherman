use anyhow::Error;
use common::jobs::JobResult;
use common::logger::init_logger;
use common::workers::{WorkerInfo, WorkerRegisterResult};

use common::COMMON_CONFIG;
use fisherman::models::job::JobBuffer;
use fisherman::server_builder::WebServerBuilder;
use fisherman::server_config::AccessControl;
use fisherman::services::service_status::WorkerStatusCheck;
use fisherman::services::{JobExecution, JobResultReporter, WebServiceBuilder};
use fisherman::state::WorkerState;
use fisherman::{
    LOG_CONFIG, SCHEDULER_AUTHORIZATION, SCHEDULER_ENDPOINT, WORKER_ENDPOINT, WORKER_ID, WORKER_IP,
    WORKER_SERVICE_ENDPOINT, ZONE,
};
use futures_util::future::join3;
use log::{debug, error, info, warn};
use reqwest::StatusCode;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    // Load env file
    if dotenv::dotenv().is_err() {
        println!("Warning: Cannot load .env file");
        panic!("Cannot load .env file");
    }
    // Init logger
    let _res = init_logger(&String::from("Fisherman-worker"), LOG_CONFIG.to_str());
    // Show env list
    info!("Envs list");
    for (key, value) in std::env::vars() {
        info!("{key}: {value}");
    }

    // Create job queue
    //Call to scheduler to register worker
    if let Ok(WorkerRegisterResult {
        report_callback,
        worker_id,
    }) = try_register().await
    {
        info!(
            "Successfully register worker {}, report_callback: {}",
            &worker_id, report_callback
        );
        let (sender, receiver): (Sender<JobResult>, Receiver<JobResult>) = channel(1024);
        let job_buffer = Arc::new(Mutex::new(JobBuffer::new()));
        let mut reporter = JobResultReporter::new(receiver, report_callback);

        let mut execution = JobExecution::new(sender.clone(), job_buffer.clone());
        let service = WebServiceBuilder::new().build();
        let access_control = AccessControl::default();
        // Create status worker check
        let worker_status_check = WorkerStatusCheck::new(sender, job_buffer.clone());
        let worker_status = worker_status_check.get_status();

        // Create job process thread
        let server = WebServerBuilder::default()
            .with_entry_point(WORKER_SERVICE_ENDPOINT.as_str())
            .with_access_control(access_control)
            .with_worker_state(WorkerState::new(job_buffer.clone()))
            .build(service, worker_status);

        let _task_execution = tokio::spawn(async move { execution.run().await });
        let task_reporter = task::spawn(async move { reporter.run().await });
        let task_worker_status_check = task::spawn(async move { worker_status_check.run().await });
        info!("Start fisherman service ");
        let task_serve = server.serve();
        let _res = join3(task_serve, task_reporter, task_worker_status_check).await;
        warn!("Never end tasks.");
    }
}

async fn try_register() -> Result<WorkerRegisterResult, Error> {
    let client_builder = reqwest::ClientBuilder::new();
    let client = client_builder.danger_accept_invalid_certs(true).build()?;
    let worker_info = WorkerInfo::new(
        WORKER_ID.as_str(),
        WORKER_ENDPOINT.as_str(),
        WORKER_IP.as_str(),
        ZONE.as_str(),
    );
    let body = serde_json::to_string(&worker_info)?;
    loop {
        let scheduler_url = format!("{}/worker/register", SCHEDULER_ENDPOINT.as_str());
        let clone_client = client.clone();
        let clone_body = body.clone();
        debug!("Register worker to scheduler {}", scheduler_url);
        let request_builder = clone_client
            .post(scheduler_url)
            .header("content-type", "application/json")
            .header("authorization", &*SCHEDULER_AUTHORIZATION)
            .body(clone_body)
            .timeout(Duration::from_millis(
                COMMON_CONFIG.default_http_request_timeout_ms,
            ));
        debug!("Register worker request builder: {:?}", request_builder);
        let response = request_builder.send().await;
        if response.is_err() {
            sleep(Duration::from_millis(2000)).await;
            error!("Register worker Error: {:?}", response);
            continue;
        }
        let response = response.unwrap();
        match response.status() {
            StatusCode::OK => match response.json::<WorkerRegisterResult>().await {
                Ok(parsed) => return Ok(parsed),
                Err(err) => {
                    info!("Error: {:?}", err);
                }
            },
            _ => {
                let text = response.text().await?;
                debug!("Cannot register worker with message {}", &text);
            }
        }
        sleep(Duration::from_millis(2000)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use httpmock::prelude::POST;
    use httpmock::MockServer;
    use std::env;
    use test_util::helper::load_env;

    const MOCK_WORKER_ID: &str = "7c7da61c-aec7-45b1-9e32-7436d4721ce0";
    const MOCK_REPORT_CALLBACK: &str = "http://127.0.0.1:3031/report";

    fn run_mock_portal_server() -> MockServer {
        // Start a lightweight mock server.
        let server = MockServer::start();
        let body = format!(
            "{{ \"worker_id\": \"{}\", \"report_callback\": \"{}\" }}",
            MOCK_WORKER_ID, MOCK_REPORT_CALLBACK
        );
        // Create a mock on the server.
        let _hello_mock = server.mock(|when, then| {
            when.method(POST).path("/worker/register");
            then.status(200)
                .header("content-type", "text/html; charset=UTF-8")
                .body(body);
        });
        server
    }

    #[tokio::test]
    async fn test_try_register_success() -> Result<(), Error> {
        load_env();
        // let _res = init_logger(&String::from("Fisherman-worker"));
        let portal = run_mock_portal_server();
        let url = format!("http://{}", portal.address());
        env::set_var("SCHEDULER_ENDPOINT", url);
        env::set_var("WORKER_ID", MOCK_WORKER_ID);
        env::set_var("WORKER_ENDPOINT", "WORKER_ENDPOINT");
        env::set_var("WORKER_IP", "WORKER_IP");
        env::set_var("ZONE", "AS");
        env::set_var("SCHEDULER_AUTHORIZATION", "DEFAULT_SCHEDULER_AUTHORIZATION");
        let res = try_register().await;
        println!("res: {:?}", res);

        if let Ok(res) = res {
            assert_eq!(res.worker_id, MOCK_WORKER_ID);
            assert_eq!(res.report_callback, MOCK_REPORT_CALLBACK);
        }
        Ok(())
    }

    // #[tokio::test]
    // #[should_panic(expected = "Cannot register case")]
    // async fn test_try_register_fail() {
    //     println!("Cannot register case");
    //     //let _res = init_logger(&String::from("Fisherman-worker"));
    //     env::set_var("SCHEDULER_ENDPOINT", "");
    //     env::set_var("WORKER_ID", MOCK_WORKER_ID);
    //     env::set_var("WORKER_ENDPOINT", "WORKER_ENDPOINT");
    //     env::set_var("WORKER_IP", "WORKER_IP");
    //     env::set_var("ZONE", "AS");
    //     let ft_try_register = try_register();
    //     println!("Cannot register case");
    //     pin_mut!(ft_try_register);
    //     if let Err(_) =
    //         tokio::time::timeout(std::time::Duration::from_secs(1), &mut ft_try_register).await
    //     {
    //         panic!("Cannot register case");
    //     }
    // }
}
