use crate::Value;
use crate::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct CheckStep {
    #[serde(default)]
    action: Value,
    #[serde(default)]
    return_name: String,
    #[serde(default)]
    failed_case: FailedCase,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct FailedCase {
    #[serde(default)]
    critical: bool,
    #[serde(default)]
    message: String,
    #[serde(default)]
    conclude: CheckMkStatus,
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
