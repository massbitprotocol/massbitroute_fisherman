use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::ReportCheck;
use anyhow::Error;
use async_trait::async_trait;
use common::models::PlanEntity;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct PingJudgment {
    result_service: Arc<JobResultService>,
}

impl PingJudgment {
    pub fn new(result_service: Arc<JobResultService>) -> Self {
        PingJudgment { result_service }
    }
}

#[async_trait]
impl ReportCheck for PingJudgment {
    fn can_apply(&self) -> bool {
        todo!()
    }

    async fn apply(&self, plan: &PlanEntity) -> Result<u32, Error> {
        if let Ok(responses) = self
            .result_service
            .get_result_pings(plan.plan_id.as_str())
            .await
        {
            println!("{:?}", &responses);
        }
        Ok(0)
    }
}
