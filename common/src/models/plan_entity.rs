use crate::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct PlanEntity {
    pub id: i64,
    pub plan_id: String,
    pub provider_id: String,
    pub request_time: Timestamp,
    pub finish_time: Option<Timestamp>,
    pub result: Option<String>,
    pub status: String,
    pub message: Option<String>,
    pub phase: String,
}

impl PlanEntity {
    pub fn new(provider_id: String, request_time: Timestamp, phase: String) -> Self {
        PlanEntity {
            id: 0,
            plan_id: Uuid::new_v4().to_string(),
            provider_id,
            request_time,
            finish_time: None,
            result: None,
            status: "".to_string(),
            message: None,
            phase,
        }
    }
}
