use crate::models::job_result::{ProviderTask, StoredJobResult};
use crate::models::job_result_cache::{JobResultCache, PlanTaskResultKey};
use crate::persistence::services::{JobResultService, JobService, PlanService};
use crate::report_processors::adapters::Appender;
use crate::report_processors::ReportProcessor;
use crate::service::judgment::{JudgmentsResult, MainJudgment};
use crate::service::report_portal::{ReportFailedReasons, ReportRecord, StoreReport};
use crate::{IS_VERIFY_REPORT, PORTAL_AUTHORIZATION};
use async_trait::async_trait;

use common::job_manage::JobRole;
use common::jobs::{Job, JobResult};
use common::models::PlanEntity;
use common::util::get_datetime_utc_7;
use common::{ComponentId, PlanId, DOMAIN};
use log::{debug, info, trace};
use sea_orm::DatabaseConnection;
pub use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct VerificationReportProcessor {
    report_adapters: Vec<Arc<dyn Appender>>,
    plan_service: Arc<PlanService>,
    job_service: Arc<JobService>,
    _result_service: Arc<JobResultService>,
    result_cache: Arc<JobResultCache>,
    judgment: MainJudgment,
    _active_plans: Mutex<HashMap<ComponentId, PlanEntity>>,
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
            _result_service: result_service,
            result_cache,
            judgment,
            _active_plans: Default::default(),
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
    ) -> Result<(), anyhow::Error> {
        // Separate the results by ProviderTask
        let mut provider_task_results = HashMap::<ProviderTask, Vec<JobResult>>::new();
        let mut plan_ids = HashSet::<PlanId>::new();

        // Add report to provider_task_results and plan_ids
        for report in reports {
            plan_ids.insert(report.plan_id.clone());
            let key = ProviderTask::new(
                report.provider_id.clone(),
                report.provider_type.clone(),
                report.result_detail.get_name(),
                report.job_name.clone(),
            );
            provider_task_results
                .entry(key)
                .or_insert_with(Vec::default)
                .push(report);
        }
        if plan_ids.is_empty() {
            return Ok(());
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
        for (provider_task, results) in provider_task_results {
            //Filter by active plan
            let active_results = results
                .into_iter()
                .filter(|result| {
                    //Keep result with active plan id
                    map_plan_jobs.get(&result.plan_id).is_some()
                })
                .collect::<Vec<JobResult>>();
            if !active_results.is_empty() {
                active_provider_task_results.insert(provider_task, active_results);
            }
        }
        debug!("Active task results {:?}", &active_provider_task_results);

        // Get active report for storage
        let mut active_reports = vec![];
        for job_results in active_provider_task_results.values() {
            active_reports.extend_from_slice(job_results);
        }
        // Write db
        debug!("Verify process report active jobs: {:?}", &active_reports);

        for adapter in self.report_adapters.iter() {
            log::info!(
                "With Adapter {} append {} results",
                adapter.get_name(),
                active_reports.len()
            );
            adapter
                .append_job_results(&active_reports)
                .await
                .expect("Cannot append job results");
        }

        for (provider_task, active_results) in active_provider_task_results {
            if let Some(plan_entity) = active_plans.get(&provider_task.provider_id) {
                if let Some(plan_jobs) = map_plan_jobs.get(&plan_entity.plan_id) {
                    self.judg_provider_results(
                        provider_task,
                        plan_entity,
                        active_results,
                        plan_jobs,
                    )
                    .await;
                } else {
                    debug!("Missing jobs of plan {}", &plan_entity.plan_id);
                };
            }
        }

        Ok(())
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

        // Check if the plan already conclude
        let map = self
            .result_cache
            .get_plan_judge_result(&plan.provider_id, &plan.plan_id)
            .await;
        trace!("get_plan_judge_result map {:?}", map);
        let last_result = Self::combine_results(&map, plan_jobs);
        if last_result.is_concluded() {
            info!(
                "Plan {:?} provider {:?}, already conclude: {:?}!",
                plan.plan_id, plan.provider_id, last_result
            );
            return;
        }

        // Else continue process
        self.result_cache
            .update_plan_results(plan, &plan_results, plan_jobs)
            .await;

        //Handle plan result
        let mut final_result = JudgmentsResult::Pass;

        let mut reasons = ReportFailedReasons::new(vec![]);
        for (job_id, plan_result) in plan_results {
            match plan_result {
                JudgmentsResult::Pass => continue,
                JudgmentsResult::Failed(sub_reasons) => {
                    reasons.extend(sub_reasons.clone().into_inner());
                }
                JudgmentsResult::Unfinished => {
                    final_result = JudgmentsResult::Unfinished;
                }
            }
        }
        if !reasons.is_empty() {
            final_result = JudgmentsResult::Failed(reasons);
        }

        self.report_judgment_result(&provider_task, plan, final_result)
            .await;
    }

    fn combine_results(
        results: &HashMap<PlanTaskResultKey, JudgmentsResult>,
        plan_jobs: &Vec<Job>,
    ) -> JudgmentsResult {
        if results.is_empty() {
            return JudgmentsResult::Unfinished;
        }

        // Check if all job report
        for job in plan_jobs {
            if !results
                .iter()
                .any(|(key, _)| key.task_type == job.job_type && key.task_name == job.job_name)
            {
                return JudgmentsResult::Unfinished;
            }
        }
        //Handle plan result
        let mut final_jug = JudgmentsResult::Pass;
        let mut reasons = ReportFailedReasons::new(vec![]);
        for plan_result in results.values() {
            match plan_result {
                JudgmentsResult::Pass => continue,
                JudgmentsResult::Failed(sub_reasons) => {
                    reasons.extend(sub_reasons.clone().into_inner())
                }
                JudgmentsResult::Unfinished => {
                    final_jug = JudgmentsResult::Unfinished;
                }
            }
        }
        if reasons.is_empty() {
            final_jug
        } else {
            JudgmentsResult::Failed(reasons)
        }
    }

    pub async fn report_judgment_result(
        &self,
        provider_task: &ProviderTask,
        plan: &PlanEntity,
        judge_result: JudgmentsResult,
    ) {
        if judge_result.is_concluded() {
            let mut report = StoreReport::build(
                &"Scheduler".to_string(),
                &JobRole::Verification,
                &*PORTAL_AUTHORIZATION,
                &DOMAIN,
            );
            report.set_report_data_short(
                &judge_result,
                &provider_task.provider_id,
                &provider_task.provider_type,
            );
            info!(
                "*** Send verify report to portal with message {:?}: {:?}",
                judge_result, report
            );
            if *IS_VERIFY_REPORT {
                let res = report.send_data().await;
                info!("Send report to portal res: {:?}", res);
            } else {
                let report_record = ReportRecord::new(
                    get_datetime_utc_7(),
                    provider_task.provider_id.clone(),
                    plan.plan_id.clone(),
                    judge_result,
                );
                let res = report.write_data(report_record);
                info!("*** Write verify report to file res: {:?}", res);
            }
        }
    }
}
