pub mod regular;
pub mod verification;

use crate::models::job_result_cache::JobResultCache;
use crate::models::jobs::JobAssignmentBuffer;
use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerInfoStorage;
use crate::persistence::services::{JobService, PlanService};
use crate::tasks::generator::get_tasks;
use crate::{CONFIG, CONFIG_DIR, JOB_VERIFICATION_GENERATOR_PERIOD};
use common::job_manage::JobRole;
use common::task_spawn;
use futures_util::future::join;
use log::{error, info};
pub use regular::RegularJobGenerator;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
pub use verification::VerificationJobGenerator;

trait JobGeneratorTrait {
    fn get_assignment(&self) -> Arc<Mutex<JobAssignmentBuffer>>;
}

impl JobGeneratorTrait for RegularJobGenerator {
    fn get_assignment(&self) -> Arc<Mutex<JobAssignmentBuffer>> {
        self.assignments.clone()
    }
}

impl JobGeneratorTrait for VerificationJobGenerator {
    fn get_assignment(&self) -> Arc<Mutex<JobAssignmentBuffer>> {
        self.assignments.clone()
    }
}

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
        worker_infos: Arc<WorkerInfoStorage>,
        job_service: Arc<JobService>,
        assignments: Arc<Mutex<JobAssignmentBuffer>>,
        result_cache: Arc<JobResultCache>,
    ) -> Self {
        //Load config
        let config_dir = CONFIG_DIR.as_str();
        // let path = format!("{}/task_master.json", config_dir);
        let path = Path::new(config_dir).join("task_master.json");
        let json = std::fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("Error {:?}. Path not found {:?}", err, path);
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
            result_cache,
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
            regular.result_cache.init_cache(assignments);
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

        let res = join(verification_task, regular_task).await;
        error!("Generator thread running error {res:?}");
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use anyhow::{anyhow, Error};

    use crate::persistence::services::WorkerService;
    use crate::persistence::PlanModel;

    use common::component::ComponentType;
    use itertools::Itertools;
    use log::info;
    use test_util::helper::{
        init_logging, load_env, mock_component_info, mock_db_connection, mock_worker,
        ChainTypeForTest, CountItems,
    };
    use tokio::task;
    use JobGeneratorTrait;

    const TEST_TIMEOUT: u64 = 30;

    #[tokio::test]
    async fn test_main_generator_verification_node() -> Result<(), Error> {
        load_env();
        init_logging();
        let db_conn = mock_db_connection();

        let arc_conn = Arc::new(db_conn);
        let plan_service = Arc::new(PlanService::new(arc_conn.clone()));
        //Get worker infos
        let worker_service = Arc::new(WorkerService::new(arc_conn.clone()));
        let job_service = Arc::new(JobService::new(arc_conn.clone()));
        let mut all_workers = worker_service.clone().get_active().await;
        all_workers.push(mock_worker("worker_id"));

        // Keep the list of node and gateway
        let provider_storage = Arc::new(ProviderStorage::default());
        log::debug!("Init with {:?} workers", all_workers.len());
        let worker_infos = Arc::new(WorkerInfoStorage::new(all_workers));
        // Keep the list of assignment Job
        let assigment_buffer = Arc::new(Mutex::new(JobAssignmentBuffer::default()));
        let result_cache = Arc::new(JobResultCache::default());

        let job_generator = JobGenerator::new(
            arc_conn.clone(),
            plan_service.clone(),
            provider_storage.clone(),
            worker_infos.clone(),
            job_service.clone(),
            assigment_buffer.clone(),
            result_cache.clone(),
        );
        let assignment = job_generator.verification.get_assignment();
        task::spawn(async move { job_generator.run().await });

        sleep(Duration::from_secs(1)).await;
        // Check Node
        let com_type = ComponentType::Node;
        let com_1 = mock_component_info("com_1", &ChainTypeForTest::Eth, &com_type);
        let com_2 = mock_component_info("com_2", &ChainTypeForTest::Dot, &com_type);

        let plan_model = PlanModel {
            id: 0,
            plan_id: "".to_string(),
            provider_id: "".to_string(),
            request_time: 0,
            finish_time: None,
            result: None,
            message: None,
            status: "".to_string(),
            phase: "".to_string(),
            expiry_time: 0,
        };
        provider_storage
            .add_verify_node(plan_model.clone(), com_1.clone())
            .await;
        provider_storage
            .add_verify_node(plan_model.clone(), com_2.clone())
            .await;
        let now = Instant::now();

        // Todo: get config for checking
        let expect_job_names: CountItems<String> = CountItems::new(vec![
            "RoundTripTime".to_string(),
            "LatestBlock".to_string(),
            "VerifyEthNode".to_string(),
            "VerifyDotNode".to_string(),
            "EthWebsocket".to_string(),
            "DotWebsocket".to_string(),
        ]);
        let expect_len = expect_job_names.len();
        let mut job_names = CountItems::new(vec![]);
        loop {
            if now.elapsed().as_secs() > TEST_TIMEOUT {
                return Err(anyhow!("Test Timeout"));
            }
            {
                println!("lock assigment_buffer");
                let lock = assignment.lock().await;
                println!("assigment_buffer: {:#?}", lock);

                for job_assign in lock.list_assignments.iter() {
                    job_names.add_item(job_assign.job.job_name.to_string());
                }
                println!(
                    "job_names.sum_len: {}, expect_len: {}",
                    job_names.len(),
                    expect_len
                );
                if job_names.len() >= expect_len {
                    break;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
        println!("{:?} == {:?}", expect_job_names.keys(), job_names.keys());
        for key in expect_job_names.keys() {
            assert!(job_names.keys().contains(key));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_main_generator_verification_gateway() -> Result<(), Error> {
        load_env();
        //init_logging();
        let db_conn = mock_db_connection();

        let arc_conn = Arc::new(db_conn);
        let plan_service = Arc::new(PlanService::new(arc_conn.clone()));
        //Get worker infos
        let worker_service = Arc::new(WorkerService::new(arc_conn.clone()));
        let job_service = Arc::new(JobService::new(arc_conn.clone()));
        let mut all_workers = worker_service.clone().get_active().await;
        all_workers.push(mock_worker("worker_id"));

        // Keep the list of node and gateway
        let provider_storage = Arc::new(ProviderStorage::default());
        log::debug!("Init with {:?} workers", all_workers.len());
        let worker_infos = Arc::new(WorkerInfoStorage::new(all_workers));
        // Keep the list of assignment Job
        let assigment_buffer = Arc::new(Mutex::new(JobAssignmentBuffer::default()));
        let result_cache = Arc::new(JobResultCache::default());

        let job_generator = JobGenerator::new(
            arc_conn.clone(),
            plan_service.clone(),
            provider_storage.clone(),
            worker_infos.clone(),
            job_service.clone(),
            assigment_buffer.clone(),
            result_cache.clone(),
        );
        let assignment = job_generator.verification.get_assignment();
        task::spawn(async move { job_generator.run().await });

        sleep(Duration::from_secs(1)).await;
        // Check Node
        let com_type = ComponentType::Gateway;
        let com_1 = mock_component_info("com_1", &ChainTypeForTest::Eth, &com_type);
        let com_2 = mock_component_info("com_2", &ChainTypeForTest::Dot, &com_type);

        let plan_model = PlanModel {
            id: 0,
            plan_id: "".to_string(),
            provider_id: "".to_string(),
            request_time: 0,
            finish_time: None,
            result: None,
            message: None,
            status: "".to_string(),
            phase: "".to_string(),
            expiry_time: 0,
        };
        provider_storage
            .add_verify_node(plan_model.clone(), com_1.clone())
            .await;
        provider_storage
            .add_verify_node(plan_model.clone(), com_2.clone())
            .await;
        let now = Instant::now();

        // Todo: get config for checking
        let expect_job_names: CountItems<String> = CountItems::new(vec![
            "RoundTripTime".to_string(),
            "EthWebsocket".to_string(),
            "DotWebsocket".to_string(),
            "VerifyGateway".to_string(),
        ]);
        let expect_len = expect_job_names.len();
        let mut job_names = CountItems::new(vec![]);
        loop {
            if now.elapsed().as_secs() > TEST_TIMEOUT {
                return Err(anyhow!("Test Timeout"));
            }
            {
                println!("lock assigment_buffer");
                let lock = assignment.lock().await;
                println!("assigment_buffer: {:#?}", lock);

                for job_assign in lock.list_assignments.iter() {
                    job_names.add_item(job_assign.job.job_name.to_string());
                }
                println!(
                    "job_names.sum_len: {}, expect_len: {}",
                    job_names.len(),
                    expect_len
                );
                if job_names.len() >= expect_len {
                    break;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
        println!("{:?} == {:?}", expect_job_names.keys(), job_names.keys());
        for key in expect_job_names.keys() {
            assert!(job_names.keys().contains(key));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_main_generator_regular_gateway() -> Result<(), Error> {
        load_env();
        //init_logging();
        let db_conn = mock_db_connection();

        let arc_conn = Arc::new(db_conn);
        let plan_service = Arc::new(PlanService::new(arc_conn.clone()));
        //Get worker infos
        let worker_service = Arc::new(WorkerService::new(arc_conn.clone()));
        let job_service = Arc::new(JobService::new(arc_conn.clone()));
        let mut all_workers = worker_service.clone().get_active().await;
        all_workers.push(mock_worker("worker_id"));

        // Keep the list of node and gateway
        let provider_storage = Arc::new(ProviderStorage::default());
        log::debug!("Init with {:?} workers", all_workers.len());
        let worker_infos = Arc::new(WorkerInfoStorage::new(all_workers));
        // Keep the list of assignment Job
        let assigment_buffer = Arc::new(Mutex::new(JobAssignmentBuffer::default()));
        let result_cache = Arc::new(JobResultCache::default());

        let job_generator = JobGenerator::new(
            arc_conn.clone(),
            plan_service.clone(),
            provider_storage.clone(),
            worker_infos.clone(),
            job_service.clone(),
            assigment_buffer.clone(),
            result_cache.clone(),
        );
        let assignment = job_generator.regular.get_assignment();
        task::spawn(async move { job_generator.run().await });

        sleep(Duration::from_secs(1)).await;
        // Check Node
        let com_type = ComponentType::Gateway;
        let com_1 = mock_component_info("com_1", &ChainTypeForTest::Eth, &com_type);
        let com_2 = mock_component_info("com_2", &ChainTypeForTest::Dot, &com_type);

        let _plan_model = PlanModel {
            id: 0,
            plan_id: "".to_string(),
            provider_id: "".to_string(),
            request_time: 0,
            finish_time: None,
            result: None,
            message: None,
            status: "".to_string(),
            phase: "".to_string(),
            expiry_time: 0,
        };
        provider_storage
            .update_components_list(com_type, vec![com_1.clone(), com_2.clone()])
            .await;
        let now = Instant::now();

        // Todo: get config for checking
        let expect_job_names: CountItems<String> = CountItems::new(vec![
            "RoundTripTime".to_string(),
            "RoundTripTime".to_string(),
        ]);
        let expect_len = expect_job_names.sum_len();
        let mut job_names = CountItems::new(vec![]);
        loop {
            if now.elapsed().as_secs() > TEST_TIMEOUT {
                return Err(anyhow!("Test Timeout"));
            }

            {
                let lock = assignment.lock().await;
                info!("assigment_buffer: {:#?}", lock);

                for job_assign in lock.list_assignments.iter() {
                    job_names.add_item(job_assign.job.job_name.to_string());
                    info!("job_names: {:?}", job_names);
                }
                if job_names.sum_len() >= expect_len {
                    println!("assigment_buffer: {:#?}", lock);
                    break;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
        assert_eq!(expect_job_names, job_names);

        Ok(())
    }
}
