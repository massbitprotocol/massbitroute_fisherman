use crate::models::component::ProviderPlan;
use crate::models::job_result_cache::{JobResultCache, TaskKey, TaskResultCache};
use crate::models::jobs::AssignmentBuffer;
use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerInfoStorage;
use crate::persistence::services::{JobService, PlanService};
use crate::persistence::PlanModel;
use crate::service::judgment::http_latestblock_judg::CacheKey;
use crate::tasks::generator::{get_tasks, TaskApplicant};
use crate::{CONFIG, CONFIG_DIR, JOB_VERIFICATION_GENERATOR_PERIOD};
use anyhow::{anyhow, Error};
use common::component::{ComponentInfo, Zone};
use common::job_manage::{JobDetail, JobRole};
use common::jobs::{Job, JobAssignment, JobResult};
use common::models::plan_entity::PlanStatus;
use common::models::PlanEntity;
use common::tasks::LoadConfig;
use common::util::get_current_time;
use common::workers::{MatchedWorkers, Worker, WorkerInfo};
use common::{task_spawn, ComponentId, Timestamp, WorkerId};
use futures_util::future::join;
use log::{debug, info, trace, warn};
use sea_orm::sea_query::IndexType::Hash;
use sea_orm::{DatabaseConnection, DbErr, TransactionTrait};
use serde::{Deserialize, Serialize};
use slog::log;
use std::collections::{HashMap, HashSet, VecDeque};
use std::default::Default;
use std::mem::take;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::sleep;

#[derive(Default)]
pub struct JobGenerator {
    verification: DetailJobGenerator,
    regular: DetailJobGenerator,
}

#[derive(Default)]
pub struct DetailJobGenerator {
    db_conn: Arc<DatabaseConnection>,
    plan_service: Arc<PlanService>,
    providers: Arc<ProviderStorage>,
    worker_infos: Arc<Mutex<WorkerInfoStorage>>,
    tasks: Vec<Arc<dyn TaskApplicant>>,
    job_service: Arc<JobService>,
    assignments: Arc<Mutex<AssignmentBuffer>>,
    result_cache: Arc<Mutex<JobResultCache>>,
}
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct TaskConfig {
    #[serde(default)]
    pub regular: Vec<String>,
    #[serde(default)]
    pub verification: Vec<String>,
}

impl DetailJobGenerator {
    /*
     * Generate verification jobs with task dependencies
     */
    pub async fn generate_verification_jobs(&mut self) {
        let providers = self.providers.get_components_for_verifications().await;
        if providers.len() > 0 {
            log::debug!(
                "Generate verification jobs for {} providers",
                providers.len()
            );
            let mut total_assignment_buffer = AssignmentBuffer::default();
            for provider_plan in providers.iter() {
                let matched_workers = self
                    .worker_infos
                    .lock()
                    .await
                    .match_workers(&provider_plan.provider)
                    .unwrap_or(MatchedWorkers::default());
                for task in self.tasks.iter() {
                    if !task.can_apply(&provider_plan.provider) {
                        continue;
                    }
                    let task_dependencies = task.get_task_dependencies();
                    if task_dependencies.is_empty() {
                        if let Ok(applied_jobs) = task.apply(
                            &provider_plan.plan.plan_id,
                            &provider_plan.provider,
                            JobRole::Verification,
                            &matched_workers,
                        ) {
                            total_assignment_buffer.append(applied_jobs);
                        }
                    } else {
                    }
                }
            }
            let AssignmentBuffer {
                mut jobs,
                mut list_assignments,
            } = total_assignment_buffer;
            if list_assignments.len() > 0 {
                self.job_service
                    .save_job_assignments(&list_assignments)
                    .await;
                self.assignments
                    .lock()
                    .await
                    .add_assignments(list_assignments);
                //Store job assignments to db
            }
            if jobs.len() > 0 {
                //Store jobs to db
                self.store_jobs(jobs).await;
            }
        }
    }

