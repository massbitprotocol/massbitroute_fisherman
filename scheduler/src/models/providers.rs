use common::component::ComponentInfo;
use std::sync::Mutex;

#[derive(Default)]
pub struct ProviderStorage {
    nodes: Mutex<Vec<ComponentInfo>>,
    gateways: Mutex<Vec<ComponentInfo>>,
}

impl ProviderStorage {
    pub fn add_node(&mut self, node: ComponentInfo) {}
    pub fn add_gateway(&mut self, gateway: ComponentInfo) {}
}
