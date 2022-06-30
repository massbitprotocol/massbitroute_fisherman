use crate::models::job_result::{ProviderTask, StoredJobResult};
use crate::persistence::services::{JobResultService, JobService, PlanService};
use crate::report_processors::adapters::{get_report_adapters, Appender};
use crate::report_processors::ReportProcessor;
use crate::service::judgment::{JudgmentsResult, MainJudgment};
use crate::service::report_portal::StoreReport;
use crate::{CONFIG, PORTAL_AUTHORIZATION};
use async_trait::async_trait;
use common::job_manage::{JobBenchmarkResult, JobResultDetail, JobRole};
use common::jobs::{Job, JobResult, JobResultWithJob};
use common::models::PlanEntity;
use common::tasks::eth::JobLatestBlockResult;
use common::tasks::http_request::{JobHttpRequest, JobHttpResult};
use common::util::get_current_time;
use common::{ComponentId, JobId, PlanId, DOMAIN};
use futures_util::{FutureExt, TryFutureExt};
use log::{debug, error, info};
use sea_orm::DatabaseConnection;
pub use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

// #[derive(Clone, Default)]
// pub struct PlanEntityWithExpiry {
//     pub plan: PlanEntity,
//     pub expiry_time: i64,
// }
//
// impl PlanEntityWithExpiry {
//     pub fn new(plan: PlanEntity, expiry_time: i64) -> PlanEntityWithExpiry {
//         Self { plan, expiry_time }
//     }
// }
#[derive(Default)]
pub struct VerificationReportProcessor {
    report_adapters: Vec<Arc<dyn Appender>>,
    plan_service: Arc<PlanService>,
    job_service: Arc<JobService>,
    result_service: Arc<JobResultService>,
    judgment: MainJudgment,
    active_plans: Mutex<HashMap<ComponentId, PlanEntity>>,
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
            active_plans: Default::default(),
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
        log::debug!("Verification process report jobs");
        let mut stored_results = Vec::<StoredJobResult>::new();
        let mut provider_task_results = HashMap::<ProviderTask, Vec<JobResult>>::new();
        let mut provider_ids = HashSet::<ComponentId>::new();
        let mut job_ids = HashSet::<JobId>::new();
        for report in reports {
            provider_ids.insert(report.provider_id.clone());
            let key = ProviderTask::new(
                report.provider_id.clone(),
                report.provider_type.clone(),
                report.result_detail.get_name(),
                report.job_name.clone(),
            );
            job_ids.insert(report.job_id.clone());
            let mut jobs = provider_task_results.entry(key).or_insert(Vec::default());
            jobs.push(report);
        }
        if job_ids.is_empty() {
            return Ok(Vec::default());
        }
        //Discard timeout plan
        let active_plans = self
            .get_active_plans(&provider_ids)
            .await
            .unwrap_or_default();
        debug!("Active plans {:?}", &active_plans);
        let mut map_jobs = HashMap::<JobId, Job>::new();
        let mut plan_ids = HashSet::<PlanId>::new();
        if let Ok(jobs) = self.job_service.get_job_by_ids(&job_ids).await {
            for job in jobs {
                plan_ids.insert(job.plan_id.clone());
                map_jobs.insert(job.job_id.clone(), job);
            }
        }
        //Filter by active plan
        let mut active_provider_task_results = HashMap::<ProviderTask, Vec<JobResult>>::new();
        for (key, results) in provider_task_results {
            //Filter by active plan
            let active_results = results
                .into_iter()
                .filter(|result| {
                    if let (Some(job), Some(plan)) = (
                        map_jobs.get(&result.job_id),
                        active_plans.get(&result.provider_id),
                    ) {
                        true
                    } else {
                        false
                    }
                })
                .collect::<Vec<JobResult>>();
            if active_results.len() > 0 {
                active_provider_task_results.insert(key, active_results);
            }
        }
        for (key, results) in active_provider_task_results.iter() {
            log::debug!("Process results {:?} for task {:?}", results, key);
            for adapter in self.report_adapters.iter() {
                adapter.append_job_results(key, results).await;
            }
        }
        for (provider_task, job_results) in active_provider_task_results {
            //After filter this unwrap is safe;
            let plan_entity = active_plans.get(&provider_task.provider_id).unwrap();
            self.judg_provider_results(provider_task, plan_entity, job_results, &map_jobs)
                .await;
        }

