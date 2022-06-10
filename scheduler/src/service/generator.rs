use crate::models::jobs::{AssignmentBuffer, JobAssignment};
use crate::models::providers::ProviderStorage;
use crate::models::workers::{Worker, WorkerInfoStorage};
use crate::persistence::services::{JobService, PlanService};
use crate::{CONFIG_DIR, JOB_VERIFICATION_GENERATOR_PERIOD};
use anyhow::Error;
use common::component::{ComponentInfo, Zone};
use common::job_manage::{Job, JobRole};
use common::models::plan_entity::PlanStatus;
use common::models::PlanEntity;
use common::tasks::generator::{get_tasks, TaskApplicant};
use common::util::get_current_time;
use common::worker::WorkerInfo;
use common::{task_spawn, ComponentId, WorkerId};
use futures_util::future::join;
use sea_orm::sea_query::IndexType::Hash;
use sea_orm::{DatabaseConnection, DbErr, TransactionTrait};
use std::collections::HashMap;
use std::mem::take;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::task;

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

impl DetailJobGenerator {
    /*
     * For verification job (long job) find next available worker
     */
    pub async fn generate_verification_jobs(&mut self) {
        let mut gen_jobs = HashMap::<Zone, Vec<Job>>::new();
        let map_plans = self
            .plan_service
            .get_verification_plans()
            .await
            .and_then(|vec| {
                let mut map = HashMap::<String, Vec<PlanEntity>>::new();
                for entity in vec {
                    if let Some(mut vec) = map.get_mut(&entity.provider_id) {
                        vec.push(entity);
                    } else {
                        map.insert(entity.plan_id.clone(), vec![entity]);
                    }
                }
                Ok(map)
            });
        //Generate jobs for nodes
        if let Ok(plans) = map_plans.as_ref() {
            let nodes = self
                .providers
                .lock()
                .await
                .pop_nodes_for_verifications()
                .await;

            for node in nodes.iter() {
                if let Some(plan) = plans.get(&node.id) {
                    if plan.len() == 0 {
                        return;
                    }
                    for task in self.tasks.iter() {
                        if task.can_apply(node) {
                            match task.apply(plan.get(0)?, node) {
                                Ok(mut jobs) => {
                                    if jobs.len() > 0 {
                                        log::debug!(
                                            "Create {:?} jobs for node {:?}",
                                            jobs.len(),
                                            &node
                                        );
                                        if let Some(current_jobs) = gen_jobs.get_mut(&node.zone) {
                                            current_jobs.append(&mut jobs);
                                        } else {
                                            gen_jobs.insert(node.zone.clone(), jobs);
                                        }
                                    }
                                }
                                Err(err) => {
                                    log::error!("Error: {:?}", &err);
                                }
                            }
                        }
                    }
                }
            }
            //Generate jobs for gateways
            let gateways = self
                .providers
                .lock()
                .await
                .pop_gateways_for_verifications()
                .await;
            for gw in gateways.iter() {
                if let Some(plan) = plans.get(&gw.id) {
                    for task in self.tasks.iter() {
                        if task.can_apply(gw) {
                            match task.apply(plan, gw) {
                                Ok(mut jobs) => {
                                    if jobs.len() > 0 {
                                        log::debug!(
                                            "Create {:?} jobs for gateway {:?}",
                                            jobs.len(),
                                            &gw
                                        );
                                        if let Some(current_jobs) = gen_jobs.get_mut(&gw.zone) {
                                            current_jobs.append(&mut jobs);
                                        } else {
                                            gen_jobs.insert(gw.zone.clone(), jobs);
                                        }
                                    }
                                }
                                Err(err) => {
                                    log::error!("Error: {:?}", &err);
                                }
                            }
                        }
                    }
                }
            }
        }
        //Distribute job to workers
        self.assign_jobs(&gen_jobs).await;
        //Store jobs to db
        self.store_jobs(&gen_jobs).await;
    }

