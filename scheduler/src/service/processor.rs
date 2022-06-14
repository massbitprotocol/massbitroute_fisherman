use crate::persistence::services::{JobResultService, JobService, PlanService};
use crate::service::judgment::{get_report_judgments, ReportCheck};
use crate::state::ProcessorState;
use common::job_manage::{Job, JobResult};
use common::worker::WorkerInfo;
use sea_orm::sea_query::IdenList;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Buf, Rejection, Reply};

#[derive(Default)]
pub struct ProcessorService {
    plan_service: Arc<PlanService>,
    job_service: Arc<JobService>,
    result_service: Arc<JobResultService>,
    judgments: Vec<Arc<dyn ReportCheck>>,
}

impl ProcessorService {
    pub fn builder() -> ProcessorServiceBuilder {
        ProcessorServiceBuilder::default()
    }
    pub async fn process_report(
        &self,
        job_results: Vec<JobResult>,
        state: Arc<Mutex<ProcessorState>>,
    ) -> Result<impl Reply, Rejection> {
        print!("Handle report from worker {:?}", &job_results);
        if job_results.len() > 0 {
            let plan_ids = Vec::from_iter(
                job_results
                    .iter()
                    .map(|res| res.get_plan_id())
                    .collect::<HashSet<String>>(),
            );
            state.lock().await.process_results(job_results).await;
            if let (Ok(plans), Ok(all_jobs)) = (
                self.plan_service.get_plan_by_ids(&plan_ids).await,
                self.job_service.get_job_by_plan_ids(&plan_ids).await,
            ) {
                let mut map_plan_jobs = HashMap::<String, Vec<Job>>::new();
                for job in all_jobs.into_iter() {
                    if let Some(mut jobs) = map_plan_jobs.get_mut(&job.plan_id) {
                        jobs.push(job);
                    } else {
                        map_plan_jobs.insert(job.plan_id.clone(), vec![job]);
                    }
                }
                for plan in plans.iter() {
                    if let Some(jobs) = map_plan_jobs.get(&plan.plan_id) {
                        for judg in self.judgments.iter() {
                            for job in jobs {
                                if judg.can_apply(job) {
                                    continue;
                                }
                                match judg.apply(plan, job).await {
                                    Ok(res) => {}
                                    Err(_) => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        return Ok(warp::reply::json(&json!({ "Message": "Report received" })));
    }
}
#[derive(Default)]
pub struct ProcessorServiceBuilder {
    plan_service: Arc<PlanService>,
    job_service: Arc<JobService>,
    result_service: Arc<JobResultService>,
}

impl ProcessorServiceBuilder {
    pub fn with_plan_service(mut self, plan_service: Arc<PlanService>) -> Self {
        self.plan_service = plan_service;
        self
    }
    pub fn with_job_service(mut self, job_service: Arc<JobService>) -> Self {
        self.job_service = job_service;
        self
    }
    pub fn with_result_service(mut self, result_service: Arc<JobResultService>) -> Self {
        self.result_service = result_service;
        self
    }
    pub fn build(self) -> ProcessorService {
        let judgments = get_report_judgments(self.result_service.clone());
        ProcessorService {
            plan_service: self.plan_service,
            job_service: self.job_service,
            result_service: self.result_service,
            judgments,
        }
    }
}
