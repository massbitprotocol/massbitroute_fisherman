use crate::models::tasks::TaskApplicant;
use common::component::{ComponentInfo, Zone};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
#[derive(Debug, Default)]
pub struct ProviderStorage {
    nodes: Mutex<Vec<ComponentInfo>>,
    gateways: Mutex<Vec<ComponentInfo>>,
    verification_nodes: Mutex<Vec<ComponentInfo>>
    verification_gateways: Mutex<>
}

impl ProviderStorage {
    pub fn add_node(&mut self, node: ComponentInfo) {}
    pub fn add_gateway(&mut self, gateway: ComponentInfo) {}
    pub fn apply_task(&self, task: &Arc<dyn TaskApplicant>) {
        println!("Apply task");
    }
    pub fn add_verify_node(&mut self, node: ComponentInfo) {

        println!("Add node to verification queue");
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
