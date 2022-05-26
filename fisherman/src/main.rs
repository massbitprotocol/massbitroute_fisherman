use common::logger::init_logger;
use common::models::WorkerInfo;
use common::models::WorkerRegisterResult;
use fisherman::server_builder::FishermanServerBuilder;
use fisherman::server_config::AccessControl;
use fisherman::service::FishermanServiceBuilder;
use fisherman::{ENVIRONMENT, FISHERMAN_ENDPOINT, SCHEDULER_ENDPOINT};
use log::{debug, info};
use std::thread::sleep;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Load env file
    dotenv::dotenv().ok();
    // Init logger
    let _res = init_logger(&String::from("Fisherman-worker"));

    let socket_addr = FISHERMAN_ENDPOINT.as_str();
    let service = FishermanServiceBuilder::default().build();
    let access_control = AccessControl::default();

    //Call to scheduler to register worker
    try_register();

    let server = FishermanServerBuilder::default()
        .with_entry_point(socket_addr)
        .with_access_control(access_control)
        .build(service);

    info!("Start fisherman service ");
    let task_serve = server.serve();
    task_serve.await;
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
        //        .header("x-api-key", node.token.as_str())
        //        .header("host", node.get_host_header(&self.domain))
        .body(serde_json::to_string(&worker_info)?);
    debug!("request_builder: {:?}", request_builder);
    let result = request_builder.send().await?.text().await?;

    let result: WorkerRegisterResult = serde_json::from_str(result.as_str())?;
    Ok(result)
}
