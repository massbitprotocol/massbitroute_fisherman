use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerInfoStorage;
use crate::persistence::services::plan_service::PlanService;
use crate::persistence::services::WorkerService;
use crate::{CONFIG, REPORT_CALLBACK};
use common::component::ComponentInfo;
use common::job_manage::JobRole;
use common::models::PlanEntity;
use common::util::get_current_time;
use common::workers::{WorkerInfo, WorkerRegisterResult};

use sea_orm::DatabaseConnection;

use log::error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct SchedulerState {
    connection: Arc<DatabaseConnection>,
    plan_service: Arc<PlanService>,
    worker_service: Arc<WorkerService>,
    worker_pool: Arc<WorkerInfoStorage>,
    providers: Arc<ProviderStorage>,
}

impl SchedulerState {
    pub fn new(
        connection: Arc<DatabaseConnection>,
        plan_service: Arc<PlanService>,
        worker_service: Arc<WorkerService>,
        worker_pool: Arc<WorkerInfoStorage>,
        providers: Arc<ProviderStorage>,
    ) -> SchedulerState {
        SchedulerState {
            connection,
            plan_service,
            worker_service,
            worker_pool,
            providers,
        }
    }
}

impl SchedulerState {
    pub async fn register_worker(
        &self,
        worker_info: WorkerInfo,
    ) -> Result<WorkerRegisterResult, anyhow::Error> {
        println!("{:?}", &worker_info);
        let report_callback = REPORT_CALLBACK.as_str().to_string();
        //Save worker to db
        if let Some(WorkerInfo { worker_id, .. }) = self
            .worker_service
            .clone()
            .get_stored_worker(&worker_info.worker_id)
            .await
        {
            self.worker_pool.add_worker(worker_info).await;
            Ok(WorkerRegisterResult {
                worker_id,
                report_callback,
            })
        } else {
            let worker_id = worker_info.worker_id.clone();
            let res = self.worker_service.clone().store_worker(&worker_info).await;
            if res.is_err() {
                error!("store_worker error: {:?}", res);
            }
            self.worker_pool.add_worker(worker_info).await;
            Ok(WorkerRegisterResult {
                worker_id,
                report_callback,
            })
        }

        //Add worker to ProviderStorage
    }
    pub async fn verify_node(&self, node_info: ComponentInfo) -> Result<PlanEntity, anyhow::Error> {
        log::debug!("Push node {:?} to verification queue", &node_info);
        //Create a scheduler in db
        let current_time = get_current_time();
        let expiry_time = current_time + CONFIG.plan_expiry_time * 1000;
        let plan = PlanEntity::new(
            node_info.id.clone(),
            current_time,
            expiry_time,
            JobRole::Verification.to_string(),
        );
        let store_res = self.plan_service.store_plan(&plan).await;
        if let Ok(model) = store_res {
            //Generate verification job base on stored plan
            self.providers.add_verify_node(model, node_info).await;
        }

        Ok(plan)
    }
}
