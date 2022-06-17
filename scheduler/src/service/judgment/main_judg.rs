use crate::persistence::services::job_result_service::JobResultService;
use crate::persistence::services::PlanService;
use crate::service::judgment::{get_report_judgments, ReportCheck};
use crate::{CONFIG, CONFIG_DIR, JUDGMENT_PERIOD};
use anyhow::Error;
use common::jobs::Job;
use common::models::plan_entity::PlanStatus;
use common::models::PlanEntity;
use log::error;
use sea_orm::DatabaseConnection;
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
    pub async fn apply(&self, plan: &PlanEntity, job: &Job) -> Result<i32, anyhow::Error> {
        let mut final_result = 0;
        for judg in self.judgments.iter() {
            if judg.can_apply(job) {
                continue;
            }
            match judg.apply(plan, job).await {
                Ok(res) => {
                    if res < 0 {
                        //Job result is bad, stop check with other judgment and send report
                        final_result = res;
                        break;
                    } else if res == 0 {
                        //Job is not finished yet. Stop check with other judgment
                        //Todo: check when timeout
                        final_result = res
                    } else {
                        final_result = final_result + res;
                    }
                }
                Err(_) => {}
            }
        }
        Ok(final_result)
    }
}
