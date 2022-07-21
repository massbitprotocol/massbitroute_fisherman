use crate::models::job_result::{ProviderTask, StoredJobResult};
use crate::models::job_result_cache::JobResultCache;
use crate::persistence::services::{JobResultService, JobService, PlanService};
use crate::report_processors::adapters::Appender;
use crate::report_processors::ReportProcessor;
use crate::service::judgment::{JudgmentsResult, MainJudgment};
use crate::service::report_portal::StoreReport;
use crate::{CONFIG, IS_TEST_MODE, PORTAL_AUTHORIZATION};
use async_trait::async_trait;
use common::job_manage::JobRole;
use common::jobs::{Job, JobResult};
use common::models::PlanEntity;
use common::{ComponentId, JobId, PlanId, DOMAIN};
use log::{debug, info};
use sea_orm::DatabaseConnection;
pub use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct VerificationReportProcessor {
    report_adapters: Vec<Arc<dyn Appender>>,
    plan_service: Arc<PlanService>,
    job_service: Arc<JobService>,
    result_service: Arc<JobResultService>,
    result_cache: Arc<JobResultCache>,
    judgment: MainJudgment,
    active_plans: Mutex<HashMap<ComponentId, PlanEntity>>,
}

impl VerificationReportProcessor {
    pub fn new(
        report_adapters: Vec<Arc<dyn Appender>>,
        plan_service: Arc<PlanService>,
        job_service: Arc<JobService>,
        result_service: Arc<JobResultService>,
        result_cache: Arc<JobResultCache>,
        judgment: MainJudgment,
    ) -> Self {
        VerificationReportProcessor {
            report_adapters,
            plan_service,
            job_service,
            result_service,
            result_cache,
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
    fn can_apply(&self, _report: &JobResult) -> bool {
        true
    }

    async fn process_job(
        &self,
        _report: &JobResult,
        _db_connection: Arc<DatabaseConnection>,
    ) -> Result<StoredJobResult, anyhow::Error> {
        todo!()
    }
    async fn process_jobs(
        &self,
        reports: Vec<JobResult>,
        _db_connection: Arc<DatabaseConnection>,
    ) -> Result<Vec<StoredJobResult>, anyhow::Error> {
        log::debug!("Verification process report jobs: {:?}", &reports);
        for adapter in self.report_adapters.iter() {
            log::info!(
                "With Adapter {} append {} results",
                adapter.get_name(),
                reports.len()
            );
            adapter.append_job_results(&reports).await;
        }
        let stored_results = Vec::<StoredJobResult>::new();
        let mut provider_task_results = HashMap::<ProviderTask, Vec<JobResult>>::new();
        //let mut provider_ids = HashSet::<ComponentId>::new();
        let mut plan_ids = HashSet::<PlanId>::new();
        let mut job_ids = HashSet::<JobId>::new();
        for report in reports {
            //provider_ids.insert(report.provider_id.clone());
            plan_ids.insert(report.plan_id.clone());
            let key = ProviderTask::new(
                report.provider_id.clone(),
                report.provider_type.clone(),
                report.result_detail.get_name(),
                report.job_name.clone(),
            );
            job_ids.insert(report.job_id.clone());
            provider_task_results
                .entry(key)
                .or_insert(Vec::default())
                .push(report);
        }
        if job_ids.is_empty() {
            return Ok(Vec::default());
        }
        //Discard timeout plan
        let active_plans = self.get_active_plans(&plan_ids).await.unwrap_or_default();
        debug!("Active plans {:?}", &active_plans);
        //get Active jobs base on active plan
        let active_plan_ids = active_plans
            .iter()
            .map(|(_, plan)| plan.plan_id.clone())
            .collect::<Vec<PlanId>>();
        let map_plan_jobs = self
            .job_service
            .get_job_by_plan_ids(&active_plan_ids)
            .await
            .map(|jobs| {
                let mut plan_jobs = HashMap::<PlanId, Vec<Job>>::new();
                for job in jobs {
                    plan_jobs
                        .entry(job.plan_id.clone())
                        .or_insert(Vec::<Job>::new())
                        .push(job);
                }
                plan_jobs
            })
            .unwrap_or_default();
        debug!("Active jobs {:?}", &map_plan_jobs);

        //Filter by active plan
        let mut active_provider_task_results = HashMap::<ProviderTask, Vec<JobResult>>::new();
        for (key, results) in provider_task_results {
            //Filter by active plan
            let active_results = results
                .into_iter()
                .filter(|result| {
                    //Keep result with active plan id
                    map_plan_jobs.get(&result.plan_id).is_some()
                })
                .collect::<Vec<JobResult>>();
            if active_results.len() > 0 {
                active_provider_task_results.insert(key, active_results);
            }
        }
        debug!("Active task results {:?}", &active_provider_task_results);
        for (key, results) in active_provider_task_results.iter() {
            log::debug!("Process results {:?} for task {:?}", results, key);
            for adapter in self.report_adapters.iter() {
                adapter.append_job_results(results).await;
            }
        }
        for (provider_task, job_results) in active_provider_task_results {
            //After filter this unwrap is safe;
            let plan_entity = active_plans.get(&provider_task.provider_id).unwrap();
            if let Some(plan_jobs) = map_plan_jobs.get(&plan_entity.plan_id) {
                self.judg_provider_results(provider_task, plan_entity, job_results, plan_jobs)
                    .await;
            } else {
                debug!("Missing jobs of plan {}", &plan_entity.plan_id);
            };
        }

        Ok(stored_results)
    }
}

impl VerificationReportProcessor {
    //Get active plan and remove expired plan
    pub async fn get_active_plans(
        &self,
        plan_ids: &HashSet<PlanId>,
    ) -> Result<HashMap<ComponentId, PlanEntity>, anyhow::Error> {
        self.plan_service
            .get_active_plan_by_ids(&Some(JobRole::Verification), &plan_ids)
            .await
            .map(|plans| {
                plans
                    .into_iter()
                    .map(|plan| (plan.provider_id.clone(), plan))
                    .collect::<HashMap<ComponentId, PlanEntity>>()
            })
    }

    pub async fn load_missing_plans(
        &self,
        providers: &HashSet<ComponentId>,
    ) -> Result<HashMap<ComponentId, PlanEntity>, anyhow::Error> {
        self.plan_service
            .get_active_plan_by_components(&Some(JobRole::Verification), &providers)
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
        plan_jobs: &Vec<Job>,
    ) {
        let plan_results = self
            .judgment
            .apply_for_verify(&provider_task, plan, &results, plan_jobs)
            .await
            .unwrap_or_default();
        debug!(
            "Plan result {:?} found for provider {:?} with plan {:?} and job_results {:?}",
            &plan_results, &provider_task, &plan.plan_id, &results
        );
        self.result_cache
            .update_plan_results(plan, &plan_results, plan_jobs)
            .await;
        //Handle plan result
        let mut final_result = JudgmentsResult::Pass;
        for (_job_id, plan_result) in plan_results {
            if final_result == JudgmentsResult::Pass {
                final_result = plan_result;
            }
            if final_result != JudgmentsResult::Pass {
                break;
            }
        }
        self.report_judgment_result(&provider_task, plan, final_result)
            .await;
    }
    pub async fn report_judgment_result(
        &self,
        provider_task: &ProviderTask,
        plan: &PlanEntity,
        judg_result: JudgmentsResult,
    ) {
        match judg_result {
            JudgmentsResult::Failed | JudgmentsResult::Pass | JudgmentsResult::Error => {
                let mut report = StoreReport::build(
                    &"Scheduler".to_string(),
                    &JobRole::Verification,
                    &*PORTAL_AUTHORIZATION,
                    &DOMAIN,
                );
                report.set_report_data_short(
                    JudgmentsResult::Pass == judg_result,
                    &provider_task.provider_id,
                    &provider_task.provider_type,
                );
                debug!("Send plan report to portal:{:?}", report);
                if !*IS_TEST_MODE {
                    let res = report.send_data().await;
                    info!("Send report to portal res: {:?}", res);
                } else {
                    let result = json!({
                        "provider_id":provider_task.provider_id,
                        "plan_id":plan.plan_id,
                        "result":judg_result
                    });
                    let res = report.write_data(result);
                    info!("Write report to file res: {:?}", res);
                }
            }
            _ => {}
        }
    }
}
