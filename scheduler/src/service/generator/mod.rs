pub mod regular;
pub mod verification;
use crate::models::job_result_cache::JobResultCache;
use crate::models::jobs::AssignmentBuffer;
use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerInfoStorage;
use crate::persistence::services::{JobService, PlanService};
use crate::tasks::generator::get_tasks;
use crate::{CONFIG, CONFIG_DIR, JOB_VERIFICATION_GENERATOR_PERIOD};
use common::job_manage::JobRole;
use common::task_spawn;
use futures_util::future::join;
use log::info;
pub use regular::RegularJobGenerator;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
pub use verification::VerificationJobGenerator;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct TaskConfig {
    #[serde(default)]
    pub regular: Vec<String>,
    #[serde(default)]
    pub verification: Vec<String>,
}

#[derive(Default)]
pub struct JobGenerator {
    verification: VerificationJobGenerator,
    regular: RegularJobGenerator,
}

impl JobGenerator {
    pub fn new(
        db_conn: Arc<DatabaseConnection>,
        plan_service: Arc<PlanService>,
        providers: Arc<ProviderStorage>,
        worker_infos: Arc<Mutex<WorkerInfoStorage>>,
        job_service: Arc<JobService>,
        assignments: Arc<Mutex<AssignmentBuffer>>,
        result_cache: Arc<Mutex<JobResultCache>>,
    ) -> Self {
        //Load config
        let config_dir = CONFIG_DIR.as_str();
        let path = format!("{}/task_master.json", config_dir);
        let json = std::fs::read_to_string(path.as_str()).unwrap_or_else(|err| {
            panic!("Error {:?}. Path not found {}", err, path);
        });
        let task_config: TaskConfig = serde_json::from_str(&*json).unwrap();
        let verification = VerificationJobGenerator {
            db_conn: db_conn.clone(),
            plan_service: plan_service.clone(),
            providers: providers.clone(),
            worker_infos: worker_infos.clone(),
            tasks: get_tasks(config_dir, JobRole::Verification, &task_config.verification),
            job_service: job_service.clone(),
            assignments: assignments.clone(),
            result_cache: result_cache.clone(),
            processing_plans: vec![],
            waiting_tasks: vec![],
        };

        let regular = RegularJobGenerator {
            db_conn,
            plan_service,
            providers,
            worker_infos,
            tasks: get_tasks(config_dir, JobRole::Regular, &task_config.regular),
            job_service,
            assignments,
            result_cache: result_cache.clone(),
        };

        JobGenerator {
            verification,
            regular,
        }
    }
    pub async fn run(self) {
        let JobGenerator {
            mut verification,
            mut regular,
        } = self;
        // Run Verification task
        let verification_task = task_spawn::spawn(async move {
            loop {
                let now = Instant::now();
                verification.generate_jobs().await;
                if now.elapsed().as_secs() < JOB_VERIFICATION_GENERATOR_PERIOD {
                    sleep(Duration::from_secs(
                        JOB_VERIFICATION_GENERATOR_PERIOD - now.elapsed().as_secs(),
                    ))
                    .await;
                }
            }
        });

        let assignments = regular
            .job_service
            .get_job_assignments()
            .await
            .unwrap_or_default();
        {
            let mut lock = regular.result_cache.lock().await;
            lock.init_cache(assignments);
        }

        // Run Regular task
        let regular_task = task_spawn::spawn(async move {
            info!("Run Regular task");
            loop {
                info!("Start generate_regular_jobs");
                let res = regular.generate_regular_jobs().await;
                info!("generate_regular_jobs result: {:?} ", res);
                sleep(Duration::from_secs(
                    CONFIG.regular_plan_generate_interval as u64,
                ))
                .await;
            }
        });

        join(verification_task, regular_task).await;
    }
}
