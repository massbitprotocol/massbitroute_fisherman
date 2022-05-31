use crate::models::tasks::TaskApplicant;
use common::component::{ComponentInfo, ComponentType, Zone};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct ProviderStorage {
    nodes: Mutex<Vec<ComponentInfo>>,
    gateways: Mutex<Vec<ComponentInfo>>,
    verification_nodes: Mutex<Vec<ComponentInfo>>,
    verification_gateways: Mutex<Vec<ComponentInfo>>,
}

impl ProviderStorage {
    pub fn add_node(&mut self, node: ComponentInfo) {}
    pub fn add_gateway(&mut self, gateway: ComponentInfo) {}
    pub fn apply_task(&self, task: &Arc<dyn TaskApplicant>) {
        println!("Apply task");
    }
    pub async fn add_verify_node(&mut self, node: ComponentInfo) {
        match node.component_type {
            ComponentType::Node => {
                log::debug!("Add node to verification queue");
                self.verification_nodes.lock().await.push(node);
            }
            ComponentType::Gateway => {
                log::debug!("Add gateway to verification queue");
                self.verification_gateways.lock().await.push(node);
            }
        }
    }
    pub fn get_zone_nodes(&self, zone: &Zone) -> Result<Vec<ComponentInfo>, anyhow::Error> {
        let mut vec = Vec::<ComponentInfo>::default();
        Ok(vec)
    }
    pub fn get_zone_gateways(&self, zone: &Zone) -> Result<Vec<ComponentInfo>, anyhow::Error> {
        let mut vec = Vec::<ComponentInfo>::default();
        Ok(vec)
    }
}
