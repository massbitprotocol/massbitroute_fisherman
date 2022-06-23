use crate::tasks::generator::TaskApplicant;
use crate::CONFIG;
use anyhow::Error;
use common::component::ComponentInfo;
use common::job_manage::{JobDetail, JobPing, JobResultDetail};
use common::jobs::{Job, JobResult};
use common::models::PlanEntity;
use common::util::get_current_time;
use common::{ComponentId, Timestamp};
use std::collections::{HashMap, VecDeque};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub type TaskName = String;

#[derive(Clone, Debug, Default)]
pub struct JobResultCache {
    pub result_cache_map: HashMap<ComponentId, HashMap<TaskName, TaskResultCache>>,
}

#[derive(Clone, Debug)]
pub struct TaskResultCache {
    pub results: VecDeque<JobResult>,
    pub create_time: Timestamp,
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
    pub fn is_result_too_old(&self) -> bool {
        let update_time = if self.results.is_empty() {
            self.create_time
        } else {
            self.results.back().unwrap().receive_timestamp
        };
        (get_current_time() - update_time) > (CONFIG.generate_new_regular_timeout * 1000)
    }
    pub fn new(create_time: Timestamp) -> Self {
        Self {
            results: VecDeque::new(),
            create_time,
        }
    }
}

impl Default for TaskResultCache {
    fn default() -> Self {
        Self {
            results: VecDeque::new(),
            create_time: 0,
        }
    }
}
