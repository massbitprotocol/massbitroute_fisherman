use crate::persistence::services::job_result_service::JobResultService;
use crate::persistence::services::PlanService;
use crate::service::judgment::{get_report_judgments, JudgmentsResult, ReportCheck};
use crate::{CONFIG, CONFIG_DIR, JUDGMENT_PERIOD};
use anyhow::Error;
use common::jobs::Job;
use common::models::plan_entity::PlanStatus;
use common::models::PlanEntity;
use log::{error, info};
use sea_orm::DatabaseConnection;
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
}
