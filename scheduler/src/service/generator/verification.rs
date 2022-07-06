use crate::models::component::ProviderPlan;
use crate::models::job_result_cache::{JobResultCache, TaskKey, TaskResultCache};
use crate::models::jobs::AssignmentBuffer;
use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerInfoStorage;
use crate::models::TaskDependency;
use crate::persistence::services::{JobService, PlanService};
use crate::persistence::PlanModel;
use crate::service::judgment::http_latestblock_judg::CacheKey;
use crate::service::judgment::JudgmentsResult;
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
use std::mem::take;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::sleep;

#[derive(Default)]
pub struct VerificationJobGenerator {
    pub db_conn: Arc<DatabaseConnection>,
    pub plan_service: Arc<PlanService>,
    pub providers: Arc<ProviderStorage>,
    pub worker_infos: Arc<Mutex<WorkerInfoStorage>>,
    pub tasks: Vec<Arc<dyn TaskApplicant>>,
    pub job_service: Arc<JobService>,
    pub assignments: Arc<Mutex<AssignmentBuffer>>,
    pub result_cache: Arc<Mutex<JobResultCache>>,
    pub generated_plans: Vec<Arc<ProviderPlan>>,
}

impl VerificationJobGenerator {
    /*
     * Generate verification jobs with task dependencies
     */
    pub async fn generate_jobs(&mut self) {
        // let expired_plans = self.providers.get_expired_verification_plans().await;
        // if expired_plans.len() > 0 {
        //     self.clean_expired_plan(expired_plans).await;
        // }
        let mut providers = self.providers.pop_components_for_verifications().await;
        if providers.len() > 0 {
            log::debug!(
                "Generate verification jobs for {} providers",
                providers.len()
            );
            let mut total_assignment_buffer = AssignmentBuffer::default();
            for provider_plan in providers.iter() {
                if let Ok(matched_workers) = self
                    .worker_infos
                    .lock()
                    .await
                    .match_workers(&provider_plan.provider)
                {
                    self.generate_provider_plan_jobs(
                        provider_plan.clone(),
                        &matched_workers,
                        &mut total_assignment_buffer,
                    );
                } else {
                    log::warn!(
                        "Workers not found for provider {:?}",
                        &provider_plan.provider
                    );
                }
            }
            self.generated_plans.append(&mut providers);
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
    async fn generate_provider_plan_jobs(
        &self,
        provider_plan: Arc<ProviderPlan>,
        matched_workers: &MatchedWorkers,
        assignment_buffer: &mut AssignmentBuffer,
    ) {
        for task in self.tasks.iter() {
            if !task.can_apply(&provider_plan.provider) {
                continue;
            }
            if self
                .check_task_for_generation(
                    provider_plan.clone(),
                    task.get_name().as_str(),
                    task.get_task_dependencies(),
                )
                .await
            {
                if let Ok(mut applied_jobs) = task.apply(
                    &provider_plan.plan.plan_id,
                    &provider_plan.provider,
                    JobRole::Verification,
                    &matched_workers,
                ) {
                    //Todo: Improve this, don't create redundant jobs
                    if applied_jobs.jobs.len() > 0 {
                        applied_jobs = self.remote_duplicated_jobs(applied_jobs).await;
                    }
                    if applied_jobs.jobs.len() > 0 {
                        assignment_buffer.append(applied_jobs);
                    }
                }
            }
        }
    }
    async fn clean_expired_plan(&self, plans: Vec<Arc<ProviderPlan>>) {
        //Todo: Implementation
    }
    async fn remote_duplicated_jobs(
        &self,
        assignment_buffer: AssignmentBuffer,
    ) -> AssignmentBuffer {
        let first_job = assignment_buffer.jobs.first().unwrap();
        let plan_id = first_job.plan_id.as_str();
        let task_type = first_job.job_type.as_str();
        let exist_tasks = self
            .assignments
            .lock()
            .await
            .get_exist_jobs(plan_id, task_type);
        assignment_buffer.remove_redundant_jobs(&exist_tasks)
    }
    /*
     * Check if task is not generated for current plan,
     * Dependencies already have results
     */
    /*
     * true: task ready for generation
     * false: task must is in waiting state
     */
    async fn check_task_for_generation(
        &self,
        provider_plan: Arc<ProviderPlan>,
        current_task_type: &str,
        task_dependencies: TaskDependency,
    ) -> bool {
        //No dependencies
        if task_dependencies.is_empty() {
            return true;
        }
        for (task_type, task_names) in task_dependencies {
            for name in task_names {
                if let Some(task_result) = self.result_cache.lock().await.get_judg_result(
                    provider_plan.clone(),
                    &task_type,
                    &name,
                ) {
                    if task_result != JudgmentsResult::Pass {
                        debug!(
                            "Task {}.{} is not passed while try to generate task {}",
                            &name, &task_type, current_task_type
                        );
                        return false;
                    }
                }
            }
        }
        return true;
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
}
