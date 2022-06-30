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
use common::{JobId, DOMAIN};
use log::{debug, error, info};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Clone, Default)]
pub struct MainJudgment {
    result_service: Arc<JobResultService>,
    judgments: Vec<Arc<dyn ReportCheck>>,
}

impl MainJudgment {
    pub fn new(result_service: Arc<JobResultService>) -> Self {
        let judgments = get_report_judgments(CONFIG_DIR.as_str(), result_service.clone());
        MainJudgment {
            result_service,
            judgments,
        }
    }
    /*
     * for each plan, check result of all judgment,
     * For judgment wich can apply for input provider task, do judgment base on input result,
     * For other judgments get result by plan
     */
    pub async fn judg_provider_results(
        &self,
        provider_task: &ProviderTask,
        plan: &PlanEntity,
        results: &Vec<JobResult>,
        jobs: &HashMap<JobId, Job>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        let mut plan_result = JudgmentsResult::Pass;

        for judgment in self.judgments.iter() {
            let judg_result = if !judgment.can_apply_for_result(&provider_task) {
                //Get judgment result from cache o db
                judgment.get_latest_judgment(provider_task, plan).await?
            } else {
                let judg_result = judgment
                    .apply_for_results(provider_task, &results)
                    .await
                    .unwrap_or(JudgmentsResult::Failed);
                info!(
                    "Judgment result {:?} for provider {:?} with results {:?}",
                    &judg_result, provider_task, results
                );

                judg_result
            };
            //Store first un Pass judgment result
            match judg_result {
                JudgmentsResult::Pass => {}
                JudgmentsResult::Failed | JudgmentsResult::Unfinished | JudgmentsResult::Error => {
                    //Failed if single task failed
                    return Ok(judg_result);
                }
            }
        }
        //If all task pass then plan result is Pass
        Ok(plan_result)
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
    pub async fn apply_for_provider(
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
                "Judgment result {:?} for provider {:?} with results {:?}",
                &judg_result, provider_task, results
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
                        let res = report.write_data();
                        info!("Write report to file res: {:?}", res);
                    }
                }
                _ => {}
            }
        }

        Ok(JudgmentsResult::Unfinished)
    }
}
