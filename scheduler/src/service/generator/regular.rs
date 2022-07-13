use crate::models::component::ProviderPlan;
use crate::models::job_result_cache::{JobResultCache, TaskKey, TaskResultCache};
use crate::models::jobs::AssignmentBuffer;
use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerInfoStorage;
use crate::persistence::services::{JobService, PlanService};
use crate::persistence::PlanModel;
use crate::tasks::generator::TaskApplicant;
use anyhow::{anyhow, Error};
use common::component::ComponentInfo;
use common::job_manage::JobRole;
use common::jobs::Job;
use common::util::{get_current_time, warning_if_error};
use common::workers::MatchedWorkers;
use common::Timestamp;
use log::{debug, info, warn};
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct RegularJobGenerator {
    pub db_conn: Arc<DatabaseConnection>,
    pub plan_service: Arc<PlanService>,
    pub providers: Arc<ProviderStorage>,
    pub worker_infos: Arc<Mutex<WorkerInfoStorage>>,
    pub tasks: Vec<Arc<dyn TaskApplicant>>,
    pub job_service: Arc<JobService>,
    pub assignments: Arc<Mutex<AssignmentBuffer>>,
    pub result_cache: Arc<Mutex<JobResultCache>>,
}

impl RegularJobGenerator {
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
        let res = self.job_service.save_jobs(&jobs).await;
        warning_if_error("save_jobs return error", res);
        let res = self
            .plan_service
            .update_plans_as_generated(Vec::from_iter(gen_plans))
            .await;
        warning_if_error("update_plans_as_generated return error", res);
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
    pub async fn generate_regular_jobs(&mut self) -> Result<(), Error> {
        let mut total_assignment_buffer = AssignmentBuffer::default();
        let components = self.providers.get_active_providers().await;
        debug!("Found {} active providers", components.len());
        {
            let mut cache = self.result_cache.lock().await;
            for component in components.iter() {
                let provider_cache = cache
                    .result_cache_map
                    .entry(component.id.clone())
                    .or_insert(Default::default());
                //let provider_plan = Self::create_provider_plan(component);
                if let Ok(assignment_buffer) = self
                    .generate_regular_provider_jobs(component, provider_cache)
                    .await
                {
                    total_assignment_buffer.append(assignment_buffer);
                }
            }
            info!("There is {} jobs in cache.", cache.get_jobs_number(),);
        }
        let AssignmentBuffer {
            jobs,
            list_assignments,
        } = total_assignment_buffer;
        //info!("There is {} components", components.len());
        info!("There is {} gen_jobs", jobs.len(),);
        info!(
            "There is {} job_assignments {:?}",
            list_assignments.len(),
            list_assignments
        );

        if list_assignments.len() > 0 {
            let res = self
                .job_service
                .save_job_assignments(&list_assignments)
                .await;
            warning_if_error("save_job_assignments return error", res);
            self.assignments
                .lock()
                .await
                .add_assignments(list_assignments);
            //Store job assignments to db
        }
        if jobs.len() > 0 {
            //Store jobs to db
            let res = self.store_jobs(jobs).await;
            warning_if_error("store_jobs return error", res);
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
        provider: &ComponentInfo,
        provider_result_cache: &mut HashMap<TaskKey, TaskResultCache>,
    ) -> Result<AssignmentBuffer, anyhow::Error> {
        let mut assignment_buffer = AssignmentBuffer::default();
        let matched_workers = self
            .worker_infos
            .lock()
            .await
            .match_workers(provider)
            .unwrap_or(MatchedWorkers::default());

        for task in self.tasks.iter() {
            if !task.can_apply(provider) {
                continue;
            }
            let task_name = task.get_name();
            // Check if there is task result
            let latest_task_update = provider_result_cache
                .iter()
                .filter(|(key, _)| key.task_type.as_str() == task_name.as_str())
                .map(|(key, value)| (key.task_name.clone(), value.get_latest_update_time()))
                .collect::<HashMap<String, Timestamp>>();
            debug!(
                "latest_task_update of task {} for provider {} {}: {:?}",
                &task_name,
                &provider.component_type.to_string(),
                &provider.ip,
                latest_task_update
            );
            let plan_id = format!("{}-{}", JobRole::Regular.to_string(), provider.id);
            if let Ok(applied_jobs) = task.apply_with_cache(
                &plan_id,
                &provider,
                JobRole::Regular,
                &matched_workers,
                latest_task_update,
            ) {
                if applied_jobs.jobs.len() > 0 {
                    debug!(
                        "Generated {} regular jobs for provider {}, {:?}",
                        &applied_jobs.jobs.len(),
                        &provider.component_type.to_string(),
                        &provider.ip
                    );
                    //Update provider_result_cache
                    for job in applied_jobs.jobs.iter() {
                        let task_key = TaskKey {
                            task_type: job.job_type.clone(),
                            task_name: job.job_name.clone(),
                        };
                        let current_time = get_current_time();
                        debug!(
                            "Set update time of task {:?} for provider {} to {}",
                            &task_key, &provider.ip, current_time
                        );
                        provider_result_cache
                            .entry(task_key)
                            .or_insert(TaskResultCache::default())
                            .update_time = current_time;
                    }
                }
                assignment_buffer.append(applied_jobs);
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
