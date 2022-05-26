use serde::{Serialize, Deserialize};
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct SchedulerState {

}

impl SchedulerState {
    pub fn default() -> SchedulerState {
        SchedulerState {

        }
    }
}