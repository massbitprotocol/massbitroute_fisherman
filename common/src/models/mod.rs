pub mod plan_entity;
pub mod response_values;
use serde::{Deserialize, Serialize};
/*
 * Store time frame information
 */
#[derive(Clone, Serialize, Deserialize, Debug, Default, Eq, PartialEq, Hash)]
pub struct TimeFrames {}

pub use plan_entity::PlanEntity;
pub use response_values::*;
