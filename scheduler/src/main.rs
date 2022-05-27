use clap::{Arg, Command};
use common::logger::init_logger;
use futures_util::future::{join, join4};
use log::{debug, info, warn};
use scheduler::models::providers::ProviderStorage;
use scheduler::models::workers::WorkerPool;
use scheduler::provider::scanner::ProviderScanner;
use scheduler::server_builder::ServerBuilder;
use scheduler::server_config::AccessControl;
use scheduler::service::delivery::JobDelivery;
use scheduler::service::generator::JobGenerator;
use scheduler::service::{ProcessorServiceBuilder, SchedulerServiceBuilder};
use scheduler::SCHEDULER_ENDPOINT;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task;

#[tokio::main]
async fn main() {
    // Load env file
    dotenv::dotenv().ok();

    let _res = init_logger(&String::from("Fisherman Scheduler"));
    let matches = create_scheduler_app().get_matches();
    let socket_addr = SCHEDULER_ENDPOINT.as_str();
    let scheduler_service = SchedulerServiceBuilder::default().build();
    let processor_service = ProcessorServiceBuilder::default().build();
    let access_control = AccessControl::default();
    let server = ServerBuilder::default()
        .with_entry_point(socket_addr)
        .with_access_control(access_control)
        .build(scheduler_service, processor_service);
    //let (tx, mut rx) = mpsc::channel(1024);
    let provider_storage = Arc::new(ProviderStorage::default());
    let worker_pool = Arc::new(WorkerPool::new());
    let mut provider_scanner = ProviderScanner::new(provider_storage.clone());
    let mut job_generator = JobGenerator::new(provider_storage.clone(), worker_pool.clone());
    let mut job_delivery = JobDelivery::new();
    info!("Init http service ");
    let task_provider_scanner = task::spawn(async move { provider_scanner.init() });
    let task_job_generator = task::spawn(async move { job_generator.init() });
    let task_job_delivery = task::spawn(async move { job_delivery.init() });
    let task_serve = server.serve();
    join4(
        task_provider_scanner,
        task_job_generator,
        task_job_delivery,
        task_serve,
    )
    .await;
}

fn create_scheduler_app() -> Command<'static> {
    Command::new("check-kind")
        .version("0.1")
        .about("fisherman-scheduler")
        .arg(
            Arg::new("list-node-id-file")
                .short('n')
                .long("list-node-id-file")
                .value_name("list-node-id-file")
                .help("Input list-node-id file")
                .takes_value(true),
        )
        .arg(
            Arg::new("list-gateway-id-file")
                .short('g')
                .long("list-gateway-id-file")
                .value_name("list-gateway-id-file")
                .help("Input list-gateway-id file")
                .takes_value(true),
        )
        .arg(
            Arg::new("list-dapi-id-file")
                .short('d')
                .long("list-dapi-id-file")
                .value_name("list-dapi-id-file")
                .help("Input list-dapi-id file")
                .takes_value(true),
        )
        .arg(
            Arg::new("list-user-file")
                .short('u')
                .long("list-user-file")
                .value_name("list-user-file")
                .help("Input list-user file")
                .takes_value(true),
        )
        .arg(
            Arg::new("check-flow")
                .short('c')
                .long("check-flow")
                .value_name("check-flow")
                .help("Input check-flow file")
                .takes_value(true),
        )
        .arg(
            Arg::new("base-endpoint")
                .short('b')
                .long("base-endpoint")
                .value_name("base-endpoint")
                .help("Input base-endpoint file")
                .takes_value(true),
        )
        .arg(
            Arg::new("massbit-chain-endpoint")
                .short('m')
                .long("massbit-chain-endpoint")
                .value_name("massbit-chain-endpoint")
                .help("Input massbit-chain-endpoint")
                .takes_value(true),
        )
        .arg(
            Arg::new("signer-phrase")
                .short('s')
                .long("signer-phrase")
                .value_name("signer-phrase")
                .help("Input signer-phrase")
                .takes_value(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("output")
                .help("Output file")
                .takes_value(true),
        )
        .arg(
            Arg::new("domain")
                .long("domain")
                .value_name("domain")
                .help("domain name")
                .takes_value(true),
        )
}
