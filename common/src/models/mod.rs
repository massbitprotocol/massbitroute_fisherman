pub mod scheduler_entity;
use serde::{Deserialize, Serialize};
/*
 * Store time frame information
 */
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct TimeFrames {}

pub use scheduler_entity::SchedulerEntity;
