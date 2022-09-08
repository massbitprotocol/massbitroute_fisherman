use common::logger::init_logger;
//use diesel::r2d2::ConnectionManager;
//use diesel::{r2d2, PgConnection};
//use diesel_migrations::embed_migrations;
use futures_util::future::join_all;
use log::info;
use scheduler::models::jobs::JobAssignmentBuffer;
use scheduler::models::providers::ProviderStorage;
use scheduler::models::workers::WorkerInfoStorage;
use scheduler::provider::scanner::ProviderScanner;
use scheduler::server_builder::ServerBuilder;
use scheduler::server_config::AccessControl;
use scheduler::service::delivery::{CancelPlanBuffer, JobDelivery};
use scheduler::service::generator::JobGenerator;
use scheduler::service::{ProcessorServiceBuilder, SchedulerServiceBuilder};
use scheduler::state::{ProcessorState, SchedulerState};
use scheduler::{
    BUILD_VERSION, DATABASE_URL, SCHEDULER_ENDPOINT, URL_GATEWAYS_LIST, URL_NODES_LIST,
};

use migration::{Migrator, MigratorTrait};
use scheduler::models::job_result_cache::JobResultCache;
use scheduler::persistence::services::job_result_service::JobResultService;
use scheduler::persistence::services::plan_service::PlanService;
use scheduler::persistence::services::provider_service::ProviderService;
use scheduler::persistence::services::WorkerService;
use scheduler::persistence::services::{get_sea_db_connection, JobService};
use scheduler::service::check_worker_health::WorkerHealthService;
use scheduler::service::submit_chain::ChainAdapter;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Load env file
    dotenv::dotenv().ok();
    // Init logger
    let _res = init_logger(&String::from("Fisherman Scheduler"));
    info!("BUILD_VERSION: {}", &*BUILD_VERSION);

    // let _matches = create_scheduler_app().get_matches();
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

    // Migration db
    Migrator::up(&db_conn, None).await?;

    let arc_conn = Arc::new(db_conn);
    let plan_service = Arc::new(PlanService::new(arc_conn.clone()));
    //Get worker infos
    let worker_service = Arc::new(WorkerService::new(arc_conn.clone()));
    let provider_service = Arc::new(ProviderService::new(arc_conn.clone()));
    let job_service = Arc::new(JobService::new(arc_conn.clone()));
    let all_workers = worker_service.clone().get_active().await;

    let socket_addr = SCHEDULER_ENDPOINT.as_str();
    // Keep the list of node and gateway that need use for generate verify and regular Job
    let provider_storage = Arc::new(ProviderStorage::default());
    log::debug!("Init with {:?} workers", all_workers.len());
    let worker_infos = Arc::new(WorkerInfoStorage::new(all_workers));
    let assigment_buffer = Arc::new(Mutex::new(JobAssignmentBuffer::default()));
    let cancel_plans_buffer: Arc<Mutex<CancelPlanBuffer>> =
        Arc::new(Mutex::new(CancelPlanBuffer::default()));

    let scheduler_service = SchedulerServiceBuilder::default().build();
    let result_service = Arc::new(JobResultService::new(arc_conn.clone()));
    let result_cache = Arc::new(JobResultCache::default());
    let processor_service = ProcessorServiceBuilder::default()
        .with_plan_service(plan_service.clone())
        .with_result_service(result_service.clone())
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
    let job_delivery = JobDelivery::new(assigment_buffer.clone(), cancel_plans_buffer.clone());

    // Check worker status task
    let worker_health = WorkerHealthService::new(worker_infos.clone(), result_cache.clone());
    // For onchain workers

    // Spawn tasks
    let task_worker_health = task::spawn(async move { worker_health.run().await });
    let task_provider_scanner = task::spawn(async move { provider_scanner.run().await });
    let task_job_generator = task::spawn(async move { job_generator.run().await });
    let task_job_delivery = task::spawn(async move { job_delivery.run().await });
    let task_subscribe_event_onchain_worker = task::spawn(async move {
        let adapter = ChainAdapter::new();
        adapter.subscribe_event_onchain_worker().await
    });

    let processor_state = ProcessorState::new(
        arc_conn.clone(),
        result_cache.clone(),
        plan_service.clone(),
        job_service.clone(),
        result_service.clone(),
        worker_infos,
        cancel_plans_buffer,
    );
    info!("Init http service ");
    let server = ServerBuilder::default()
        .with_entry_point(socket_addr)
        .with_access_control(access_control)
        .with_scheduler_state(scheduler_state)
        .with_processor_state(processor_state)
        .build(scheduler_service, processor_service);
    //let task_serve = server.serve();
    let task_serve = task::spawn(async move { server.serve().await });
    // Run all spawn task
    let tasks = vec![
        task_provider_scanner,
        task_job_generator,
        task_job_delivery,
        task_serve,
        task_worker_health,
        task_subscribe_event_onchain_worker,
    ];
    let _res = join_all(tasks).await;

    Ok(())
}
