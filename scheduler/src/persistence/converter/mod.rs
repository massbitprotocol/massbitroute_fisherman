use crate::persistence::seaorm::{
    job_result_benchmarks, job_result_latest_blocks, job_result_pings, jobs, plans, workers,
};
use common::component::ComponentType;
use common::job_manage::{Job, JobBenchmarkResult, JobDetail};
use common::models::PlanEntity;
use common::tasks::eth::JobLatestBlockResult;
use common::tasks::ping::JobPingResult;
use common::worker::WorkerInfo;
use core::default::Default;
use log::debug;
use sea_orm::ActiveValue::Set;
use sea_orm::NotSet;
use serde_json::{Error, Value};
use std::collections::HashMap;
use std::str::FromStr;

impl From<&WorkerInfo> for workers::ActiveModel {
    fn from(worker: &WorkerInfo) -> Self {
        let workers = workers::ActiveModel {
            worker_id: Set(worker.worker_id.to_owned()),
            worker_ip: Set(worker.worker_ip.to_owned()),
            active: Set(1),
            zone: Set(format!("{:?}", &worker.zone)),
            url: Set(worker.url.to_owned()),
            ..Default::default()
        };
        workers
    }
}

impl From<&Job> for jobs::ActiveModel {
    fn from(job: &Job) -> Self {
        debug!("job.job_detail: {:?}", job.job_detail);
        let job_detail = match &job.job_detail {
            None => None,
            Some(detail) => match serde_json::to_value(detail) {
                Ok(value) => Some(value),
                Err(err) => {
                    debug!("err from job_detail: {:?}", err);
                    None
                }
            },
        };
        let header = match job.header.is_empty() {
            true => None,
            false => match serde_json::to_value(job.header.to_owned()) {
                Ok(value) => Some(value),
                Err(err) => {
                    debug!("err from job_detail: {:?}", err);
                    None
                }
            },
        };

        debug!("from job_detail: {:?}", job_detail);

        jobs::ActiveModel {
            job_id: Set(job.job_id.to_owned()),
            job_name: Set(job.job_name.to_owned()),
            plan_id: Set(job.plan_id.to_string()),
            component_id: Set(job.component_id.to_owned()),
            component_type: Set(job.component_type.to_string()),
            priority: Set(job.priority),
            expected_runtime: Set(job.expected_runtime as i64),
            parallelable: Set(job.parallelable),
            timeout: Set(job.timeout as i64),
            job_detail: Set(job_detail),
            component_url: Set(job.component_url.to_owned()),
            header: Set(header),
            interval: Set(job.interval),
            repeat_number: Set(job.repeat_number),
            id: NotSet,
        }
    }
}

impl job_result_pings::Model {
    pub fn add_response_time(&mut self, response_time: i64) {
        if let Some(response_times) = self.response_times.as_array_mut() {
            response_times.push(Value::from(response_time))
        } else {
            self.response_times = Value::Array(vec![Value::from(response_time)]);
        }
    }
}

impl From<&jobs::Model> for Job {
    fn from(model: &jobs::Model) -> Self {
        let mut headers = HashMap::<String, String>::new();
        if let Some(header) = &model.header {
            if let Some(map) = header.as_object() {
                for (key, val) in map.iter() {
                    headers.insert(
                        key.clone(),
                        val.as_str()
                            .map(|str| str.to_string())
                            .unwrap_or(String::new()),
                    );
                }
            }
        }
        // let mut header: HashMap<String, String> = serde_json::from_str(json).unwrap();
        // let mut map = HashMap::new();
        // for key in keys {
        //     let (k, v) = lookup.remove_entry (key).unwrap();
        //     map.insert(k, v);
        // }
        // Ok(map)
        Job {
            job_id: model.job_id.to_string(),
            job_name: model.job_name.to_string(),
            plan_id: model.plan_id.to_string(),
            component_id: model.component_id.to_string(),
            component_type: ComponentType::from_str(model.component_type.as_str())
                .unwrap_or(ComponentType::Node),
            priority: model.priority.clone(),
            expected_runtime: model.expected_runtime.clone(),
            parallelable: model.parallelable.clone(),
            timeout: model.timeout.clone(),
            component_url: model.component_url.to_string(),
            repeat_number: model.repeat_number.clone(),
            interval: model.interval.clone(),
            header: headers,
            //Fixme: add converter here
            job_detail: None,
        }
    }
}

