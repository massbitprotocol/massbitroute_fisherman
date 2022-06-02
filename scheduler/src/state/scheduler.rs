use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerInfoStorage;
use crate::seaorm::workers;
use common::component::ComponentInfo;
use common::worker::WorkerInfo;
use sea_orm::ActiveModelTrait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Default)]
pub struct SchedulerState {
    connection: Arc<DatabaseConnection>,
    worker_pool: Arc<Mutex<WorkerInfoStorage>>,
    providers: Arc<Mutex<ProviderStorage>>,
}

impl SchedulerState {
    pub fn new(
        connection: Arc<DatabaseConnection>,
        worker_pool: Arc<Mutex<WorkerInfoStorage>>,
        providers: Arc<Mutex<ProviderStorage>>,
    ) -> SchedulerState {
        SchedulerState {
            connection,
            worker_pool,
            providers,
        }
    }
}

impl SchedulerState {
    pub async fn register_worker(&self, worker_info: WorkerInfo) {
        //Save worker to db
        let worker = workers::ActiveModel::from(&worker_info);
        let saved_worker = worker.insert(self.connection.as_ref()).await;
        self.worker_pool.lock().await.add_worker(worker_info);
        println!("{:?}", &saved_worker);
        //Add worker to ProviderStorage
    }
    pub async fn verify_node(&mut self, node_info: ComponentInfo) {
        log::debug!("Push node {:?} to verification queue", &node_info);
        self.providers.lock().await.add_verify_node(node_info).await;
    }
}
