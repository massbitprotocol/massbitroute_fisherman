use serde::{Serialize, Deserialize};
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct ProcessorState {

}

impl ProcessorState {
    pub fn default() -> ProcessorState {
        ProcessorState {

        }
    }
}