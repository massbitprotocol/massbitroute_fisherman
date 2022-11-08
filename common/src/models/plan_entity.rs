use crate::Timestamp;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct PlanEntity {
    pub id: i64,
    pub plan_id: String,
    pub provider_id: String,
    pub request_time: Timestamp,
    pub finish_time: Option<Timestamp>,
    pub expiry_time: Timestamp,
    pub result: Option<String>, //success/failed/expired
    pub status: PlanStatus,
    pub message: Option<String>,
    pub phase: String,
}

impl PlanEntity {
    pub fn new(
        provider_id: String,
        request_time: Timestamp,
        expiry_time: Timestamp,
        phase: String,
    ) -> Self {
        PlanEntity {
            id: 0,
            plan_id: Uuid::new_v4().to_string(),
            provider_id,
            request_time,
            finish_time: None,
            expiry_time,
            result: None,
            status: Default::default(),
            message: None,
            phase,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum PlanStatus {
    Init,
    Generated,
    Cancel,
    Finished,
}

impl ToString for PlanStatus {
    fn to_string(&self) -> String {
        match self {
            PlanStatus::Init => "Init".to_string(),
            PlanStatus::Generated => "Generated".to_string(),
            PlanStatus::Cancel => "Cancel".to_string(),
            PlanStatus::Finished => "Finished".to_string(),
        }
    }
}

impl FromStr for PlanStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Init" => Ok(PlanStatus::Init),
            "Generated" => Ok(PlanStatus::Generated),
            "Cancel" => Ok(PlanStatus::Cancel),
            "Finished" => Ok(PlanStatus::Finished),
            _ => Err(()),
        }
    }
}

impl Default for PlanStatus {
    fn default() -> Self {
        PlanStatus::Init
    }
}
