use crate::models::job_result;
use crate::models::job_result::ProviderTask;
use crate::persistence::services::job_result_service::JobResultService;
use crate::persistence::services::PlanService;
use crate::service::judgment::{get_report_judgments, JudgmentsResult, ReportCheck};
use crate::service::report_portal::StoreReport;
use crate::{CONFIG, CONFIG_DIR, JUDGMENT_PERIOD, PORTAL_AUTHORIZATION};
use anyhow::Error;
use common::job_manage::JobRole;
use common::jobs::{Job, JobResult, JobResultWithJob};
use common::models::plan_entity::PlanStatus;
use common::models::PlanEntity;
use common::{ComponentId, JobId, PlanId, DOMAIN};
use log::{debug, error, info};
use migration::IndexType::Hash;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct MainJudgment {
    result_service: Arc<JobResultService>,
    judgments: Vec<Arc<dyn ReportCheck>>,
    judgment_result_cache: LatestJudgmentCache,
}
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash, Default)]
pub struct JudgmentKey {
    pub plan_id: PlanId,
    pub job_id: JobId,
}

impl JudgmentKey {
    pub fn new(plan_id: PlanId, job_id: JobId) -> JudgmentKey {
        Self { plan_id, job_id }
    }
}
#[derive(Debug, Default)]
pub struct LatestJudgmentCache {
    values: Mutex<HashMap<JudgmentKey, JudgmentsResult>>,
}

impl LatestJudgmentCache {
    pub fn insert_value(&self, plan_id: PlanId, job_id: JobId, judg_result: JudgmentsResult) {
        let key = JudgmentKey::new(plan_id, job_id);
        let mut map = self.values.lock().unwrap();
        map.insert(key, judg_result);
    }
    pub fn get_value(&self, key: &JudgmentKey) -> Option<JudgmentsResult> {
        let values = self.values.lock().unwrap();
        values.get(key).map(|r| r.clone())
    }
}

