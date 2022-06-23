use crate::models::job_result_cache::JobResultCache;
use crate::persistence::services::{JobResultService, JobService, PlanService};
use crate::service::judgment::main_judg::MainJudgment;
use crate::service::judgment::{get_report_judgments, JudgmentsResult, PingJudgment, ReportCheck};
use crate::service::report_portal::StoreReport;
use crate::state::ProcessorState;
use crate::{CONFIG, PORTAL_AUTHORIZATION};
use common::job_manage::{JobResult, JobResultDetail, JobRole};
use common::jobs::Job;
use common::util::get_current_time;
use common::workers::WorkerInfo;
use common::DOMAIN;
use log::{debug, info};
use sea_orm::sea_query::IdenList;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::default::Default;
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
        job_result_details: Vec<JobResultDetail>,
        state: Arc<Mutex<ProcessorState>>,
    ) -> Result<impl Reply, Rejection> {
        info!("Handle report from worker {:?}", &job_result_details);
        if job_result_details.len() > 0 {
            let plan_ids = Vec::from_iter(
                job_result_details
                    .iter()
                    .map(|res| res.get_plan_id())
                    .collect::<HashSet<String>>(),
            );
            //Store results to persistence storage: csv file, sql db, monitor system v.v...
            let job_results = job_result_details
                .into_iter()
                .map(|jrd| JobResult::new(jrd, get_current_time()))
                .collect();
            state.lock().await.process_results(&job_results).await;

            if let (Ok(plans), Ok(all_jobs)) = (
                self.plan_service.get_plan_by_ids(&plan_ids).await,
                self.job_service.get_job_by_plan_ids(&plan_ids).await,
            ) {
                info!(
                    "get_job_by_plan_ids {} plan_service: {:?}, {} all_jobs: {:?}",
                    plans.len(),
                    plans,
                    all_jobs.len(),
                    all_jobs
                );
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
                        let component_type = match jobs.first() {
                            None => Default::default(),
                            Some(job) => job.component_type.clone(),
                        };

                        for job in jobs {
                            let job_result = self
                                .judgment
                                .apply(plan, job)
                                .await
                                .unwrap_or(JudgmentsResult::Failed);
                            info!(
                                "job_result :{:?}, plan: {:?},job: {:?}",
                                job_result, plan, job
                            );
                            match job_result {
                                JudgmentsResult::Pass => {}
                                JudgmentsResult::Failed | JudgmentsResult::Error => {
                                    plan_result = job_result;
                                    break;
                                }
                                JudgmentsResult::Unfinished => {
                                    plan_result = JudgmentsResult::Unfinished;
                                    break;
                                }
                            }
                        }
                        info!("Plan_result :{:?}", plan_result);
                        match plan_result {
                            JudgmentsResult::Pass
                            | JudgmentsResult::Failed
                            | JudgmentsResult::Error => {
                                let plan_phase =
                                    JobRole::from_str(&*plan.phase).unwrap_or_default();
                                //Todo: move StoreReport to processor Service member
                                // call portal for report result
                                if JobRole::Verification == plan_phase
                                    || plan_result == JudgmentsResult::Failed
                                    || plan_result == JudgmentsResult::Error
                                {
                                    let mut report = StoreReport::build(
                                        &"Scheduler".to_string(),
                                        &plan_phase,
                                        &*PORTAL_AUTHORIZATION,
                                        &DOMAIN,
                                    );

                                    let is_data_correct = plan_result.is_pass();
                                    report.set_report_data_short(
                                        is_data_correct,
                                        &plan.provider_id,
                                        &component_type,
                                    );
                                    debug!("Send plan report to portal:{:?}", report);
                                    if !CONFIG.is_test_mode {
                                        let res = report.send_data(&plan_phase).await;
                                        info!("Send report to portal res: {:?}", res);
                                    } else {
                                        let res = report.write_data();
                                        info!("Write report to file res: {:?}", res);
                                    }
                                }
                            }
                            JudgmentsResult::Unfinished => {}
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
    result_cache: Arc<Mutex<JobResultCache>>,
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
    pub fn with_result_cache(mut self, result_cache: Arc<Mutex<JobResultCache>>) -> Self {
        self.result_cache = result_cache;
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
