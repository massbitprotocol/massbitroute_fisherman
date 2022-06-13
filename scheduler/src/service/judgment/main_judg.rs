use crate::persistence::services::job_result_service::JobResultService;
use crate::persistence::services::PlanService;
use crate::service::judgment::{get_report_judgments, ReportCheck};
use crate::JUDGMENT_PERIOD;
use anyhow::Error;
use common::models::plan_entity::PlanStatus;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Judgment {
    plan_service: Arc<PlanService>,
    result_service: Arc<JobResultService>,
    judgments: Vec<Arc<dyn ReportCheck>>,
}

impl Judgment {
    pub fn new(plan_service: Arc<PlanService>, connection: Arc<DatabaseConnection>) -> Self {
        let result_service = Arc::new(JobResultService::new(connection));
        let judgments = get_report_judgments(result_service.clone());
        Judgment {
            plan_service,
            result_service,
            judgments,
        }
    }
    pub async fn run(&mut self) {
        loop {
            let plan_service = self.plan_service.clone();
            log::debug!("Get plan from dbs");
            if let Ok(plans) = plan_service
                .get_plans(&None, &vec![PlanStatus::Generated])
                .await
            {
                let now = Instant::now();
                for plan in plans.iter() {
                    for judg in self.judgments.iter() {
                        match judg.apply(plan).await {
                            Ok(res) => {}
                            Err(_) => {}
                        }
                    }
                }
                if now.elapsed().as_secs() < JUDGMENT_PERIOD {
                    sleep(Duration::from_secs(
                        JUDGMENT_PERIOD - now.elapsed().as_secs(),
                    ));
                }
            }
        }
    }
}
