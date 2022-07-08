use crate::seaorm::plans::Model as PlanModel;
use crate::seaorm::{
    job_assignments, job_result_benchmarks, job_result_http_requests, job_result_latest_blocks,
    job_result_pings, jobs, plans, workers,
};
use common::component::{ChainInfo, ComponentType, Zone};
use common::job_manage::{JobBenchmarkResult, JobResultDetail, JobRole};
use common::jobs::{Job, JobAssignment, JobResult};
use common::models::plan_entity::PlanStatus;
use common::models::PlanEntity;
use common::tasks::eth::JobLatestBlockResult;
use common::tasks::ping::JobPingResult;
use common::util::get_current_time;
use common::workers::WorkerInfo;
use core::default::Default;
use log::debug;
use sea_orm::ActiveValue::Set;
use sea_orm::NotSet;
use serde_json::Value;
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

        //debug!("from job_detail: {:?}", job_detail);

        jobs::ActiveModel {
            job_id: Set(job.job_id.to_owned()),
            job_type: Set(job.job_type.to_owned()),
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
            phase: Set(job.phase.to_string()),
        }
    }
}

impl From<&JobAssignment> for job_assignments::ActiveModel {
    fn from(assign: &JobAssignment) -> Self {
        job_assignments::ActiveModel {
            job_id: Set(assign.job.job_id.to_owned()),
            job_type: Set(assign.job.job_detail.as_ref().unwrap().get_job_name()),
            job_name: Set(assign.job.job_name.to_owned()),
            worker_id: Set(assign.worker.get_id()),
            plan_id: Set(assign.job.plan_id.to_owned()),
            status: Set(String::from("assigned")),
            assign_time: Set(get_current_time() as i64),
            ..Default::default()
        }
    }
}

impl job_result_pings::Model {
    pub fn add_response_time(&mut self, response_duration: i64) {
        if let Some(response_durations) = self.response_durations.as_array_mut() {
            response_durations.push(Value::from(response_duration))
        } else {
            self.response_durations = Value::Array(vec![Value::from(response_duration)]);
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

        Job {
            job_id: model.job_id.to_string(),
            job_type: model.job_type.to_string(),
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
            phase: JobRole::from_str(model.phase.as_str()).unwrap_or_default(),
        }
    }
}

impl From<&JobPingResult> for job_result_pings::Model {
    fn from(result: &JobPingResult) -> Self {
        if result.response.error_code == 0 {
            job_result_pings::Model {
                id: 0,
                job_id: result.job.job_id.clone(),
                plan_id: result.job.plan_id.clone(),
                worker_id: result.worker_id.clone(),
                provider_id: result.job.component_id.clone(),
                provider_type: result.job.component_type.to_string(),
                execution_timestamp: 0,
                recorded_timestamp: 0,
                response_durations: Value::Array(vec![Value::from(
                    result.response.response_duration as i64,
                )]),
                error_number: 0,
            }
        } else {
            job_result_pings::Model {
                id: 0,
                job_id: result.job.job_id.clone(),
                plan_id: result.job.plan_id.clone(),
                worker_id: result.worker_id.clone(),
                provider_id: result.job.component_id.clone(),
                provider_type: result.job.component_type.to_string(),
                execution_timestamp: 0,
                recorded_timestamp: 0,
                response_durations: Value::Array(vec![]),
                error_number: 1,
            }
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
            response_durations: Set(model.response_durations.clone()),
            error_number: Set(model.error_number),
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
            response_durations: Set(result.response.response_duration as i64),
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
            plan_id: Set(result.job.plan_id.to_string()),
            worker_id: Set(result.worker_id.to_owned()),
            provider_id: Set(result.job.component_id.to_owned()),
            provider_type: Set(result.job.component_type.to_string()),
            block_number: Set(result.response.block_number as i64),
            block_timestamp: Set(result.response.block_timestamp as i64),
            chain_id: Set(result.response.chain_info.chain_id()),
            http_code: Set(result.response.http_code as i32),
            error_code: Set(result.response.error_code as i32),
            message: Set(result.response.message.to_string()),
            response_duration: Set(result.response.response_duration),
            execution_timestamp: Set(result.execution_timestamp as i64),
            block_hash: Set(result.response.block_hash.to_owned()),
            ..Default::default()
        }
    }
}

impl From<&JobResult> for job_result_http_requests::ActiveModel {
    fn from(job_result: &JobResult) -> Self {
        let mut result = &Default::default();
        if let JobResultDetail::HttpRequest(_result) = &job_result.result_detail {
            result = _result;
        }

        let values = serde_json::to_value(result.response.detail.to_owned()).unwrap_or_default();
        job_result_http_requests::ActiveModel {
            id: NotSet,
            job_id: Set(result.job.job_id.to_owned()),
            job_name: Set(result.job.job_name.to_owned()),
            plan_id: Set(result.job.plan_id.to_string()),
            worker_id: Set(job_result.worker_id.to_owned()),
            provider_id: Set(result.job.component_id.to_owned()),
            provider_type: Set(result.job.component_type.to_string()),
            execution_timestamp: Set(job_result.receive_timestamp),
            chain_id: Set(job_result
                .chain_info
                .as_ref()
                .unwrap_or(&ChainInfo::default())
                .chain_id()
                .clone()),
            http_code: Set(result.response.http_code as i32),
            error_code: Set(result.response.error_code as i32),
            message: Set(result.response.message.to_string()),
            values: Set(values),
            response_duration: Set(result.response.response_duration),
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
            expiry_time: Set(entity.expiry_time.to_owned()),
            result: Set(entity.result.to_owned()),
            message: Set(entity.message.to_owned()),
            status: Set(entity.status.to_string()),
            phase: Set(entity.phase.to_owned()),
            ..Default::default()
        }
    }
}

impl From<&PlanEntity> for PlanModel {
    fn from(entity: &PlanEntity) -> Self {
        PlanModel {
            id: entity.id,
            plan_id: entity.plan_id.clone(),
            provider_id: entity.provider_id.clone(),
            request_time: 0,
            finish_time: None,
            result: None,
            message: None,
            status: entity.status.to_string(),
            phase: entity.phase.to_string(),
            expiry_time: entity.expiry_time,
        }
    }
}
impl From<&crate::plans::Model> for PlanEntity {
    fn from(info: &crate::plans::Model) -> Self {
        PlanEntity {
            id: info.id.clone(),
            provider_id: info.provider_id.clone(),
            request_time: info.request_time.clone(),
            finish_time: info.finish_time.clone(),
            expiry_time: info.expiry_time.clone(),
            result: info.result.clone(),
            message: info.message.clone(),
            status: PlanStatus::from_str(&*info.status).unwrap_or_default(),
            plan_id: info.plan_id.clone(),
            phase: info.phase.clone(),
        }
    }
}

impl From<&workers::Model> for WorkerInfo {
    fn from(info: &workers::Model) -> Self {
        let zone = match Zone::from_str(info.zone.as_str()) {
            Ok(zone) => zone,
            Err(_err) => Zone::default(),
        };
        WorkerInfo {
            worker_id: info.worker_id.clone(),
            worker_ip: info.worker_ip.clone(),
            url: info.url.clone(),
            zone,
            worker_spec: Default::default(),
            available_time_frame: None,
        }
    }
}
