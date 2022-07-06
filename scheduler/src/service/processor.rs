use crate::models::job_result_cache::JobResultCache;
use crate::persistence::services::{JobResultService, JobService, PlanService};
use crate::service::judgment::main_judg::MainJudgment;
use crate::service::judgment::{get_report_judgments, JudgmentsResult, PingJudgment, ReportCheck};
use crate::service::report_portal::StoreReport;
use crate::state::ProcessorState;
use crate::{CONFIG, PORTAL_AUTHORIZATION};
use anyhow::Error;
use common::job_manage::{JobResultDetail, JobRole};
use common::jobs::{Job, JobResult};
use common::util::get_current_time;
use common::workers::WorkerInfo;
use common::DOMAIN;
use log::{debug, info};
use sea_orm::sea_query::IdenList;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Buf, Rejection, Reply};

#[derive(Default)]
pub struct ProcessorService {
    plan_service: Arc<PlanService>,
    job_service: Arc<JobService>,
    result_service: Arc<JobResultService>,
}

impl ProcessorService {
    pub fn builder() -> ProcessorServiceBuilder {
        ProcessorServiceBuilder::default()
    }
    pub async fn process_report(
        &self,
        results: Vec<JobResult>,
        state: Arc<ProcessorState>,
    ) -> Result<(), Error> {
        if results.len() > 0 {
            let worker_id = results.get(0).unwrap().worker_id.clone();
            info!(
                "Handle report from worker {:?} with {} details",
                &worker_id,
                results.len()
            );
            //Store results to persistence storage: csv file, sql db, monitor system v.v...
            return state.process_results(results).await;
        }
        return Ok(());
    }
}
#[derive(Default)]
pub struct ProcessorServiceBuilder {
    plan_service: Arc<PlanService>,
    job_service: Arc<JobService>,
    result_service: Arc<JobResultService>,
    result_cache: Arc<Mutex<JobResultCache>>,
}

impl ProcessorServiceBuilder {
    pub fn with_plan_service(mut self, plan_service: Arc<PlanService>) -> Self {
        self.plan_service = plan_service;
        self
    }
    pub fn with_job_service(mut self, job_service: Arc<JobService>) -> Self {
        self.job_service = job_service;
        self
    }
    pub fn with_result_service(mut self, result_service: Arc<JobResultService>) -> Self {
        self.result_service = result_service;
        self
    }
    pub fn with_result_cache(mut self, result_cache: Arc<Mutex<JobResultCache>>) -> Self {
        self.result_cache = result_cache;
        self
    }

    pub fn build(self) -> ProcessorService {
        ProcessorService {
            plan_service: self.plan_service,
            job_service: self.job_service,
            result_service: self.result_service,
        }
    }
}
