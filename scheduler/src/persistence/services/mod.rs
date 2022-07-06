pub mod job_result_service;
pub mod job_service;
pub mod plan_service;
pub mod provider_service;
pub mod worker_service;

pub use job_result_service::JobResultService;
pub use job_service::JobService;
pub use plan_service::PlanService;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;
pub use worker_service::WorkerService;

pub async fn get_sea_db_connection(url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(url.to_owned());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false); // Enable logging sqlx

    Database::connect(opt).await
}
