use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::ReportCheck;
use anyhow::Error;
use async_trait::async_trait;
use common::models::PlanEntity;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct BenchmarkJudgment {
    result_service: Arc<JobResultService>,
}

impl BenchmarkJudgment {
    pub fn new(result_service: Arc<JobResultService>) -> Self {
        BenchmarkJudgment { result_service }
    }
}

#[async_trait]
impl ReportCheck for BenchmarkJudgment {
    fn can_apply(&self) -> bool {
        true
    }

    async fn apply(&self, plan: &PlanEntity) -> Result<u32, Error> {
        Ok(0)
    }
}
