use crate::models::job_result_cache::JobResultCache;
use crate::models::jobs::JobAssignmentBuffer;
use crate::models::providers::ProviderStorage;
use crate::models::workers::WorkerInfoStorage;
use crate::service::delivery::CancelPlanBuffer;
use anyhow::Error;
use common::workers::{Worker, WorkerStatus};
use common::{Timestamp, COMMON_CONFIG};
use log::{debug, error, info};
use reqwest::Client;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{sleep, Instant};

const GET_STATUS_PATH: &str = "get_status";

#[derive(Default)]
pub struct ServiceMonitor {
    report_file: PathBuf,
    workers: Arc<WorkerInfoStorage>,
    result_cache: Arc<JobResultCache>,
    cancel_plans_buffer: Arc<Mutex<CancelPlanBuffer>>,
    provider_storage: Arc<ProviderStorage>,
    assigment_buffer: Arc<Mutex<JobAssignmentBuffer>>,
}

trait CheckMkReport {
    fn to_check_mk_string(&self) -> String;
}

impl CheckMkReport for WorkerMonitor {
    fn to_check_mk_string(&self) -> String {
        //P "My 2nd service" count1=42|count2=21;23;27 A service with 2 graphs
        let metrics: Vec<Box<dyn ToString>> = vec![
            Box::new(self.response_time.clone()),
            Box::new(self.jobs_number_in_queue.clone()),
            Box::new(self.reports_number_in_queue.clone()),
        ];
        // Create metric string
        let metrics_string =
            metrics
                .iter()
                .enumerate()
                .fold("".to_string(), |acc, (index, metric)| {
                    if index == 0 {
                        acc + &metric.to_string()
                    } else {
                        acc + &format!("|{}", metric.to_string())
                    }
                });

        format!(
            "P \"{}\" {} {}",
            self.name, metrics_string, self.status_detail
        )
    }
}

#[derive(Clone, Default)]
pub struct Metric<T: Default + ToString + Clone> {
    name: String,
    value: T,
    warn: Option<T>,
    crit: Option<T>,
    min: Option<T>,
    max: Option<T>,
}

impl<T: Default + ToString + Clone> Metric<T> {
    pub fn new(name: &str, value: T) -> Self {
        Metric {
            name: name.to_string(),
            value,
            crit: None,
            warn: None,
            min: None,
            max: None,
        }
    }
}

impl<T: Default + ToString + Clone> ToString for Metric<T> {
    fn to_string(&self) -> String {
        let warn = self
            .warn
            .as_ref()
            .map(|x| x.to_string())
            .unwrap_or_default();
        let crit = self
            .crit
            .as_ref()
            .map(|x| x.to_string())
            .unwrap_or_default();
        let min = self.min.as_ref().map(|x| x.to_string()).unwrap_or_default();
        let max = self.max.as_ref().map(|x| x.to_string()).unwrap_or_default();

        format!(
            "{}={};{};{};{};{}",
            self.name,
            self.value.to_string(),
            warn,
            crit,
            min,
            max
        )
    }
}

pub struct WorkerMonitor {
    name: String,
    response_time: Metric<Timestamp>,
    jobs_number_in_queue: Metric<usize>,
    reports_number_in_queue: Metric<usize>,
    status_detail: String,
}

#[derive(Default)]
pub struct SchedulerMonitor {
    name: String,
    metrics: Vec<Box<dyn ToString>>,
    status_detail: String,
}

impl CheckMkReport for SchedulerMonitor {
    fn to_check_mk_string(&self) -> String {
        //P "My 2nd service" count1=42|count2=21;23;27 A service with 2 graphs
        // Create metric string
        let metrics_string =
            self.metrics
                .iter()
                .enumerate()
                .fold("".to_string(), |acc, (index, metric)| {
                    if index == 0 {
                        acc + &metric.to_string()
                    } else {
                        acc + &format!("|{}", metric.to_string())
                    }
                });

        format!(
            "P \"{}\" {} {}",
            self.name, metrics_string, self.status_detail
        )
    }
}

impl SchedulerMonitor {
    pub fn new(name: String, status_detail: String) -> Self {
        SchedulerMonitor {
            name,
            status_detail,
            ..Default::default()
        }
    }
}

impl WorkerMonitor {
    pub fn new(
        name: String,
        response_time: Metric<Timestamp>,
        worker_status: WorkerStatus,
        status_detail: String,
    ) -> Self {
        let jobs_number_in_queue =
            Metric::new("jobs_number_in_queue", worker_status.jobs_number_in_queue);
        let reports_number_in_queue = Metric::new(
            "reports_number_in_queue",
            worker_status.reports_number_in_queue,
        );
        WorkerMonitor {
            name,
            response_time,
            jobs_number_in_queue,
            reports_number_in_queue,
            status_detail,
        }
    }
}