    /*
     * For verification job (long job) find next available worker
     */
    /*
    pub async fn _generate_verification_jobs(&mut self) {
        //let mut gen_jobs = HashMap::<Zone, Vec<Job>>::new();
        let mut total_assignment_buffer = AssignmentBuffer::default();
        // Get Nodes for verification
        let nodes = self.providers.pop_nodes_for_verifications().await;

        if nodes.len() > 0 {
            log::debug!("Generate verification jobs for {} nodes", nodes.len());
            for provider_plan in nodes.iter() {
                let matched_workers = self
                    .worker_infos
                    .lock()
                    .await
                    .match_workers(&provider_plan.provider)
                    .unwrap_or(MatchedWorkers::default());
                for task in self.tasks.iter() {
                    if !task.can_apply(&provider_plan.provider) {
                        continue;
                    }
                    if let Ok(applied_jobs) = task.apply(
                        &provider_plan.plan.plan_id,
                        &provider_plan.provider,
                        JobRole::Verification,
                        &matched_workers,
                    ) {
                        total_assignment_buffer.append(applied_jobs);
                    }
                }
            }
        }
        //Generate jobs for gateways
        let gateways = self.providers.pop_gateways_for_verifications().await;
        if gateways.len() > 0 {
            log::debug!("Generate verification jobs for {} gateways", gateways.len());
            for provider_plan in gateways.iter() {
                let matched_workers = self
                    .worker_infos
                    .lock()
                    .await
                    .match_workers(&provider_plan.provider)
                    .unwrap_or(MatchedWorkers::default());
                for task in self.tasks.iter() {
                    if !task.can_apply(&provider_plan.provider) {
                        continue;
                    }
                    if let Ok(applied_jobs) = task.apply(
                        &provider_plan.plan.plan_id,
                        &provider_plan.provider,
                        JobRole::Verification,
                        &matched_workers,
                    ) {
                        total_assignment_buffer.append(applied_jobs);
                    }
                }
            }
        }
        let AssignmentBuffer {
            mut jobs,
            mut list_assignments,
        } = total_assignment_buffer;
        if list_assignments.len() > 0 {
            self.job_service
                .save_job_assignments(&list_assignments)
                .await;
            self.assignments
                .lock()
                .await
                .add_assignments(list_assignments);
            //Store job assignments to db
        }
        if jobs.len() > 0 {
            //Store jobs to db
            self.store_jobs(jobs).await;
        }
    }
     */
    pub async fn store_jobs(&self, jobs: Vec<Job>) -> Result<(), anyhow::Error> {
        let mut gen_plans = HashSet::<String>::default();
        for job in jobs.iter() {
            gen_plans.insert(job.plan_id.clone());
        }
        warn!(
            "update_plans_as_generated {}: {:?}",
            gen_plans.len(),
            gen_plans
        );
        let tnx = self.db_conn.begin().await?;
        self.job_service.save_jobs(&jobs).await;
        self.plan_service
            .update_plans_as_generated(Vec::from_iter(gen_plans))
            .await;
        match tnx.commit().await {
            Ok(_) => {
                log::debug!("Transaction commited successful");
                Ok(())
            }
            Err(err) => {
                log::debug!("Transaction commited with error {:?}", &err);
                Err(anyhow!("{:?}", &err))
            }
        }
    }
    /*
    pub async fn store_jobs(
        &self,
        map_jobs: &HashMap<Zone, Vec<Job>>,
    ) -> Result<(), anyhow::Error> {
        if map_jobs.is_empty() {
            return Ok(());
        }
        let mut gen_plans = Vec::default();
        let tnx = self.db_conn.begin().await?;
        for (zone, jobs) in map_jobs.iter() {
            jobs.iter().for_each(|job| {
                if !gen_plans.contains(&job.plan_id) {
                    gen_plans.push(job.plan_id.clone());
                }
            });
            self.job_service.save_jobs(jobs).await;
        }
        warn!(
            "update_plans_as_generated {}: {:?}",
            gen_plans.len(),
            gen_plans
        );
        self.plan_service.update_plans_as_generated(gen_plans).await;
        match tnx.commit().await {
            Ok(_) => {
                log::debug!("Transaction commited successful");
                Ok(())
            }
            Err(err) => {
                log::debug!("Transaction commited with error {:?}", &err);
                Err(anyhow!("{:?}", &err))
            }
        }
    }
     */
    /*
    pub async fn assign_jobs(&self, map_jobs: &HashMap<Zone, Vec<Job>>) {
        let mut assignments = Vec::default();
        for (zone, jobs) in map_jobs.iter() {
            if let Some(workers) = self.worker_infos.lock().await.get_workers(zone) {
                if workers.len() > 0 {
                    jobs.iter().enumerate().for_each(|(ind, job)| {
                        let wind = ind % workers.len();
                        let worker: &Arc<Worker> = workers.get(wind).unwrap();
                        let job_assignment = JobAssignment::new(worker.clone(), job);
                        assignments.push(job_assignment);
                    });
                }
            }
        }
        self.assignments.lock().await.add_assignments(assignments);
    }
    */
    pub async fn generate_regular_jobs(&mut self) -> Result<(), Error> {
        let mut total_assignment_buffer = AssignmentBuffer::default();
        let mut components;
        {
            components = self.providers.clone_nodes_list().await;
            info!("There are {} node", components.len());
            components.append(&mut self.providers.clone_gateways_list().await);
            info!("There are {} component", components.len());
        }

        {
            let mut cache = self.result_cache.lock().await;
            for component in components.iter() {
                let provider_cache = cache
                    .result_cache_map
                    .entry(component.id.clone())
                    .or_insert(Default::default());
                let provider_plan = Self::create_provider_plan(component);
                if let Ok(assignment_buffer) = self
                    .generate_regular_provider_jobs(provider_plan, provider_cache)
                    .await
                {
                    total_assignment_buffer.append(assignment_buffer);
                }
            }
            info!("There is {} jobs in cache.", cache.get_jobs_number(),);
        }
        let AssignmentBuffer {
            mut jobs,
            mut list_assignments,
        } = total_assignment_buffer;
        //info!("There is {} components", components.len());
        info!("There is {} gen_jobs", jobs.len(),);
        info!(
            "There is {} job_assignments {:?}",
            list_assignments.len(),
            list_assignments
        );

        if list_assignments.len() > 0 {
            self.job_service
                .save_job_assignments(&list_assignments)
                .await;
            self.assignments
                .lock()
                .await
                .add_assignments(list_assignments);
            //Store job assignments to db
        }
        if jobs.len() > 0 {
            //Store jobs to db
            self.store_jobs(jobs).await;
        }
        Ok(())
    }

