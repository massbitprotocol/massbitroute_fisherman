use crate::job_manage::{
    BenchmarkResponse, JobBenchmark, JobBenchmarkResult, JobDetail, JobResultDetail,
};
use crate::jobs::Job;
use crate::logger::helper::message;
use crate::tasks::eth::CallBenchmarkError;
use crate::tasks::executor::TaskExecutor;
use crate::util::get_current_time;
use crate::{task_spawn, Timestamp, WorkerId};
use anyhow::Error;
use async_trait::async_trait;
use bytesize::ByteSize;
use log::Level::Debug;
use log::{debug, info};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::format;
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;
use wrap_wrk::{WrkBenchmark, WrkReport};

const WRK_NAME: &str = "./wrk";

#[derive(Clone, Debug, Default)]
pub struct BenchmarkExecutor {
    worker_id: WorkerId,
    benchmark_wrk_path: String,
}

#[derive(Debug)]
pub struct DetailedPercentileSpectrum {
    latency: f32,
    percent: f32,
    count: u64,
}

impl BenchmarkResponse {
    pub fn new_error(error_code: u32, message: &str) -> Self {
        BenchmarkResponse {
            request_rate: 0.0,
            transfer_rate: 0.0,
            average_latency: 0.0,
            histograms: Default::default(),
            error_code,
            message: message.to_string(),
        }
    }
}

impl BenchmarkExecutor {
    pub fn new(worker_id: WorkerId, benchmark_wrk_path: &str) -> Self {
        BenchmarkExecutor {
            worker_id,
            benchmark_wrk_path: benchmark_wrk_path.to_string(),
        }
    }
    pub async fn call_benchmark(&self, job: &Job) -> Result<BenchmarkResponse, CallBenchmarkError> {
        let Job {
            component_url,
            header,
            job_detail,
            ..
        } = job;
        let job_detail = job_detail
            .clone()
            .ok_or(CallBenchmarkError::GetJobInfoError(
                "No job detail".to_string(),
            ))?;
        let host = header.get("host").unwrap_or(&"".to_string()).to_string();
        let token = header
            .get("x-api-key")
            .unwrap_or(&Default::default())
            .to_string();
        match job_detail {
            JobDetail::Benchmark(job_detail) => {
                let JobBenchmark {
                    thread,
                    connection,
                    duration,
                    rate,
                    script,
                    chain_type,
                    component_type,
                    histograms,
                    url_path,
                } = job_detail;
                let duration = format!("{}s", duration / 1000i64);
                let mut benchmark = WrkBenchmark::build(
                    thread,
                    connection,
                    duration,
                    rate,
                    component_url.to_string(),
                    token.clone(),
                    host.clone(),
                    script,
                    WRK_NAME.to_string(),
                    self.benchmark_wrk_path.clone(),
                    Default::default(),
                );

                let stdout = benchmark.run(&component_type.to_string(), &url_path, &chain_type);
                if let Ok(stdout) = stdout {
                    return self
                        .get_result(&stdout, &histograms)
                        .map_err(|err| CallBenchmarkError::ParseResultError(format!("{:?}", err)));
                }
            }
            _ => {}
        }

        return Err(CallBenchmarkError::GetJobInfoError(format!(
            "Unknown error"
        )));
    }

    pub fn get_latency_by_percent(
        percent: f32,
        sorted_table: &Vec<DetailedPercentileSpectrum>,
    ) -> Result<f32, Error> {
        let mut latency = Err(Error::msg("cannot get latency by percent"));
        for line in sorted_table {
            if percent >= line.percent {
                latency = Ok(line.latency);
            } else {
                break;
            }
        }
        latency
    }

