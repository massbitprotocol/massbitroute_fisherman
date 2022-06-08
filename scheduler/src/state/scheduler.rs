use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerInfoStorage;
use crate::persistence::seaorm::workers;
use crate::persistence::services::WorkerService;
use crate::REPORT_CALLBACK;
use common::component::ComponentInfo;
use common::worker::{WorkerInfo, WorkerRegisterResult};
use sea_orm::ActiveModelTrait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct SchedulerState {
    connection: Arc<DatabaseConnection>,
    worker_service: Arc<WorkerService>,
    worker_pool: Arc<Mutex<WorkerInfoStorage>>,
    providers: Arc<Mutex<ProviderStorage>>,
}

impl SchedulerState {
    pub fn new(
        connection: Arc<DatabaseConnection>,
        worker_service: Arc<WorkerService>,
        worker_pool: Arc<Mutex<WorkerInfoStorage>>,
        providers: Arc<Mutex<ProviderStorage>>,
    ) -> SchedulerState {
        SchedulerState {
            connection,
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
            Ok(WorkerRegisterResult {
                worker_id,
                report_callback,
            })
        } else {
            let worker_id = worker_info.worker_id.clone();
            self.worker_service.clone().store_worker(&worker_info).await;
            self.worker_pool.lock().await.add_worker(worker_info);
            Ok(WorkerRegisterResult {
                worker_id,
                report_callback,
            })
        }

        //Add worker to ProviderStorage
    }
    pub async fn verify_node(&mut self, node_info: ComponentInfo) {
        log::debug!("Push node {:?} to verification queue", &node_info);
        self.providers.lock().await.add_verify_node(node_info).await;
    }
}
