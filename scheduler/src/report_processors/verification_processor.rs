use crate::models::job_result::StoredJobResult;
use crate::persistence::services::{JobResultService, JobService, PlanService};
use crate::report_processors::adapters::{get_report_adapters, Appender};
use crate::report_processors::ReportProcessor;
use crate::service::judgment::{JudgmentsResult, MainJudgment};
use crate::service::report_portal::StoreReport;
use crate::{CONFIG, PORTAL_AUTHORIZATION};
use async_trait::async_trait;
use common::job_manage::{JobBenchmarkResult, JobResultDetail, JobRole};
use common::jobs::{Job, JobResult};
use common::tasks::eth::JobLatestBlockResult;
use common::tasks::http_request::{JobHttpRequest, JobHttpResult};
use common::{PlanId, DOMAIN};
use log::{debug, info};
use sea_orm::DatabaseConnection;
pub use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
#[derive(Clone, Default)]
pub struct VerificationReportProcessor {
    report_adapters: Vec<Arc<dyn Appender>>,
    plan_service: Arc<PlanService>,
    job_service: Arc<JobService>,
    result_service: Arc<JobResultService>,
    judgment: MainJudgment,
}

impl VerificationReportProcessor {
    pub fn new(
        report_adapters: Vec<Arc<dyn Appender>>,
        plan_service: Arc<PlanService>,
        job_service: Arc<JobService>,
        result_service: Arc<JobResultService>,
        judgment: MainJudgment,
    ) -> Self {
        VerificationReportProcessor {
            report_adapters,
            plan_service,
            job_service,
            result_service,
            judgment,
        }
    }
    pub fn add_adapter(&mut self, adapter: Arc<dyn Appender>) {
        self.report_adapters.push(adapter);
    }
}
#[async_trait]
impl ReportProcessor for VerificationReportProcessor {
    fn can_apply(&self, report: &JobResult) -> bool {
        true
    }

    async fn process_job(
        &self,
        report: &JobResult,
        db_connection: Arc<DatabaseConnection>,
    ) -> Result<StoredJobResult, anyhow::Error> {
        todo!()
    }

    async fn process_jobs(
        &self,
        reports: Vec<JobResult>,
        db_connection: Arc<DatabaseConnection>,
    ) -> Result<Vec<StoredJobResult>, anyhow::Error> {
        log::debug!("Verificatoin report process jobs");
        let mut ping_results = Vec::new();
        let mut benchmark_results: Vec<JobBenchmarkResult> = Vec::new();
        let mut latest_block_results: Vec<JobLatestBlockResult> = Vec::new();
        let mut stored_results = Vec::<StoredJobResult>::new();
        let mut http_request_results: Vec<JobResult> = Vec::new();
        let mut plan_ids = HashSet::<PlanId>::new();
        for report in reports {
            match report.result_detail {
                JobResultDetail::Ping(result) => {
                    plan_ids.insert(result.job.plan_id.clone());
                    ping_results.push(result);
                    //println!("{:?}", &ping_result);
                }
                JobResultDetail::LatestBlock(result) => {
                    plan_ids.insert(result.job.plan_id.clone());
                    latest_block_results.push(result);
                }
                JobResultDetail::Benchmark(result) => {
                    plan_ids.insert(result.job.plan_id.clone());
                    benchmark_results.push(result);
                }
                JobResultDetail::HttpRequest(ref result) => {
                    plan_ids.insert(result.job.plan_id.clone());
                    http_request_results.push(report);
                }
                _ => {}
            }
        }
        //update provider map base on ping result
        // Todo: Add response time for each Job result
        for adapter in self.report_adapters.iter() {
            if ping_results.len() > 0 {
                adapter.append_ping_results(&ping_results).await;
            }
            if latest_block_results.len() > 0 {
                adapter
                    .append_latest_block_results(&latest_block_results)
                    .await;
            }
            if benchmark_results.len() > 0 {
                adapter.append_benchmark_results(&benchmark_results).await;
            }
            if http_request_results.len() > 0 {
                adapter
                    .append_http_request_results(&http_request_results)
                    .await;
            }
        }
        self.judg_results(Vec::from_iter(plan_ids)).await;
        Ok(stored_results)
    }
}

impl VerificationReportProcessor {
    pub async fn judg_results(&self, plan_ids: Vec<PlanId>) {
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
                let mut jobs = map_plan_jobs.entry(job.plan_id.clone()).or_insert(vec![]);
                jobs.push(job);
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
                            let plan_phase = JobRole::from_str(&*plan.phase).unwrap_or_default();
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
}
