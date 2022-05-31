use crate::models::providers::ProviderStorage;
use common::component::ComponentInfo;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Default)]
pub struct SchedulerState {
    providers: Arc<Mutex<ProviderStorage>>,
}

impl SchedulerState {
    pub fn new(providers: Arc<Mutex<ProviderStorage>>) -> SchedulerState {
        SchedulerState { providers }
    }
}

impl SchedulerState {
    pub async fn verify_node(&mut self, node_info: ComponentInfo) {
        log::debug!("Push node {:?} to verification queue", &node_info);
        self.providers.lock().await.add_verify_node(node_info).await;
    }
}
