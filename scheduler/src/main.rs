#[macro_use]
extern crate diesel_migrations;

use clap::{Arg, Command};
use common::logger::init_logger;
//use diesel::r2d2::ConnectionManager;
//use diesel::{r2d2, PgConnection};
//use diesel_migrations::embed_migrations;
use futures_util::future::join4;
use log::info;
use scheduler::models::jobs::AssignmentBuffer;
use scheduler::models::providers::ProviderStorage;
use scheduler::models::workers::WorkerInfoStorage;
use scheduler::provider::scanner::ProviderScanner;
use scheduler::server_builder::ServerBuilder;
use scheduler::server_config::AccessControl;
use scheduler::service::delivery::JobDelivery;
use scheduler::service::generator::JobGenerator;
use scheduler::service::{ProcessorServiceBuilder, SchedulerServiceBuilder};
use scheduler::state::{ProcessorState, SchedulerState};
use scheduler::{CONFIG, DATABASE_URL, SCHEDULER_ENDPOINT, URL_GATEWAYS_LIST, URL_NODES_LIST};

use scheduler::models::job_result_cache::JobResultCache;
use scheduler::persistence::services::job_result_service::JobResultService;
use scheduler::persistence::services::plan_service::PlanService;
use scheduler::persistence::services::provider_service::ProviderService;
use scheduler::persistence::services::WorkerService;
use scheduler::persistence::services::{get_sea_db_connection, JobService};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;

#[tokio::main]
async fn main() {
    // Load env file
    dotenv::dotenv().ok();
    let _res = init_logger(&String::from("Fisherman Scheduler"));
    let _matches = create_scheduler_app().get_matches();
    // let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL.as_str());
    // let connection_pool = r2d2::Pool::builder()
    //     .max_size(*CONNECTION_POOL_SIZE)
    //     .build(manager)
    //     .expect("Can not create connection pool");
    // if let Ok(conn) = &connection_pool.get() {
    //     match embedded_migrations::run(conn) {
    //         Ok(res) => log::info!("Finished embedded_migration {:?}", &res),
    //         Err(err) => log::info!("{:?}", &err),
    //     };
    // }
    let db_conn = match get_sea_db_connection(DATABASE_URL.as_str()).await {
        Ok(con) => con,
        Err(_) => {
            panic!("Please check database connection and retry.")
        }
    };
    let arc_conn = Arc::new(db_conn);
    let plan_service = Arc::new(PlanService::new(arc_conn.clone()));
    //Get worker infos
    let worker_service = Arc::new(WorkerService::new(arc_conn.clone()));
    let provider_service = Arc::new(ProviderService::new(arc_conn.clone()));
    let job_service = Arc::new(JobService::new(arc_conn.clone()));
    let all_workers = worker_service.clone().get_active().await;

    let socket_addr = SCHEDULER_ENDPOINT.as_str();
    let provider_storage = Arc::new(Mutex::new(ProviderStorage::default()));
    log::debug!("Init with {:?} workers", all_workers.len());
    let worker_infos = Arc::new(Mutex::new(WorkerInfoStorage::new(all_workers)));
    let assigment_buffer = Arc::new(Mutex::new(AssignmentBuffer::default()));

    let scheduler_service = SchedulerServiceBuilder::default().build();
    let result_service = Arc::new(JobResultService::new(arc_conn.clone()));
    let result_cache = Arc::new(Mutex::new(JobResultCache::default()));
    let processor_service = ProcessorServiceBuilder::default()
        .with_plan_service(plan_service.clone())
        .with_result_service(result_service)
        .with_job_service(job_service.clone())
        .with_result_cache(result_cache.clone())
        .build();
    let access_control = AccessControl::default();
    //let (tx, mut rx) = mpsc::channel(1024);
    //Scanner for update provider list from portal
    let provider_scanner = ProviderScanner::new(
        URL_NODES_LIST.to_string(),
        URL_GATEWAYS_LIST.to_string(),
        provider_storage.clone(),
        worker_infos.clone(),
        provider_service.clone(),
    );
    let job_generator = JobGenerator::new(
        arc_conn.clone(),
        plan_service.clone(),
        provider_storage.clone(),
        worker_infos.clone(),
        job_service.clone(),
        assigment_buffer.clone(),
        result_cache.clone(),
    );
    let scheduler_state = SchedulerState::new(
        arc_conn.clone(),
        plan_service.clone(),
        worker_service,
        worker_infos.clone(),
        provider_storage.clone(),
    );
    //let mut judgment = Judgment::new(plan_service.clone(), arc_conn.clone());
    let mut job_delivery = JobDelivery::new(worker_infos.clone(), assigment_buffer.clone());
    //let task_judgment = task::spawn(async move { judgment.run().await });
    let task_provider_scanner = task::spawn(async move { provider_scanner.run().await });

    let task_job_generator = task::spawn(async move { job_generator.run().await });
    let task_job_delivery = task::spawn(async move { job_delivery.run().await });

    let processor_state = ProcessorState::new(
        arc_conn.clone(),
        result_cache.clone(),
        result_service.clone(),
        plan_service.clone(),
        job_service.clone(),
    );
    info!("Init http service ");
    let server = ServerBuilder::default()
        .with_entry_point(socket_addr)
        .with_access_control(access_control)
        .with_scheduler_state(scheduler_state)
        .with_processor_state(processor_state)
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
