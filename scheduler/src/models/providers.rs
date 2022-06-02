use crate::models::component::ZoneComponents;
use crate::models::tasks::TaskApplicant;
use anyhow::Error;
use common::component::{ComponentInfo, ComponentType, Zone};
use common::job_manage::Job;
use common::ComponentId;
use log::log;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub async fn add_node(&mut self, node: ComponentInfo) {
        match node.component_type {
            ComponentType::Node => {
                log::debug!("Add node to verification queue");
                self.nodes.lock().await.push(node);
            }
            ComponentType::Gateway => {
                log::debug!("Add gateway to verification queue");
                self.gateways.lock().await.push(node);
            }
        }
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
    pub async fn pop_nodes_for_verifications(&mut self) -> Vec<ComponentInfo> {
        let mut res = Vec::new();
        let mut nodes = self.verification_nodes.lock().await;
        res.append(&mut nodes);
        res
    }
    pub async fn pop_gateways_for_verifications(&mut self) -> Vec<ComponentInfo> {
        let mut res = Vec::new();
        let mut nodes = self.verification_gateways.lock().await;
        res.append(&mut nodes);
        res
    }
    pub async fn generate_regular_jobs(
        &mut self,
        task: Arc<dyn TaskApplicant>,
    ) -> Result<Vec<Job>, anyhow::Error> {
        //For nodes
        let mut result = Vec::default();
        let mut nodes = self.nodes.lock().await;
        while let Some(node) = nodes.pop() {
            let mut jobs = task.apply(&node)?;
            result.append(&mut jobs);
        }
        nodes.clear();
        //For gateways
        let mut gateways = self.gateways.lock().await;
        while let Some(node) = gateways.pop() {
            let mut jobs = task.apply(&node)?;
            result.append(&mut jobs);
        }
        Ok(result)
    }
    /*
     * Count nodes in verification queue to check available worker
     */
    /*
     * count verifying nodes by zone
     */
    // pub async fn count_verifying_nodes(&self) -> HashMap<Zone, List<&ComponentInfo>> {
    //     let mut map = HashMap::new();
    //     self.nodes.lock().await.iter().for_each(|n1| {
    //         let counter = map.get(&n1.zone).map_or(0_u16, |v| *v);
    //         map.insert(n1.zone.clone(), counter + 1);
    //     });
    //     map
    // }
}
