use std::collections::HashMap;

pub mod component;
pub mod job_result;
pub mod job_result_cache;
pub mod jobs;
pub mod providers;
pub mod workers;

pub type TaskDependency = HashMap<String, Vec<String>>;
