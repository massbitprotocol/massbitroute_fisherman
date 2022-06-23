use crate::persistence::services::job_result_service::JobResultService;
use crate::persistence::services::PlanService;
use crate::service::judgment::{get_report_judgments, JudgmentsResult, ReportCheck};
use crate::service::report_portal::StoreReport;
use crate::{CONFIG, CONFIG_DIR, JUDGMENT_PERIOD, PORTAL_AUTHORIZATION};
use anyhow::Error;
use common::job_manage::JobRole;
use common::jobs::{Job, JobResult};
use common::models::plan_entity::PlanStatus;
use common::models::PlanEntity;
use common::DOMAIN;
use log::{debug, error, info};
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Default)]
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
    pub async fn apply(
        &self,
        plan: &PlanEntity,
        job: &Job,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        let mut final_result = JudgmentsResult::Unfinished;
        for jud in self.judgments.iter() {
            info!(
                "Apply plan: {:?}, job: {:?}, jud: {:?}, can_apply: {:?}",
                plan,
                job,
                jud,
                jud.can_apply(job)
            );
            if !jud.can_apply(job) {
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
    pub async fn apply_for_provider(
        &self,
        provider_id: String,
        results: Vec<JobResult>,
    ) -> Result<JudgmentsResult, anyhow::Error> {
        //Separate jobs by job name
        let provider_type = results
            .get(0)
            .map(|res| res.provider_type.clone())
            .unwrap_or_default();
        let mut map_results = HashMap::<String, Vec<JobResult>>::new();
        for result in results {
            let mut results = map_results
                .entry(result.job_name.clone())
                .or_insert(Vec::default());
            results.push(result);
        }
        for (job_name, results) in map_results.iter() {
            for judgment in self.judgments.iter() {
                if !judgment.can_apply_for_result(job_name) {
                    continue;
                }
                let judg_result = judgment
                    .apply_for_results(results)
                    .await
                    .unwrap_or(JudgmentsResult::Failed);
                debug!(
                    "Judgment result {:?} for provider {:?} with results {:?}",
                    &judg_result, &provider_id, results
                );
                match judg_result {
                    JudgmentsResult::Failed | JudgmentsResult::Error => {
                        let mut report = StoreReport::build(
                            &"Scheduler".to_string(),
                            &JobRole::Regular,
                            &*PORTAL_AUTHORIZATION,
                            &DOMAIN,
                        );
                        report.set_report_data_short(false, &provider_id, &provider_type);
                        debug!("Send plan report to portal:{:?}", report);
                        if !CONFIG.is_test_mode {
                            let res = report.send_data(&JobRole::Verification).await;
                            info!("Send report to portal res: {:?}", res);
                        } else {
                            let res = report.write_data();
                            info!("Write report to file res: {:?}", res);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(JudgmentsResult::Unfinished)
    }
}
