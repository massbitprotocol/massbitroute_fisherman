#[macro_use]
extern crate diesel_migrations;

use clap::{Arg, Command};
use common::component::ComponentInfo;
use common::logger::init_logger;
use diesel::r2d2::ConnectionManager;
use diesel::{r2d2, PgConnection};
use diesel_migrations::embed_migrations;
use futures_util::future::{join, join4};
use log::{debug, info, warn};
use scheduler::models::providers::ProviderStorage;
use scheduler::models::workers::WorkerPool;
use scheduler::provider::scanner::ProviderScanner;
use scheduler::server_builder::ServerBuilder;
use scheduler::server_config::{AccessControl, Config};
use scheduler::service::delivery::JobDelivery;
use scheduler::service::generator::JobGenerator;
use scheduler::service::{ProcessorServiceBuilder, SchedulerServiceBuilder};
use scheduler::state::SchedulerState;
use scheduler::{CONNECTION_POOL_SIZE, DATABASE_URL, SCHEDULER_CONFIG, SCHEDULER_ENDPOINT};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::task;

embed_migrations!("./migrations");

#[tokio::main]
async fn main() {
    // Load env file
    dotenv::dotenv().ok();
    let _res = init_logger(&String::from("Fisherman Scheduler"));
    let matches = create_scheduler_app().get_matches();
    let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL.as_str());
    let connection_pool = r2d2::Pool::builder()
        .max_size(*CONNECTION_POOL_SIZE)
        .build(manager)
        .expect("Can not create connection pool");
    if let Ok(conn) = &connection_pool.get() {
        match embedded_migrations::run(conn) {
            Ok(res) => log::info!("Finished embedded_migration {:?}", &res),
            Err(err) => log::info!("{:?}", &err),
        };
    }
    let config = Config::load(SCHEDULER_CONFIG.as_str());
    let socket_addr = SCHEDULER_ENDPOINT.as_str();
    let provider_storage = Arc::new(Mutex::new(ProviderStorage::default()));
    let worker_pool = Arc::new(WorkerPool::default());
    let scheduler_state = SchedulerState::new(provider_storage.clone());
    let scheduler_service = SchedulerServiceBuilder::default().build();
    let processor_service = ProcessorServiceBuilder::default().build();
    let access_control = AccessControl::default();
    //let (tx, mut rx) = mpsc::channel(1024);

    let mut provider_scanner = ProviderScanner::new(
        config.url_list_nodes.clone(),
        config.url_list_gateways.clone(),
        provider_storage.clone(),
        worker_pool.clone(),
    );
    let mut job_generator = JobGenerator::new(provider_storage.clone(), worker_pool.clone());
    let mut job_delivery = JobDelivery::new(worker_pool.clone());
    info!("Init http service ");
    let task_provider_scanner = task::spawn(async move { provider_scanner.init() });
    let task_job_generator = task::spawn(async move { job_generator.init() });
    let task_job_delivery = task::spawn(async move { job_delivery.init() });
    let server = ServerBuilder::default()
        .with_entry_point(socket_addr)
        .with_access_control(access_control)
        .with_scheduler_state(scheduler_state)
        .build(scheduler_service, processor_service);
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
