use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::ReportCheck;
use anyhow::Error;
use async_trait::async_trait;
use common::models::PlanEntity;
use minifier::js::Keyword::Default;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct LatestBlockJudgment {
    result_service: Arc<JobResultService>,
}

impl LatestBlockJudgment {
    pub fn new(result_service: Arc<JobResultService>) -> Self {
        LatestBlockJudgment { result_service }
    }
}

#[async_trait]
impl ReportCheck for LatestBlockJudgment {
    fn can_apply(&self) -> bool {
        false
    }

    async fn apply(&self, plan: &PlanEntity) -> Result<u32, Error> {
        Ok(0)
    }
}
