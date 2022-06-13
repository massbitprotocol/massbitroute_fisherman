use crate::models::component::ZoneComponents;
use anyhow::Error;
use common::component::{ComponentInfo, ComponentType, Zone};
use common::job_manage::Job;
use common::tasks::generator::TaskApplicant;
use common::ComponentId;
use log::{debug, log};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct ProviderStorage {
    nodes: Mutex<Vec<ComponentInfo>>,
    gateways: Mutex<Vec<ComponentInfo>>,
    verification_nodes: Mutex<HashMap<String, ComponentInfo>>,
    verification_gateways: Mutex<HashMap<String, ComponentInfo>>,
}

impl ProviderStorage {
    pub async fn update_components_list(
        &mut self,
        component_type: ComponentType,
        components: Vec<ComponentInfo>,
    ) {
        match component_type {
            ComponentType::Node => {
                log::debug!("Add node to verification queue");
                let mut lock = self.nodes.lock().await;
                *lock = components;
            }
            ComponentType::Gateway => {
                log::debug!("Add gateway to verification queue");
                let mut lock = self.gateways.lock().await;
                *lock = components;
            }
        }
    }

    pub async fn add_verify_node(&mut self, plan_id: String, node: ComponentInfo) {
        match node.component_type {
            ComponentType::Node => {
                log::debug!("Add node to verification queue");
                self.verification_nodes.lock().await.insert(plan_id, node);
            }
            ComponentType::Gateway => {
                log::debug!("Add gateway to verification queue");
                self.verification_gateways
                    .lock()
                    .await
                    .insert(plan_id, node);
            }
        }
    }
    pub async fn pop_nodes_for_verifications(&mut self) -> HashMap<String, ComponentInfo> {
        let mut res = HashMap::new();
        let mut nodes = self.verification_nodes.lock().await;
        for (plan_id, node) in nodes.iter() {
            res.insert(plan_id.clone(), node.clone());
        }
        nodes.clear();
        res
    }
    pub async fn pop_gateways_for_verifications(&mut self) -> HashMap<String, ComponentInfo> {
        let mut res = HashMap::new();
        let mut nodes = self.verification_gateways.lock().await;
        for (plan_id, node) in nodes.iter() {
            res.insert(plan_id.clone(), node.clone());
        }
        nodes.clear();
        res
    }

    pub async fn clone_nodes_list(&mut self) -> Vec<ComponentInfo> {
        let mut nodes = self.nodes.lock().await;
        nodes.clone()
    }
    pub async fn clone_gateways_list(&mut self) -> Vec<ComponentInfo> {
        let mut gateways = self.gateways.lock().await;
        gateways.clone()
    }

    // pub async fn generate_regular_jobs(
    //     &mut self,
    //     task: Arc<dyn TaskApplicant>,
    // ) -> Result<Vec<Job>, anyhow::Error> {
    //     //For nodes
    //     let mut jobs_list = Vec::default();
    //     let nodes = self.nodes.lock().await;
    //     for node in nodes.iter() {
    //         if task.can_apply(node) {
    //             let mut jobs = task.apply(node)?;
    //             jobs_list.append(&mut jobs);
    //         }
    //     }
    //     //For gateways
    //     let gateways = self.gateways.lock().await;
    //     for gw in gateways.iter() {
    //         if task.can_apply(gw) {
    //             let mut jobs = task.apply(gw)?;
    //             jobs_list.append(&mut jobs);
    //         }
    //     }
    //     debug!("Regular jobs_list: {:?}", jobs_list);
    //     Ok(jobs_list)
    // }

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