    fn get_result(
        &self,
        stdout: &String,
        histograms: &Vec<u32>,
    ) -> Result<BenchmarkResponse, Error> {
        //info!("{}", stdout);
        // Get percent_low_latency
        let sorted_table = Self::get_latency_table(stdout)?;
        //info!("vec table:{:?}", sorted_table);
        let mut histograms_table: HashMap<u32, f32> = Default::default();
        for percent in histograms.iter() {
            histograms_table.insert(
                *percent,
                Self::get_latency_by_percent(*percent as f32 / 100f32, &sorted_table)?,
            );
        }

        // Get Requests/sec, Transfer/sec
        let re = Regex::new(
            r"Requests/sec:\s+(?P<request_rate>\d+\.\d+)\s+Transfer/sec:\s+(?P<transfer_rate>\d+\.\d+\w+?)\s+",
        )?;
        let caps = re.captures(stdout).unwrap();
        let request_rate = caps
            .name("request_rate")
            .unwrap()
            .as_str()
            .parse::<f32>()
            .unwrap();

        // Get Requests/sec, Transfer/sec
        let transfer_rate = caps.name("transfer_rate").unwrap().as_str();
        debug!("tran_per_sec:{}", transfer_rate);
        let transfer_rate = ByteSize::from_str(&transfer_rate).unwrap();

        // Get Requests/sec, Transfer/sec
        let re = Regex::new(
            r"Thread Stats   Avg      Stdev     Max   \+/- Stdev\s+Latency\s+(?P<average_latency>\S+)",
        )?;
        let caps = re.captures(stdout).unwrap();
        let average_latency = caps.name("average_latency").unwrap().as_str().to_string();
        let average_latency = Self::parse_string_duration(&average_latency).unwrap_or_default();

        Ok(BenchmarkResponse {
            request_rate,
            transfer_rate: transfer_rate.as_u64() as f32,
            average_latency: average_latency.as_millis() as f32,
            histograms: histograms_table,
            error_code: 0,
            message: "success".to_string(),
        })
    }

    fn get_latency_table(text: &String) -> Result<Vec<DetailedPercentileSpectrum>, Error> {
        let re = Regex::new(
            r"Value   Percentile   TotalCount 1/\(1-Percentile\)\s+(?P<table>[\d.\sinf]+)#",
        )?;
        let caps = re
            .captures(text)
            .ok_or(Error::msg("Cannot capture latency table"))?;
        let table = caps.name("table").unwrap().as_str();
        //info!("table:{}", table);

        let sorted_table: Vec<DetailedPercentileSpectrum> = table
            .split("\n")
            .filter_map(|line| {
                //info!("s:{}", line);
                let arr = line
                    .split_whitespace()
                    .map(|value| value.to_string())
                    .collect::<Vec<String>>();
                //info!("arr:{:?}", arr);
                if arr.len() == 4 {
                    Some(DetailedPercentileSpectrum {
                        latency: arr[0].parse::<f32>().unwrap_or(f32::MAX),
                        percent: arr[1].parse::<f32>().unwrap_or(f32::MAX),
                        count: arr[2].parse::<u64>().unwrap_or(u64::MAX),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(sorted_table)
    }

    fn parse_string_duration(time: &String) -> Option<Duration> {
        if time.contains("-nan") || time.contains("-nanus") {
            return None;
        }
        if time.contains("ms") {
            Some(Duration::from_secs_f32(
                time.strip_suffix("ms").unwrap().parse::<f32>().unwrap() / 1000f32,
            ))
        } else if time.contains("us") {
            Some(Duration::from_secs_f32(
                time.strip_suffix("us").unwrap().parse::<f32>().unwrap() / 1000_000f32,
            ))
        } else {
            Some(Duration::from_secs_f32(
                time.strip_suffix("s").unwrap().parse::<f32>().unwrap(),
            ))
        }
    }
}

#[async_trait]
impl TaskExecutor for BenchmarkExecutor {
    async fn execute(
        &self,
        job: &Job,
        result_sender: Sender<JobResultDetail>,
    ) -> Result<(), Error> {
        debug!("TaskBenchmark execute for job {:?}", &job);
        let res = self.call_benchmark(job).await;
        let response = match res {
            Ok(res) => res,
            Err(err) => err.into(),
        };
        let current_time = get_current_time();
        debug!("Benchmark result {:?}", &response);
        // Send result
        let result = JobBenchmarkResult {
            job: job.clone(),
            worker_id: self.worker_id.clone(),
            response_timestamp: current_time,
            response,
        };
        let res = result_sender.send(JobResultDetail::Benchmark(result)).await;
        debug!("send res: {:?}", res);

        Ok(())
    }
    fn can_apply(&self, job: &Job) -> bool {
        return match job.job_detail.as_ref() {
            Some(JobDetail::Benchmark(_)) => true,
            _ => false,
        };
    }
}
