use crate::models::job_result::ProviderTask;
use crate::service::judgment::http_latestblock_judg::CacheKey;
use crate::tasks::generator::TaskApplicant;
use crate::CONFIG;
use anyhow::Error;
use common::component::ComponentInfo;
use common::job_manage::{JobDetail, JobPing, JobResultDetail};
use common::jobs::{Job, JobAssignment, JobResult};
use common::models::PlanEntity;
use common::util::get_current_time;
use common::{ComponentId, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub type TaskName = String;
pub type TaskType = String;
#[derive(Clone, Default, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TaskKey {
    pub task_type: String,
    pub task_name: String,
}

#[derive(Clone, Debug, Default)]
pub struct JobResultCache {
    pub result_cache_map: HashMap<ComponentId, HashMap<TaskKey, TaskResultCache>>,
}

impl JobResultCache {
    pub fn init_cache(&mut self, assignments: HashMap<ComponentId, JobAssignment>) {
        //Todo: Init cache
    }
    pub fn get_jobs_number(&self) -> usize {
        self.result_cache_map
            .iter()
            .fold(0, |count, (key, map)| count + map.len())
    }
}

#[derive(Clone, Debug)]
pub struct TaskResultCache {
    pub results: VecDeque<JobResult>,
    pub update_time: Timestamp,
}

impl Deref for TaskResultCache {
    type Target = VecDeque<JobResult>;

    fn deref(&self) -> &VecDeque<JobResult> {
        &self.results
    }
}
impl DerefMut for TaskResultCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.results
    }
}

impl TaskResultCache {
    pub fn push_back_cache(&mut self, job_result: JobResult) {
        self.results.push_back(job_result);
        self.update_time = get_current_time();
    }

    pub fn new(create_time: Timestamp) -> Self {
        Self {
            results: VecDeque::new(),
            update_time: create_time,
        }
    }
    pub fn is_result_too_old(&self) -> bool {
        (get_current_time() - self.get_latest_update_time())
            > (CONFIG.generate_new_regular_timeout * 1000)
    }
    pub fn get_latest_update_time(&self) -> Timestamp {
        self.update_time
        // if self.results.is_empty() {
        //     self.update_time
        // } else {
        //     self.results.back().unwrap().receive_timestamp
        // }
    }
    pub fn reset_timestamp(&mut self, timestamp: Timestamp) {
        self.update_time = timestamp;
        self.results.clear()
    }
}

impl Default for TaskResultCache {
    fn default() -> Self {
        Self {
            results: VecDeque::new(),
            update_time: 0,
        }
    }
}
