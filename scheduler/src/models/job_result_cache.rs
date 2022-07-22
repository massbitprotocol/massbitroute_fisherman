use crate::models::component::ProviderPlan;
use crate::persistence::PlanModel;
use crate::service::judgment::JudgmentsResult;
use crate::{CONFIG, RESULT_CACHE_MAX_LENGTH};
use common::jobs::{Job, JobAssignment, JobResult};
use common::models::PlanEntity;
use common::util::get_current_time;
use common::{ComponentId, JobId, PlanId, Timestamp};
use futures_util::StreamExt;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type TaskName = String;
pub type TaskType = String;
#[derive(Clone, Default, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TaskKey {
    pub task_type: String,
    pub task_name: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct PlanTaskResultKey {
    pub plan_id: PlanId,
    pub task_type: String,
    pub task_name: String,
}

impl PlanTaskResultKey {
    pub fn new(plan_id: String, task_type: TaskType, task_name: TaskName) -> Self {
        Self {
            plan_id,
            task_type,
            task_name,
        }
    }
}
#[derive(Debug, Default)]
pub struct JobResultCache {
    pub result_cache_map: Mutex<HashMap<ComponentId, HashMap<TaskKey, TaskResultCache>>>,
    //Fixme: Check size of this cache. There is no clean up function
    pub task_judg_result: Mutex<HashMap<ComponentId, HashMap<PlanTaskResultKey, JudgmentsResult>>>,
}

impl JobResultCache {
    pub fn init_cache(&self, _assignments: HashMap<ComponentId, JobAssignment>) {
        //Todo: Init cache
    }
    pub async fn append_results(
        &self,
        results: HashMap<ComponentId, HashMap<TaskKey, TaskResultCache>>,
    ) -> Result<(), anyhow::Error> {
        let mut cache_content = self.result_cache_map.lock().await;
        for (provider_id, values) in results {
            let mut result_cache = cache_content
                .entry(provider_id)
                .or_insert(HashMap::default());
            for (key, task_result) in values {
                match result_cache.get_mut(&key) {
                    None => {
                        result_cache.insert(key, task_result);
                    }
                    Some(current_value) => {
                        let TaskResultCache {
                            mut results,
                            update_time,
                        } = task_result;
                        current_value.update_time = update_time;
                        current_value.append(&mut results);
                        while current_value.len() > RESULT_CACHE_MAX_LENGTH {
                            current_value.pop_front();
                        }
                    }
                }
            }
        }
        Ok(())
    }
    pub async fn get_latest_update_task(
        &self,
        provider_id: &ComponentId,
    ) -> HashMap<TaskKey, Timestamp> {
        self.result_cache_map
            .lock()
            .await
            .get(provider_id)
            .map(|res| {
                res.iter()
                    .map(|(key, value)| (key.clone(), value.update_time.clone()))
                    .collect::<HashMap<TaskKey, Timestamp>>()
            })
            .unwrap_or_default()
    }
    pub async fn get_plan_judge_result(
        &self,
        provider_id: &ComponentId,
        plan_id: &PlanId,
    ) -> HashMap<PlanTaskResultKey, JudgmentsResult> {
        let result_content = self.task_judg_result.lock().await;
        return match result_content.get(provider_id) {
            None => HashMap::new(),
            Some(map) => {
                let results = map
                    .iter()
                    .filter_map(|(key, value)| {
                        if key.plan_id == *plan_id {
                            Some((key.clone(), value.clone()))
                        } else {
                            None
                        }
                    })
                    .collect();
                results
            }
        };
    }

    // pub fn get_jobs_number(&self) -> usize {
    //     self.result_cache_map
    //         .iter()
    //         .fold(0, |count, (_key, map)| count + map.len())
    // }
    pub async fn get_provider_judg_result(
        &self,
        provider_id: &ComponentId,
        plan_id: &PlanId,
    ) -> HashMap<TaskKey, JudgmentsResult> {
        let cache_judg_result = self.task_judg_result.lock().await;
        debug!("Cache judg result {:?}", &cache_judg_result);
        cache_judg_result
            .get(provider_id)
            .map(|res| {
                res.iter()
                    .filter(|(key, _)| key.plan_id.as_str() == plan_id.as_str())
                    .map(|(key, value)| {
                        (
                            TaskKey {
                                task_type: key.task_type.clone(),
                                task_name: key.task_name.clone(),
                            },
                            value.clone(),
                        )
                    })
                    .collect::<HashMap<TaskKey, JudgmentsResult>>()
            })
            .unwrap_or_default()
        // .and_then(|res| {
        //     res.iter(|(key, value)| {
        //         (
        //             TaskKey {
        //                 task_type: key.task_type.clone(),
        //                 task_name: key.task_name.clone(),
        //             },
        //             value.clone(0),
        //         )
        //     })
        //     .collect::<HashMap<TaskKey, JudgmentsResult>>()
        // })
        // .unwrap_or_default()
    }
    pub async fn get_judg_result(
        &self,
        provider_plan: Arc<ProviderPlan>,
        task_type: &String,
        task_name: &String,
    ) -> Option<JudgmentsResult> {
        let key = PlanTaskResultKey::new(
            provider_plan.plan.plan_id.clone(),
            task_type.clone(),
            task_name.clone(),
        );
        self.task_judg_result
            .lock()
            .await
            .get(&provider_plan.plan.provider_id)
            .and_then(|map| map.get(&key))
            .map(|val| val.clone())
    }
    pub async fn update_plan_results(
        &self,
        plan: &PlanEntity,
        job_results: &HashMap<JobId, JudgmentsResult>,
        plan_jobs: &Vec<Job>,
    ) {
        let map_jobs = plan_jobs
            .iter()
            .map(|job| {
                (
                    job.job_id.clone(),
                    PlanTaskResultKey::new(
                        plan.plan_id.clone(),
                        job.job_type.clone(),
                        job.job_name.clone(),
                    ),
                )
            })
            .collect::<HashMap<JobId, PlanTaskResultKey>>();
        let mut results = HashMap::new();
        for (job_id, judgment_result) in job_results.iter() {
            if let Some(key) = map_jobs.get(job_id) {
                results.insert(key.clone(), judgment_result.clone());
            }
        }
        debug!("Extend task_judg_result {:?}", results);
        self.task_judg_result
            .lock()
            .await
            .entry(plan.provider_id.clone())
            .or_insert(HashMap::default())
            .extend(results);
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
