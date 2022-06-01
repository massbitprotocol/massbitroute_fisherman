use crate::Value;
use crate::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CheckStep {
    #[serde(default)]
    pub action: Value,
    #[serde(default)]
    pub return_name: String,
    #[serde(default)]
    pub failed_case: FailedCase,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct FailedCase {
    #[serde(default)]
    pub critical: bool,
    #[serde(default)]
    message: String,
    #[serde(default)]
    pub conclude: CheckMkStatus,
}

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub enum CheckMkStatus {
    Ok = 0,
    Warning = 1,
    Critical = 2,
    Unknown = 3,
}

impl Default for CheckMkStatus {
    fn default() -> Self {
        CheckMkStatus::Unknown
    }
}
