use crate::models::component::ProviderPlan;
use crate::models::job_result_cache::JobResultCache;
use crate::models::jobs::AssignmentBuffer;
use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerInfoStorage;
use crate::models::TaskDependency;
use crate::persistence::services::{JobService, PlanService};
use crate::persistence::PlanModel;

use crate::service::judgment::JudgmentsResult;
use crate::tasks::generator::TaskApplicant;

use anyhow::anyhow;
use common::component::ComponentInfo;
use common::job_manage::JobRole;
use common::jobs::Job;

use common::tasks::LoadConfig;
use common::util::get_current_time;
use common::workers::MatchedWorkers;
use common::Timestamp;

use log::{debug, warn};

use sea_orm::{DatabaseConnection, TransactionTrait};

use std::collections::HashSet;

use std::sync::Arc;

use tokio::sync::Mutex;

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
    pub processing_plans: Vec<Arc<ProviderPlan>>,
    pub waiting_tasks: Vec<WaitingProviderPlanTask>,
}
/*
 * Struct for store dependent task, waiting for others' results
 */
#[derive(Clone)]
pub struct WaitingProviderPlanTask {
    provider_plan: Arc<ProviderPlan>,
    tasks: Vec<Arc<dyn TaskApplicant>>,
}

impl WaitingProviderPlanTask {
    fn new(provider_plan: Arc<ProviderPlan>) -> Self {
        Self {
            provider_plan,
            tasks: Vec::new(),
        }
    }
    pub fn add_task(&mut self, task: Arc<dyn TaskApplicant>) {
        self.tasks.push(task);
    }
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}
impl VerificationJobGenerator {
    /*
     * Generate verification jobs with task dependencies
     */
    pub async fn generate_jobs(&mut self) {
        //Clean up processing plan base on received result
        self.clean_processing_plan();
        //renew expired plan
        self.renew_expired_plan();
        //Generate jobs for waiting task from previous iteration base on new incoming results
        if self.waiting_tasks.len() > 0 {
            log::debug!(
                "Try generating jobs for {} waiting tasks in queue",
                self.waiting_tasks.len()
            );
            self.generate_jobs_for_waiting_tasks().await;
        }

        let mut providers = self.providers.pop_components_for_verifications().await;
        if providers.len() > 0 {
            log::debug!(
                "Generate verification jobs for {} providers with {} tasks",
                providers.len(),
                self.tasks.len()
            );
            let mut total_assignment_buffer = AssignmentBuffer::default();
            for provider_plan in providers.iter() {
                if let Ok(matched_workers) = self
                    .worker_infos
                    .lock()
                    .await
                    .match_workers(&provider_plan.provider)
                {
                    let plan_task = WaitingProviderPlanTask {
                        provider_plan: provider_plan.clone(),
                        tasks: self.tasks.clone(),
                    };
                    let waiting_task = self
                        .generate_provider_plan_jobs(
                            &plan_task,
                            &matched_workers,
                            &mut total_assignment_buffer,
                        )
                        .await;
                    if !waiting_task.is_empty() {
                        self.waiting_tasks.push(waiting_task);
                    }
                } else {
                    log::debug!(
                        "Workers not found for provider {:?}",
                        &provider_plan.provider
                    );
                }
            }
            self.processing_plans.append(&mut providers);
            self.process_assignment_buffer(total_assignment_buffer)
                .await;
        }
    }
    fn clean_processing_plan(&mut self) {}
    fn renew_expired_plan(&mut self) {
        let mut expired_plans = Vec::new();
        let current_timestamp = get_current_time();
        self.processing_plans.retain(|plan| {
            expired_plans.push(plan.clone());
            if plan.plan.expiry_time < current_timestamp {
                false
            } else {
                true
            }
        });
    }
    async fn generate_jobs_for_waiting_tasks(&mut self) {
        let mut assignment_buffer = AssignmentBuffer::default();
        let mut waiting_tasks = Vec::new();
        for item in self.waiting_tasks.iter() {
            let provider_plan = item.provider_plan.clone();
            if let Ok(matched_workers) = self
                .worker_infos
                .lock()
                .await
                .match_workers(&provider_plan.provider)
            {
                let waiting_task = self
                    .generate_provider_plan_jobs(item, &matched_workers, &mut assignment_buffer)
                    .await;
                if !waiting_task.is_empty() {
                    waiting_tasks.push(waiting_task);
                }
            } else {
                waiting_tasks.push(item.clone());
            }
        }
        self.waiting_tasks = waiting_tasks;
        self.process_assignment_buffer(assignment_buffer).await;
    }
    async fn process_assignment_buffer(&self, assignment_buffer: AssignmentBuffer) {
        let AssignmentBuffer {
            jobs,
            list_assignments,
        } = assignment_buffer;
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
    async fn generate_provider_plan_jobs(
        &self,
        plan_task: &WaitingProviderPlanTask,
        matched_workers: &MatchedWorkers,
        assignment_buffer: &mut AssignmentBuffer,
    ) -> WaitingProviderPlanTask {
        let provider_plan = plan_task.provider_plan.clone();
        let mut waiting_task = WaitingProviderPlanTask::new(provider_plan.clone());
        for task in plan_task.tasks.iter() {
            if !task.can_apply(&provider_plan.provider) {
                log::debug!(
                    "Task {} cannot apply for component {}",
                    task.get_name(),
                    &provider_plan.provider
                );
                continue;
            }
            if self
                .check_task_dependencies(
                    provider_plan.clone(),
                    task.get_name().as_str(),
                    task.get_task_dependencies(),
                )
                .await
            {
                log::debug!("Generate jobs for task {}", task.get_name());
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
            } else {
                waiting_task.add_task(task.clone());
                log::debug!("Task {} is not ready for job generation", task.get_name());
            }
        }
        waiting_task
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
    async fn check_task_dependencies(
        &self,
        provider_plan: Arc<ProviderPlan>,
        current_task_type: &str,
        task_dependencies: TaskDependency,
    ) -> bool {
        //No dependencies
        log::debug!(
            "Check task dependencies for {:?} with dependencies {:?}",
            current_task_type,
            &task_dependencies
        );
        if task_dependencies.is_empty() {
            return true;
        }
        for (task_type, task_names) in task_dependencies {
            for name in task_names {
                let task_result = self
                    .result_cache
                    .lock()
                    .await
                    .get_judg_result(provider_plan.clone(), &task_type, &name)
                    .unwrap_or(JudgmentsResult::Unfinished);
                if task_result != JudgmentsResult::Pass {
                    debug!(
                        "Task {}.{} is not passed while try to generate task {}",
                        &name, &task_type, current_task_type
                    );
                    return false;
                }
            }
        }
        return true;
    }
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