impl From<&JobPingResult> for job_result_pings::Model {
    fn from(result: &JobPingResult) -> Self {
        job_result_pings::Model {
            id: 0,
            job_id: result.job.job_id.clone(),
            plan_id: result.job.plan_id.clone(),
            worker_id: result.worker_id.clone(),
            provider_id: result.job.component_id.clone(),
            provider_type: result.job.component_type.to_string(),
            execution_timestamp: 0,
            recorded_timestamp: 0,
            response_times: Value::Array(vec![Value::from(result.response.response_time as i64)]),
        }
    }
}

impl job_result_pings::ActiveModel {
    pub fn from_model(model: &job_result_pings::Model) -> Self {
        job_result_pings::ActiveModel {
            job_id: Set(model.job_id.to_owned()),
            plan_id: Set(model.plan_id.to_string()),
            worker_id: Set(model.worker_id.to_owned()),
            provider_id: Set(model.provider_id.to_owned()),
            provider_type: Set(model.provider_type.to_string()),
            response_times: Set(model.response_times.clone()),
            ..Default::default()
        }
    }
}
/*
impl From<&JobPingResult> for job_result_pings::ActiveModel {
    fn from(result: &JobPingResult) -> Self {
        job_result_pings::ActiveModel {
            job_id: Set(result.job.job_id.to_owned()),
            //job_name: Set(result.job.job_name.to_owned()),
            plan_id: Set(result.job.plan_id.to_string()),
            worker_id: Set(result.worker_id.to_owned()),
            provider_id: Set(result.job.component_id.to_owned()),
            provider_type: Set(result.job.component_type.to_string()),
            response_times: Set(result.response.response_time as i64),
            ..Default::default()
        }
    }
}
*/
impl From<&JobBenchmarkResult> for job_result_benchmarks::ActiveModel {
    fn from(result: &JobBenchmarkResult) -> Self {
        job_result_benchmarks::ActiveModel {
            job_id: Set(result.job.job_id.to_owned()),
            //job_name: Set(result.job.job_name.to_owned()),
            plan_id: Set(result.job.plan_id.to_string()),
            worker_id: Set(result.worker_id.to_owned()),
            provider_id: Set(result.job.component_id.to_owned()),
            provider_type: Set(result.job.component_type.to_string()),
            request_rate: Set(result.response.request_rate as f64),
            transfer_rate: Set(result.response.transfer_rate as f64),
            average_latency: Set(result.response.average_latency as f64),
            histogram90: Set(result
                .response
                .histograms
                .get(&90)
                .map(|val| val.clone())
                .unwrap_or(0_f32) as f64),
            histogram95: Set(result
                .response
                .histograms
                .get(&95)
                .map(|val| val.clone())
                .unwrap_or(0_f32) as f64),
            histogram99: Set(result
                .response
                .histograms
                .get(&99)
                .map(|val| val.clone())
                .unwrap_or(0_f32) as f64),
            ..Default::default()
        }
    }
}

impl From<&JobLatestBlockResult> for job_result_latest_blocks::ActiveModel {
    fn from(result: &JobLatestBlockResult) -> Self {
        job_result_latest_blocks::ActiveModel {
            job_id: Set(result.job.job_id.to_owned()),
            //job_name: Set(result.job.job_name.to_owned()),
            plan_id: Set(result.job.plan_id.to_string()),
            worker_id: Set(result.worker_id.to_owned()),
            provider_id: Set(result.job.component_id.to_owned()),
            provider_type: Set(result.job.component_type.to_string()),
            block_chain: Set(String::new()),
            block_number: Set(result.response.block_number as i64),
            block_timestamp: Set(result.response.block_timestamp as i64),
            ..Default::default()
        }
    }
}

impl From<&PlanEntity> for plans::ActiveModel {
    fn from(entity: &PlanEntity) -> Self {
        plans::ActiveModel {
            plan_id: Set(entity.plan_id.to_owned()),
            provider_id: Set(entity.provider_id.to_owned()),
            request_time: Set(entity.request_time.to_owned()),
            finish_time: Set(entity.finish_time.to_owned()),
            result: Set(entity.result.to_owned()),
            message: Set(entity.message.to_owned()),
            status: Set(entity.status.to_string()),
            phase: Set(entity.phase.to_owned()),
            ..Default::default()
        }
    }
}
