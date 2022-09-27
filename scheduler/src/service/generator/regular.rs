use crate::models::component::ProviderPlan;
use crate::models::job_result_cache::{JobResultCache, TaskKey};
use crate::models::jobs::JobAssignmentBuffer;
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
use common::{ComponentId, Timestamp};
use log::{debug, error, info, warn};
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct RegularJobGenerator {
    pub db_conn: Arc<DatabaseConnection>,
    pub plan_service: Arc<PlanService>,
    pub providers: Arc<ProviderStorage>,
    pub worker_infos: Arc<WorkerInfoStorage>,
    pub tasks: Vec<Arc<dyn TaskApplicant>>,
    pub job_service: Arc<JobService>,
    pub assignments: Arc<Mutex<JobAssignmentBuffer>>,
    pub result_cache: Arc<JobResultCache>,
    pub generated_provider: Arc<Mutex<HashSet<ComponentId>>>,
}

impl RegularJobGenerator {
    pub async fn store_jobs(&self, jobs: Vec<Job>) -> Result<(), anyhow::Error> {
        let mut gen_plans = HashSet::<String>::default();
        for job in jobs.iter() {
            gen_plans.insert(job.plan_id.clone());
        }
        info!("store_jobs update {} generated plans", gen_plans.len(),);
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
        let mut total_assignment_buffer = JobAssignmentBuffer::default();
        let components = self.providers.get_active_providers().await;
        if components.is_empty() {
            warn!("There are no active component");
            return Ok(());
        }
        debug!("Found {} active providers", components.len());
        {
            for component in components.iter() {
                let mut latest_update = self
                    .result_cache
                    .get_latest_update_task(&component.id)
                    .await;
                //let provider_plan = Self::create_provider_plan(component);
                if let Ok(assignment_buffer) = self
                    .generate_regular_provider_jobs(component, &mut latest_update)
                    .await
                {
                    total_assignment_buffer.append(assignment_buffer);
                } else {
                }
            }
            //info!("There is {} jobs in cache.", cache.get_jobs_number(),);
        }
        let JobAssignmentBuffer {
            jobs,
            list_assignments,
        } = total_assignment_buffer;
        //info!("There is {} components", components.len());
        info!("There is {} gen_jobs", jobs.len(),);
        info!("There is {} job_assignments", list_assignments.len(),);

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

    fn _create_provider_plan(component: &ComponentInfo) -> ProviderPlan {
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
        latest_update: &mut HashMap<TaskKey, Timestamp>,
    ) -> Result<JobAssignmentBuffer, anyhow::Error> {
        let mut assignment_buffer = JobAssignmentBuffer::default();
        let matched_workers = self
            .worker_infos
            .match_workers(provider)
            .await
            .unwrap_or(MatchedWorkers::default());

        for task in self.tasks.iter() {
            if !task.can_apply(provider) {
                continue;
            }
            let task_type = task.get_type();
            // Check if there is task result
            let latest_task_update = latest_update
                .iter_mut()
                .filter(|(key, _)| key.task_type.as_str() == task_type.as_str())
                .map(|(key, value)| (key.task_name.clone(), value))
                .collect::<HashMap<String, &mut Timestamp>>();
            debug!(
                "latest_task_update of task {} for provider {} {}: {:?}",
                &task_type,
                &provider.component_type.to_string(),
                &provider.ip,
                latest_task_update
            );
            let plan_id = format!("{}-{}", JobRole::Regular.to_string(), provider.id);
            // Fix for grant only
            let is_generated;
            {
                let mut generated_provider = self.generated_provider.lock().await;
                info!(
                    "before checking generated_provider: {:?}",
                    generated_provider
                );
                is_generated = generated_provider.contains(&provider.id);
                if !is_generated {
                    generated_provider.insert(provider.id.clone());
                    info!(
                        "after checking generated_provider: {:?}",
                        generated_provider
                    );
                }
            }

            if !is_generated {
                let res = task.apply_with_cache(
                    &plan_id,
                    &provider,
                    JobRole::Regular,
                    &matched_workers,
                    latest_task_update,
                );
                match res {
                    Ok(applied_jobs) => {
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
                                latest_update.insert(task_key, current_time);
                            }
                        }
                        assignment_buffer.append(applied_jobs);
                    }
                    Err(err) => {
                        error!("apply_with_cache error: {:?}", err);
                    }
                }
            }
        }
        Ok(assignment_buffer)
    }
}