pub struct StatsMonitor {
    name: String,
    status_detail: String,
}

impl ServiceMonitor {
    pub fn new(
        report_file: &PathBuf,
        workers: Arc<WorkerInfoStorage>,
        result_cache: Arc<JobResultCache>,
        cancel_plans_buffer: Arc<Mutex<CancelPlanBuffer>>,
        provider_storage: Arc<ProviderStorage>,
        assigment_buffer: Arc<Mutex<JobAssignmentBuffer>>,
    ) -> Self {
        ServiceMonitor {
            report_file: report_file.clone(),
            workers,
            result_cache,
            cancel_plans_buffer,
            provider_storage,
            assigment_buffer,
        }
    }
    pub async fn get_scheduler_monitor(&self) -> SchedulerMonitor {
        let assigment_buffer_len = self.assigment_buffer.lock().await.jobs.len();
        let (gateways_number, nodes_number) =
            self.provider_storage.get_number_active_providers().await;
        let cancel_plan_buffer_len = self.cancel_plans_buffer.lock().await.len();
        let task_judge_result = self.result_cache.get_task_judge_result_len().await;
        let result_cache_map = self.result_cache.get_result_cache_map_len().await;
        let active_workers_number = self.workers.get_workers_number().await;

        let metrics: Vec<Box<dyn ToString>> = vec![
            Box::new(Metric::new("assigment_buffer_len", assigment_buffer_len)),
            Box::new(Metric::new("gateways_number", gateways_number)),
            Box::new(Metric::new("nodes_number", nodes_number)),
            Box::new(Metric::new(
                "cancel_plan_buffer_len",
                cancel_plan_buffer_len,
            )),
            Box::new(Metric::new("task_judge_result", task_judge_result)),
            Box::new(Metric::new("result_cache_map", result_cache_map)),
            Box::new(Metric::new("active_workers_number", active_workers_number)),
        ];

        SchedulerMonitor {
            name: "Scheduler Monitor".to_string(),
            status_detail: "".to_string(),
            metrics,
        }
    }
    pub async fn run(self) {
        loop {
            {
                // Get worker status list
                let workers_status = self.get_workers_status().await;
                // Get scheduler status
                let scheduler_monitor = self.get_scheduler_monitor().await;
                // To string
                let mut report = Self::worker_status_to_string(&workers_status);
                report.push_str(&scheduler_monitor.to_check_mk_string());
                info!("workers_status: {}", report);
                // Write to file
                match self.write_to_file(&report) {
                    Ok(_) => {
                        info!("Updated status file");
                    }
                    Err(error) => {
                        error!("Cannot update status file: {}", error);
                    }
                }
            }

            sleep(Duration::from_millis(COMMON_CONFIG.update_status_interval)).await;
        }
    }
    fn write_to_file(&self, report: &str) -> Result<(), Error> {
        if let Some(p) = self.report_file.parent() {
            std::fs::create_dir_all(p)?
        };
        std::fs::write(&self.report_file, report)?;
        Ok(())
    }

    pub fn worker_status_to_string(workers_status: &Vec<Result<WorkerMonitor, Error>>) -> String {
        workers_status
            .iter()
            .fold("".to_string(), |acc, worker| match worker {
                Ok(worker) => acc + &worker.to_check_mk_string() + "\n",
                Err(error) => {
                    error!("worker_status_to_string error: {}", error);
                    acc
                }
            })
    }

    pub async fn get_workers_status(&self) -> Vec<Result<WorkerMonitor, Error>> {
        let mut res = vec![];
        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build();
        if client.is_err() {
            error!("get_workers_status Cannot create client {:?}", client);
            return res;
        }
        let client = client.unwrap();
        for worker in self.workers.get_workers().await {
            // get url
            let worker_status = Self::get_worker_status(worker, &client).await;
            res.push(worker_status)
        }
        res
    }
    pub async fn get_worker_status(
        worker: Arc<Worker>,
        client: &Client,
    ) -> Result<WorkerMonitor, Error> {
        // get url
        let url = worker.get_url(GET_STATUS_PATH);

        let request_builder = client
            .get(url)
            .header("content-type", "application/json")
            .timeout(Duration::from_millis(
                COMMON_CONFIG.default_http_request_timeout_ms,
            ));
        let now = Instant::now();
        debug!("request_builder get_workers_status: {:?}", request_builder);
        let response = request_builder.send().await?.text().await?;
        let elapsed = now.elapsed().as_micros() as Timestamp;
        let worker_status = serde_json::from_str::<WorkerStatus>(&response)?;
        let worker_monitor = WorkerMonitor::new(
            worker.worker_info.worker_id.to_string(),
            Metric::new("response_time", elapsed),
            worker_status,
            "".to_string(),
        );
        return Ok(worker_monitor);
    }
}
