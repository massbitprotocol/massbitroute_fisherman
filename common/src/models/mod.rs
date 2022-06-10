pub mod plan_entity;
use serde::{Deserialize, Serialize};
/*
 * Store time frame information
 */
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct TimeFrames {}

pub use plan_entity::PlanEntity;