impl MainJudgment {
    pub fn new(result_service: Arc<JobResultService>) -> Self {
        let judgments = get_report_judgments(CONFIG_DIR.as_str(), result_service.clone());
        MainJudgment {
            result_service,
            judgments,
            judgment_result_cache: Default::default(),
        }
    }
    pub fn put_judment_result(&self, plan: &PlanEntity, job_id: JobId, result: JudgmentsResult) {
        self.judgment_result_cache
            .insert_value(plan.plan_id.clone(), job_id.clone(), result);
    }
    pub fn get_latest_judgment(
        &self,
        plan: &PlanEntity,
        job_id: &JobId,
    ) -> Option<JudgmentsResult> {
        let key = JudgmentKey::new(plan.plan_id.clone(), job_id.clone());
        self.judgment_result_cache
            .get_value(&key)
            .map(|r| r.clone())
    }
    /*
     * for each plan, check result of all judgment,
     * For judgment wich can apply for input provider task, do judgment base on input result,
     * For other judgments get result by plan
     */
    pub async fn apply_for_verify(
        &self,
        provider_task: &ProviderTask,
        plan: &PlanEntity,
        results: &Vec<JobResult>,
        plan_jobs: &Vec<Job>,
    ) -> Result<HashMap<JobId, JudgmentsResult>, anyhow::Error> {
        //Judg received results
        debug!(
            "Make judgment for {} job results of plan {} with provider {:?} and {} jobs",
            results.len(),
            &plan.plan_id,
            provider_task,
            plan_jobs.len()
        );
        let mut total_result = HashMap::<JobId, JudgmentsResult>::new();
        let mut currentjob_result = JudgmentsResult::Unfinished;
        let job_id = results
            .get(0)
            .map(|job_result| job_result.job_id.clone())
            .unwrap_or_default();
        for judgment in self.judgments.iter() {
            if judgment.can_apply_for_result(provider_task) {
                currentjob_result = judgment
                    .apply_for_results(provider_task, &results)
                    .await
                    .unwrap_or(JudgmentsResult::Failed);
                info!(
                    "Verify judgment result {:?} for provider {:?} with results {:?}",
                    &currentjob_result, provider_task, results
                );
                total_result.insert(job_id.clone(), currentjob_result.clone());
            };
        }

        //Put judgment result to cache
        self.put_judment_result(plan, job_id, currentjob_result.clone());
        //Input plan_result as result of current job
        for job in plan_jobs {
            match self.get_latest_judgment(plan, &job.job_id) {
                Some(latest_result) => {
                    debug!(
                        "Latest result of plan {:?}, job {:?} is {:?}",
                        &plan.plan_id, &job.job_id, &latest_result
                    );
                    total_result.insert(job.job_id.clone(), latest_result);
                }
                None => {
                    debug!(
                        "Result not found for plan {:?} and job {:?}",
                        &plan.plan_id, &job.job_id
                    );
                    total_result.insert(job.job_id.clone(), JudgmentsResult::Unfinished);
                }
            }
        }
        //If all task pass then plan result is Pass
        Ok(total_result)
    }
    /*
    pub async fn apply(
        &self,
        plan: &PlanEntity,
        job: &Job,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        let mut final_result = JudgmentsResult::Unfinished;
        for jud in self.judgments.iter() {
            let can_apply = jud.can_apply(job);
            info!(
                "Apply plan: {:?}, job: {:?}, jud: {:?}, can_apply: {:?}",
                plan, job, jud, can_apply
            );
            if !can_apply {
                continue;
            }

            match jud.apply(plan, job).await {
                Ok(res) => {
                    match res {
                        JudgmentsResult::Pass => final_result = res,
                        JudgmentsResult::Error | JudgmentsResult::Failed => {
                            //Job result is bad, stop check with other judgment and send report
                            final_result = res;
                            break;
                        }
                        JudgmentsResult::Unfinished => {
                            //Job is not finished yet. Stop check with other judgment
                            //Todo: check when timeout
                            final_result = res
                        }
                    }
                }
                Err(e) => {
                    error!("Apply Judgments error: {}", e);
                    final_result = JudgmentsResult::Error;
                    break;
                }
            };
        }
        info!(
            "Apply plan: {:?}, job: {:?}, final_result: {:?}",
            plan, job, final_result
        );
        Ok(final_result)
    }
     */
    /*
     * Use for regular judgment
     */
    pub async fn apply_for_regular(
        &self,
        provider_task: &ProviderTask,
        results: &Vec<JobResult>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        //Separate jobs by job name
        let provider_type = results
            .get(0)
            .map(|res| res.provider_type.clone())
            .unwrap_or_default();

        for judgment in self.judgments.iter() {
            if !judgment.can_apply_for_result(&provider_task) {
                continue;
            }
            let judg_result = judgment
                .apply_for_results(provider_task, results)
                .await
                .unwrap_or(JudgmentsResult::Failed);
            info!(
                "Regular judgment result {:?} on task {} for provider {:?}",
                &judg_result, provider_task.task_name, provider_task.provider_id
            );
            match judg_result {
                JudgmentsResult::Failed | JudgmentsResult::Error => {
                    let mut report = StoreReport::build(
                        &"Scheduler".to_string(),
                        &JobRole::Regular,
                        &*PORTAL_AUTHORIZATION,
                        &DOMAIN,
                    );
                    report.set_report_data_short(false, &provider_task.provider_id, &provider_type);
                    debug!("Send plan report to portal:{:?}", report);
                    if !CONFIG.is_test_mode {
                        let res = report.send_data().await;
                        info!("Send report to portal res: {:?}", res);
                    } else {
                        let result = json!({"provider_task":provider_task,"result":judg_result});
                        let res = report.write_data(result);
                        info!("Write report to file res: {:?}", res);
                    }
                }
                _ => {}
            }
        }

        Ok(JudgmentsResult::Unfinished)
    }
}