    fn create_provider_plan(component: &ComponentInfo) -> ProviderPlan {
        let plan_id = format!("{}-{}", JobRole::Regular.to_string(), component.id);
        let plan = PlanModel {
            id: Default::default(),
            plan_id,
            provider_id: component.id.clone(),
            request_time: get_current_time(),
            finish_time: None,
            result: None,
            message: None,
            status: "generated".to_string(),
            phase: JobRole::Regular.to_string(),
            expiry_time: Timestamp::MAX,
        };
        ProviderPlan::new(component.clone(), plan)
    }

    async fn generate_regular_provider_jobs(
        &self,
        provider_plan: ProviderPlan,
        provider_result_cache: &mut HashMap<TaskKey, TaskResultCache>,
    ) -> Result<AssignmentBuffer, anyhow::Error> {
        let mut assignment_buffer = AssignmentBuffer::default();
        let matched_workers = self
            .worker_infos
            .lock()
            .await
            .match_workers(&provider_plan.provider)
            .unwrap_or(MatchedWorkers::default());

        for task in self.tasks.iter() {
            if !task.can_apply(&provider_plan.provider) {
                continue;
            }
            // Check if there is task result
            let latest_task_update = provider_result_cache
                .iter()
                .filter(|(key, _)| key.task_type.as_str() == task.get_name().as_str())
                .map(|(key, value)| (key.task_name.clone(), value.get_latest_update_time()))
                .collect::<HashMap<String, Timestamp>>();
            trace!("latest_task_update: {:?}", latest_task_update);

            if let Ok(applied_jobs) = task.apply_with_cache(
                &provider_plan.plan.plan_id,
                &provider_plan.provider,
                JobRole::Regular,
                &matched_workers,
                latest_task_update,
            ) {
                assignment_buffer.append(applied_jobs);
            }
            //Update provider_result_cache
            for job in assignment_buffer.jobs.iter() {
                let task_key = TaskKey {
                    task_type: task.get_name().clone(),
                    task_name: job.job_name.clone(),
                };
                let cache = provider_result_cache
                    .entry(task_key)
                    .or_insert(TaskResultCache::default());
                cache.update_time = get_current_time();
            }

            /*
            if applied_jobs.len() > 0 {
                log::debug!(
                    "Create {} regular jobs for {:?} {:?}",
                    applied_jobs.len(),
                    &provider_plan.provider.component_type,
                    &provider_plan.provider.id
                );
                //Update latest timestamp in cache
                for job in applied_jobs.iter() {
                    let key = TaskKey {
                        task_type: job
                            .job_detail
                            .as_ref()
                            .map(|detail| detail.get_job_name())
                            .unwrap_or(job.job_name.clone()),
                        task_name: job.job_name.clone(),
                    };
                    let cache = provider_result_cache
                        .entry(key)
                        .or_insert(TaskResultCache::new(get_current_time()));
                    cache.reset_timestamp(get_current_time());
                }
                if let Ok(mut assignments) = task.assign_jobs(
                    &provider_plan.plan,
                    &provider_plan.provider,
                    &applied_jobs,
                    &matched_workers,
                ) {
                    job_assignments.append(&mut assignments);
                }
                gen_jobs
                    .entry(provider_plan.provider.zone.clone())
                    .or_insert(Vec::new())
                    .append(&mut applied_jobs);
            }
            */
            //task_result.create_time = get_current_time();
        }
        Ok(assignment_buffer)
    }
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
        let verification = DetailJobGenerator {
            db_conn: db_conn.clone(),
            plan_service: plan_service.clone(),
            providers: providers.clone(),
            worker_infos: worker_infos.clone(),
            tasks: get_tasks(config_dir, JobRole::Verification, &task_config.verification),
            job_service: job_service.clone(),
            assignments: assignments.clone(),
            result_cache: result_cache.clone(),
        };

        let regular = DetailJobGenerator {
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
    pub async fn run(mut self) {
        let JobGenerator {
            mut verification,
            mut regular,
        } = self;
        // Run Verification task
        let verification_task = task_spawn::spawn(async move {
            loop {
                let now = Instant::now();
                verification.generate_verification_jobs().await;
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
