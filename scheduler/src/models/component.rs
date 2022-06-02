use common::component::{ComponentInfo, Zone};
use common::ComponentId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Index;
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ZoneComponents {
    inner: HashMap<Zone, Vec<Arc<ComponentInfo>>>,
}

impl ZoneComponents {
    pub fn add_node(&mut self, node: ComponentInfo) {
        if let Some(comps) = self.inner.get_mut(&node.zone) {
            comps.push(Arc::new(node));
        } else {
            self.inner.insert(node.zone.clone(), vec![Arc::new(node)]);
        }
    }
    pub fn get_inner_ref(&self) -> &HashMap<Zone, Vec<Arc<ComponentInfo>>> {
        &self.inner
    }
    pub fn get_comp_zones(&self) -> HashMap<Zone, Vec<Arc<ComponentInfo>>> {
        self.inner.clone()
    }
    // pub fn get_comps_by_zone_ids(
    //     &self,
    //     zone: &Zone,
    //     ids: &Vec<ComponentId>,
    // ) -> Vec<&ComponentInfo> {
    //     self.inner
    //         .get(zone)
    //         .map(|vec| vec.iter().filter(|id| ids.index(id) >= 0).collect())
    // }
}