        Ok(stored_results)
    }
    /*
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
     */
}

impl VerificationReportProcessor {
    //Get active plan and remove expired plan
    pub async fn get_active_plans(
        &self,
        providers: &HashSet<ComponentId>,
    ) -> Result<HashMap<ComponentId, PlanEntity>, anyhow::Error> {
        let current_time = get_current_time();
        let mut result = HashMap::new();
        let mut missing_plan = HashSet::<ComponentId>::new();
        {
            let mut active_plan_cache = self.active_plans.lock().unwrap();
            let mut expired_plan = HashSet::<ComponentId>::new();
            for provider_id in providers {
                if let Some(plan_entity) = active_plan_cache.get(provider_id) {
                    if plan_entity.expiry_time < current_time {
                        expired_plan.insert(plan_entity.plan_id.clone());
                    } else {
                        result.insert(provider_id.clone(), plan_entity.clone());
                    }
                } else {
                    missing_plan.insert(provider_id.clone());
                }
            }
            for expired_id in expired_plan {
                active_plan_cache.remove(&expired_id);
            }
        }
        //Load active plan from database if missing
        if missing_plan.len() > 0 {
            debug!("Load missing plans {:?} from database", &missing_plan);
            if let Ok(plans) = self.load_missing_plans(&missing_plan).await {
                let mut active_plan_cache = self.active_plans.lock().unwrap();
                for (provider_id, plan) in plans {
                    result.insert(provider_id.clone(), plan.clone());
                    active_plan_cache.insert(provider_id, plan);
                }
            }
        }

        Ok(result)
    }
    pub async fn load_missing_plans(
        &self,
        providers: &HashSet<ComponentId>,
    ) -> Result<HashMap<ComponentId, PlanEntity>, anyhow::Error> {
        self.plan_service
            .get_active_plans(&Some(JobRole::Verification), &providers)
            .await
            .map(|plans| {
                plans
                    .into_iter()
                    .map(|plan| (plan.provider_id.clone(), plan))
                    .collect::<HashMap<ComponentId, PlanEntity>>()
            })
    }
    pub async fn judg_provider_results(
        &self,
        provider_task: ProviderTask,
        plan: &PlanEntity,
        results: Vec<JobResult>,
        map_jobs: &HashMap<JobId, Job>,
    ) {
        let plan_result = self
            .judgment
            .judg_provider_results(&provider_task, plan, &results, map_jobs)
            .await
            .unwrap_or(JudgmentsResult::Failed);
        //Handle plan result
        self.report_judgment_result(&provider_task, plan_result)
            .await;
    }
    pub async fn report_judgment_result(
        &self,
        provider_task: &ProviderTask,
        judg_result: JudgmentsResult,
    ) {
        match judg_result {
            JudgmentsResult::Failed | JudgmentsResult::Error => {
                let mut report = StoreReport::build(
                    &"Scheduler".to_string(),
                    &JobRole::Regular,
                    &*PORTAL_AUTHORIZATION,
                    &DOMAIN,
                );
                report.set_report_data_short(
                    false,
                    &provider_task.provider_id,
                    &provider_task.provider_type,
                );
                debug!("Send plan report to portal:{:?}", report);
                if !CONFIG.is_test_mode {
                    let res = report.send_data().await;
                    info!("Send report to portal res: {:?}", res);
                } else {
                    let res = report.write_data();
                    info!("Write report to file res: {:?}", res);
                }
            }
            _ => {}
        }
    }
    /*
    pub async fn judg_plans(&self, plan_ids: Vec<PlanId>) {
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
                                    let res = report.send_data().await;
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
    */
    pub async fn judg_plan_results(&self, plan: PlanEntity) {}
}
