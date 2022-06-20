use crate::models::component::ProviderPlan;
use crate::models::jobs::AssignmentBuffer;
use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerInfoStorage;
use crate::persistence::services::{JobService, PlanService};
use crate::persistence::PlanModel;
use crate::tasks::generator::{get_tasks, TaskApplicant};
use crate::{CONFIG, CONFIG_DIR, JOB_VERIFICATION_GENERATOR_PERIOD};
use anyhow::{anyhow, Error};
use common::component::{ComponentInfo, Zone};
use common::job_manage::JobRole;
use common::jobs::{Job, JobAssignment};
use common::models::plan_entity::PlanStatus;
use common::models::PlanEntity;
use common::tasks::LoadConfig;
use common::util::get_current_time;
use common::workers::{MatchedWorkers, Worker, WorkerInfo};
use common::{task_spawn, ComponentId, WorkerId};
use futures_util::future::join;
use log::{debug, info, warn};
use minifier::js::Keyword::Default;
use sea_orm::sea_query::IndexType::Hash;
use sea_orm::{DatabaseConnection, DbErr, TransactionTrait};
use serde::{Deserialize, Serialize};
use slog::log;
use std::collections::HashMap;
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
    providers: Arc<Mutex<ProviderStorage>>,
    worker_infos: Arc<Mutex<WorkerInfoStorage>>,
    tasks: Vec<Arc<dyn TaskApplicant>>,
    job_service: Arc<JobService>,
    assignments: Arc<Mutex<AssignmentBuffer>>,
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
     * For verification job (long job) find next available worker
     */
    /*
     * For verification job (long job) find next available worker
     */
    pub async fn generate_verification_jobs(&mut self) {
        let mut gen_jobs = HashMap::<Zone, Vec<Job>>::new();
        let mut job_assignments = Vec::default();
        let nodes = self
            .providers
            .lock()
            .await
            .pop_nodes_for_verifications()
            .await;
        if nodes.len() > 0 {
            log::debug!("Generate verification jobs for {} nodes", nodes.len());
            for provider_plan in nodes.iter() {
                let matched_workers = self
                    .worker_infos
                    .lock()
                    .await
                    .match_workers(&provider_plan.provider)
                    .unwrap_or(MatchedWorkers::default());
                log::debug!(
                    "Found workers {:?} for plan {:?}",
                    &matched_workers,
                    provider_plan
                );
                for task in self.tasks.iter() {
                    if !task.can_apply(&provider_plan.provider) {
                        continue;
                    }
                    let mut applied_jobs = task
                        .apply(&provider_plan.plan.plan_id, &provider_plan.provider)
                        .unwrap_or(vec![]);
                    if applied_jobs.len() > 0 {
                        log::debug!(
                            "Create {:?} jobs for node {:?}",
                            applied_jobs.len(),
                            &provider_plan.provider
                        );
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
                }
                log::debug!(
                    "Create {:?} assignments for provider plan {:?}",
                    job_assignments.len(),
                    &provider_plan
                );
            }
        }
        //Generate jobs for gateways
        let gateways = self
            .providers
            .lock()
            .await
            .pop_gateways_for_verifications()
            .await;
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
                    let mut applied_jobs = task
                        .apply(&provider_plan.plan.plan_id, &provider_plan.provider)
                        .unwrap_or(vec![]);

                    if applied_jobs.len() > 0 {
                        log::debug!(
                            "Create {:?} jobs for gateway {:?}",
                            applied_jobs.len(),
                            &provider_plan.provider
                        );
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
                }
            }
        }
        log::debug!("Job assignment length {}", job_assignments.len());
        if job_assignments.len() > 0 {
            self.assignments
                .lock()
                .await
                .add_assignments(job_assignments);
        }
        if gen_jobs.len() > 0 {
            //Store jobs to db
            self.store_jobs(&gen_jobs).await;
        }
    }

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

    pub async fn generate_regular_jobs(&mut self) -> Result<(), Error> {
        // Get components in system
        let mut components = self.providers.lock().await.clone_nodes_list().await;
        components.append(&mut self.providers.lock().await.clone_gateways_list().await);
        info!("components list: {:?}", components);
        // Read Schedule to check what schedule already init and generated
        let plans = self
            .plan_service
            .get_plan_models(&Some(JobRole::Regular), &vec![])
            .await?;
        // Convert Plan vec to hashmap
        let mut plans: HashMap<ComponentId, PlanModel> = plans
            .into_iter()
            .map(|plan| (plan.provider_id.clone(), plan))
            .collect();

        let mut gen_jobs = HashMap::<Zone, Vec<Job>>::new();
        let mut job_assignments = Vec::default();
        let mut new_plans: Vec<PlanEntity> = Vec::new();
        for component in components.iter() {
            // Init all the components that not init or generated
            if !plans.contains_key(&component.id) {
                let new_plan = PlanEntity::new(
                    component.id.clone(),
                    get_current_time(),
                    JobRole::Regular.to_string(),
                );
                plans.insert(component.id.clone(), PlanModel::from(&new_plan));
                new_plans.push(new_plan.clone());
            }
        }
        if !new_plans.is_empty() {
            debug!("Store new_plans {}: {:?}", new_plans.len(), new_plans);
            self.plan_service.store_plans(&new_plans).await;
        }
        for component in components {
            let matched_workers = self
                .worker_infos
                .lock()
                .await
                .match_workers(&component)
                .unwrap_or(MatchedWorkers::default());
            log::debug!(
                "Found workers {:?} for component {:?}",
                &matched_workers,
                &component
            );
            let plan = plans.get(&*component.id).unwrap();
            let provider_plan = ProviderPlan {
                provider: component,
                plan: plan.clone(),
            };

            // Generate job
            for task in self.tasks.iter() {
                if !task.can_apply(&provider_plan.provider) {
                    continue;
                }
                let jobs = task.apply(&plan.plan_id, &provider_plan.provider);
                if let Ok(jobs) = jobs {
                    if jobs.is_empty() {
                        continue;
                    }
                    if let Ok(mut assignments) = task.assign_jobs(
                        &provider_plan.plan,
                        &provider_plan.provider,
                        &jobs,
                        &matched_workers,
                    ) {
                        job_assignments.append(&mut assignments);
                    }
                    let entry = gen_jobs
                        .entry(provider_plan.provider.zone.clone())
                        .or_insert(Vec::new());
                    entry.extend(jobs);
                }
            }
            // Change plant status to
            //plan.status = PlanStatus::Generated;
        }

        //warn!("assign gen_jobs {}: {:?}", gen_jobs.len(), gen_jobs);
        if !gen_jobs.is_empty() {
            // Store job
            match self.store_jobs(&gen_jobs).await {
                Ok(_) => {}
                Err(_) => {}
            }
        }
        Ok(())
    }
}

impl JobGenerator {
    pub fn new(
        db_conn: Arc<DatabaseConnection>,
        plan_service: Arc<PlanService>,
        providers: Arc<Mutex<ProviderStorage>>,
        worker_infos: Arc<Mutex<WorkerInfoStorage>>,
        job_service: Arc<JobService>,
        assignments: Arc<Mutex<AssignmentBuffer>>,
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
        };

        let regular = DetailJobGenerator {
            db_conn,
            plan_service,
            providers,
            worker_infos,
            tasks: get_tasks(config_dir, JobRole::Regular, &task_config.regular),
            job_service,
            assignments,
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

        // Run Regular task
        let regular_task = task_spawn::spawn(async move {
            info!("Run Regular task");
            loop {
                info!("generate_regular_jobs result 1");
                let res = regular.generate_regular_jobs().await;
                info!("generate_regular_jobs result 2: {:?} ", res);
                sleep(Duration::from_secs(CONFIG.regular_plan_generate_interval)).await;
            }
        });

        join(verification_task, regular_task).await;
    }
}
