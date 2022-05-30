use common::component::{ComponentInfo, Zone};
use std::sync::Mutex;

#[derive(Default)]
pub struct ProviderStorage {
    nodes: Mutex<Vec<ComponentInfo>>,
    gateways: Mutex<Vec<ComponentInfo>>,
}

impl ProviderStorage {
    pub fn add_node(&mut self, node: ComponentInfo) {}
    pub fn add_gateway(&mut self, gateway: ComponentInfo) {}
    pub fn get_zone_nodes(&self, zone: &Zone) -> Result<Vec<ComponentInfo>, anyhow::Error> {
        let mut vec = Vec::<ComponentInfo>::default();
        Result(vec);
    }
    pub fn get_zone_gateways(&self, zone: &Zone) -> Result<Vec<ComponentInfo>, anyhow::Error> {
        let mut vec = Vec::<ComponentInfo>::default();
        Result(vec);
    }
}
