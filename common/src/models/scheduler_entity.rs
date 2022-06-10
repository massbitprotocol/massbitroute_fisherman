use crate::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct SchedulerEntity {
    pub id: i64,
    pub scheduler_id: String,
    pub provider_id: String,
    pub request_time: Timestamp,
    pub finish_time: Option<Timestamp>,
    pub result: Option<String>,
    pub status: String,
    pub message: Option<String>,
    pub phase: String,
}