    pub async fn store_jobs(&self, map_jobs: &HashMap<Zone, Vec<Job>>) {
        let mut gen_plans = Vec::default();
        let tnx = self.db_conn.begin().await?;
        for (zone, jobs) in map_jobs.iter() {
            jobs.iter().for_each(|job| {
                gen_plans.push(job.plan_id.clone());
            });
            self.job_service.save_jobs(jobs).await;
        }
        self.plan_service.update_plans_as_generated(gen_plans);
        match tnx.commit().await {
            Ok(_) => {
                log::debug!("Transaction commited successful")
            }
            Err(err) => {
                log::debug!("Transaction commited with error {:?}", &err);
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
                        let worker: &Arc<WorkerInfo> = workers.get(wind).unwrap();
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

        // Read Schedule to check what schedule already init and generated
        let plans = self
            .plan_service
            .get_plans(&JobRole::Regular, &vec![])
            .await?;
        // Convert Plan vec to hashmap
        let mut plans: HashMap<ComponentId, PlanEntity> = plans
            .into_iter()
            .map(|plan| (plan.provider_id.clone(), plan))
            .collect();

        for component in components {
            // Init all the components that not init or generated
            let mut plan = plans.entry(component.id.clone()).or_insert(PlanEntity::new(
                component.id.clone(),
                get_current_time(),
                JobRole::Regular.to_string(),
            ));
            if !(plan.status == PlanStatus::Init) {
                continue;
            }
            // Generate job
            let mut gen_jobs = HashMap::<Zone, Vec<Job>>::new();
            for task in self.tasks.iter() {
                if task.can_apply(&component) {
                    let jobs = task.apply(plan, &component);
                    if let Ok(jobs) = jobs {
                        gen_jobs.entry(component.zone.clone()).or_insert(jobs);
                        self.assign_jobs(&gen_jobs).await;
                        self.store_jobs(&gen_jobs).await;
                    }
                }
            }
            // Change plant status to
            plan.status = PlanStatus::Generated;
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
        let verification = DetailJobGenerator {
            db_conn: db_conn.clone(),
            plan_service: plan_service.clone(),
            providers: providers.clone(),
            worker_infos: worker_infos.clone(),
            tasks: get_tasks(CONFIG_DIR.as_str(), JobRole::Verification),
            job_service: job_service.clone(),
            assignments: assignments.clone(),
        };

        let regular = DetailJobGenerator {
            db_conn,
            plan_service,
            providers,
            worker_infos,
            tasks: get_tasks(CONFIG_DIR.as_str(), JobRole::Regular),
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
        let verification_task = task::spawn(async move {
            loop {
                let now = Instant::now();
                verification.generate_verification_jobs().await;
                if now.elapsed().as_secs() < JOB_VERIFICATION_GENERATOR_PERIOD {
                    sleep(Duration::from_secs(
                        JOB_VERIFICATION_GENERATOR_PERIOD - now.elapsed().as_secs(),
                    ));
                }
            }
        });

        // Run Regular task
        let regular_task = task::spawn(async move {
            regular.generate_regular_jobs().await;
        });

        join(verification_task, regular_task);
    }

    /*
     * For verification job (long job) find next available worker
     */
    // pub async fn generate_verification_node_jobsV0(&mut self) {
    //     log::debug!("Generator verification node jobs");
    //     /*
    //      * find all worker in current zone and distribute components to found workers
    //      */
    //     let zones = self.providers.lock().await.get_verifying_nodes().await;
    //
    //     for (zone, ids) in zones.iter() {
    //         let mut worker_nodes = HashMap::<WorkerId, Vec<Arc<ComponentInfo>>>::default();
    //         if let Some(workers) = self.workers.lock().await.get_workers(zone) {
    //             if workers.len() > 0 {
    //                 ids.iter().enumerate().for_each(|(ind, item)| {
    //                     let wind = ind % workers.len();
    //                     let worker = workers.get(wind).unwrap();
    //                     if let Some(nodes) = worker_nodes.get_mut(&worker.worker_id) {
    //                         nodes.push(item.clone());
    //                     } else {
    //                         worker_nodes.insert(worker.worker_id.clone(), vec![item.clone()]);
    //                     }
    //                 });
    //             }
    //         }
    //         for (worker, comps) in worker_nodes.into_iter() {
    //             self.generate_verification_node_jobs_for_worker(zone, worker, comps);
    //         }
    //     }
    // }
    // async fn generate_verification_node_jobs_for_worker(
    //     &mut self,
    //     zone: &Zone,
    //     worker_id: WorkerId,
    //     comps: Vec<Arc<ComponentInfo>>,
    // ) {
    //     //println!("{:?} => {}", &worker_id, &comp_ids.join(","));
    //     if let Some(worker) = self.workers.lock().await.get_worker_by_id(zone, &worker_id) {
    //         for comp in comps.iter() {
    //             for task in self.tasks.iter() {
    //                 task.apply(comp.clone());
    //             }
    //         }
    //     }
    //     //Remove component from queue
    // }
}
