use crate::persistence::services::{JobResultService, PlanService};
use crate::service::judgment::{get_report_judgments, ReportCheck};
use crate::state::ProcessorState;
use common::job_manage::JobResult;
use common::worker::WorkerInfo;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Buf, Rejection, Reply};

#[derive(Default)]
pub struct ProcessorService {
    plan_service: Arc<PlanService>,
    result_service: Arc<JobResultService>,
    judgments: Vec<Arc<dyn ReportCheck>>,
}

impl ProcessorService {
    pub fn builder() -> ProcessorServiceBuilder {
        ProcessorServiceBuilder::default()
    }
    pub async fn process_report(
        &self,
        job_results: Vec<JobResult>,
        state: Arc<Mutex<ProcessorState>>,
    ) -> Result<impl Reply, Rejection> {
        print!("Handle report from worker {:?}", &job_results);
        if job_results.len() > 0 {
            let plan_ids = job_results
                .iter()
                .map(|res| res.get_plan_id())
                .collect::<Vec<String>>();
            state.lock().await.process_results(job_results).await;
            if let Ok(plans) = self.plan_service.get_plan_by_ids(&plan_ids).await {
                for plan in plans.iter() {
                    for judg in self.judgments.iter() {
                        match judg.apply(plan).await {
                            Ok(res) => {}
                            Err(_) => {}
                        }
                    }
                }
            }
        }
        return Ok(warp::reply::json(&json!({ "Message": "Report received" })));
    }
}
#[derive(Default)]
pub struct ProcessorServiceBuilder {
    plan_service: Arc<PlanService>,
    result_service: Arc<JobResultService>,
}

impl ProcessorServiceBuilder {
    pub fn with_plan_service(mut self, plan_service: Arc<PlanService>) -> Self {
        self.plan_service = plan_service;
        self
    }
    pub fn with_result_service(mut self, result_service: Arc<JobResultService>) -> Self {
        self.result_service = result_service;
        self
    }
    pub fn build(self) -> ProcessorService {
        let judgments = get_report_judgments(self.result_service.clone());
        ProcessorService {
            plan_service: self.plan_service,
            result_service: self.result_service,
            judgments,
        }
    }
}
