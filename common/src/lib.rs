pub mod component;
pub mod job_action;
pub mod job_manage;
pub mod jobs;
pub mod logger;
pub mod models;
pub mod task_spawn;
pub mod tasks;
pub mod types;
pub mod util;
pub mod workers;

use crate::component::ComponentInfo;
use lazy_static::lazy_static;
pub use types::*;
const DEFAULT_JOB_INTERVAL: Timestamp = 1000;
const DEFAULT_JOB_TIMEOUT: Timestamp = 5000;
lazy_static! {
    pub static ref DOMAIN: String = env::var("DOMAIN").expect("There is no env var DOMAIN");
    pub static ref WORKER_ID: String =
        env::var("WORKER_ID").expect("There is no env var WORKER_ID");
}
