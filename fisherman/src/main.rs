use log::{debug, info};
use fisherman::{FISHERMAN_ENDPOINT, SCHEDULER_ENDPOINT};
use fisherman::server_builder::FishermanServerBuilder;
use fisherman::server_config::AccessControl;
use fisherman::service::FishermanServiceBuilder;
use core::models::WorkerInfo;
use core::models::WorkerRegisterResult;
#[tokio::main]
async fn main() {
    // Load env file
    dotenv::dotenv().ok();
    let socket_addr = FISHERMAN_ENDPOINT.as_str();
    let service = FishermanServiceBuilder::default().build();
    let access_control = AccessControl::default();
    let server = FishermanServerBuilder::default()
        .with_entry_point(socket_addr)
        .with_access_control(access_control)
        .build(service);
    info!("Start fisherman service ");
    //Call to scheduler to register worker
    let task_serve = server.serve();
    task_serve.await;

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
        .body(worker_info);
    let result = WorkerRegisterResult::new();
    Ok(result)
}