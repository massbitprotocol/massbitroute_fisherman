use crate::models::component::ProviderPlan;
use crate::persistence::PlanModel;
use common::component::{ComponentInfo, ComponentType};
use common::util::get_current_time;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct ProviderStorage {
    nodes: Mutex<Vec<ComponentInfo>>,
    gateways: Mutex<Vec<ComponentInfo>>,
    verification_nodes: Mutex<Vec<Arc<ProviderPlan>>>,
    verification_gateways: Mutex<Vec<Arc<ProviderPlan>>>,
}

impl ProviderStorage {
    pub async fn update_components_list(
        &self,
        component_type: ComponentType,
        components: Vec<ComponentInfo>,
    ) {
        log::debug!(
            "Update {:?} list for regular schedule: {:?}",
            component_type,
            components
        );
        match component_type {
            ComponentType::Node => {
                let mut lock = self.nodes.lock().await;
                *lock = components;
            }
            ComponentType::Gateway => {
                let mut lock = self.gateways.lock().await;
                *lock = components;
            }
        }
    }

    pub async fn add_verify_node(&self, plan_model: PlanModel, node: ComponentInfo) {
        match node.component_type {
            ComponentType::Node => {
                log::debug!("Add node to verification queue");
                self.verification_nodes
                    .lock()
                    .await
                    .push(Arc::new(ProviderPlan::new(node, plan_model)));
            }
            ComponentType::Gateway => {
                log::debug!("Add gateway to verification queue");
                self.verification_gateways
                    .lock()
                    .await
                    .push(Arc::new(ProviderPlan::new(node, plan_model)));
            }
        }
    }
    pub async fn get_expired_verification_plans(&self) -> Vec<Arc<ProviderPlan>> {
        let current_time = get_current_time();
        let mut nodes = self.verification_nodes.lock().await;
        let mut active_nodes = Vec::new();
        let mut expired_nodes = Vec::new();
        let mut renew_plans = Vec::new();
        for plan in nodes.iter() {
            if plan.plan.expiry_time <= current_time {
                let renew_plan = Arc::new(plan.renew());
                renew_plans.push(renew_plan.clone());
                expired_nodes.push(plan.clone());
                active_nodes.push(renew_plan);
            } else {
                active_nodes.push(plan.clone());
            }
        }
        nodes.clear();
        nodes.append(&mut active_nodes);
        let mut gateways = self.verification_gateways.lock().await;
        let mut active_gateways = Vec::new();
        for plan in gateways.iter() {
            if plan.plan.expiry_time <= current_time {
                let renew_plan = Arc::new(plan.renew());
                renew_plans.push(renew_plan.clone());
                expired_nodes.push(plan.clone());
                active_gateways.push(renew_plan);
            } else {
                active_gateways.push(plan.clone());
            }
        }
        gateways.clear();
        gateways.append(&mut active_gateways);
        expired_nodes
    }
    pub async fn pop_components_for_verifications(&self) -> Vec<Arc<ProviderPlan>> {
        let mut res = Vec::new();
        let _current_time = get_current_time();
        let mut nodes = self.verification_nodes.lock().await;
        res.append(&mut nodes);
        let mut gateways = self.verification_gateways.lock().await;
        res.append(&mut gateways);
        res
    }
    /*
    pub async fn pop_nodes_for_verifications(&self) -> Vec<ProviderPlan> {
        let mut res = Vec::new();
        let mut nodes = self.verification_nodes.lock().await;
        res.append(&mut nodes);
        res
    }
    pub async fn pop_gateways_for_verifications(&self) -> Vec<ProviderPlan> {
        let mut res = Vec::new();
        let mut nodes = self.verification_gateways.lock().await;
        res.append(&mut nodes);
        res
    }
    */
    pub async fn clone_nodes_list(&self) -> Vec<ComponentInfo> {
        let nodes = self.nodes.lock().await;
        nodes.clone()
    }
    pub async fn clone_gateways_list(&self) -> Vec<ComponentInfo> {
        let gateways = self.gateways.lock().await;
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
