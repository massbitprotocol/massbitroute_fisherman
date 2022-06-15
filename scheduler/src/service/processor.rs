use crate::persistence::services::{JobResultService, JobService, PlanService};
use crate::service::judgment::main_judg::MainJudgment;
use crate::service::judgment::{get_report_judgments, JudgmentsResult, PingJudgment, ReportCheck};
use crate::service::report_portal::StoreReport;
use crate::state::ProcessorState;
use crate::PORTAL_AUTHORIZATION;
use common::job_manage::{Job, JobResult, JobRole};
use common::worker::WorkerInfo;
use common::DOMAIN;
use sea_orm::sea_query::IdenList;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Buf, Rejection, Reply};

#[derive(Default)]
pub struct ProcessorService {
    plan_service: Arc<PlanService>,
    job_service: Arc<JobService>,
    result_service: Arc<JobResultService>,
    judgment: MainJudgment,
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
            //Store results to persistence storage: csv file, sql db, monitor system v.v...
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
                        let mut plan_result = JudgmentsResult::Pass;
                        for job in jobs {
                            let job_result = self
                                .judgment
                                .apply(plan, job)
                                .await
                                .unwrap_or(JudgmentsResult::Failed);
                            match job_result {
                                JudgmentsResult::Pass => {}
                                JudgmentsResult::Failed => {
                                    plan_result = JudgmentsResult::Failed;
                                    break;
                                }
                                JudgmentsResult::Unfinished => {
                                    plan_result = JudgmentsResult::Unfinished;
                                    break;
                                }
                            }
                        }

                        if plan_result == JudgmentsResult::Failed {
                            //Todo: call portal for report bad result
                            let mut report = StoreReport::build(
                                &plan.provider_id,
                                JobRole::from_str(&*plan.phase).unwrap_or_default(),
                                &*PORTAL_AUTHORIZATION,
                                &DOMAIN,
                            );

                            report.set_report_data_short(
                                false,
                                &"".to_string(),
                                &Default::default(),
                            )
                        } else if plan_result == JudgmentsResult::Pass {
                            //Todo: call portal for report good result
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
        let judgment = MainJudgment::new(self.result_service.clone());
        ProcessorService {
            plan_service: self.plan_service,
            job_service: self.job_service,
            result_service: self.result_service,
            judgment,
        }
    }
}
