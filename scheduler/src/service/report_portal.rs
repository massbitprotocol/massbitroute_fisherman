use crate::service::judgment::JudgmentsResult;
use crate::{URL_PORTAL_PROVIDER_REPORT, URL_PORTAL_PROVIDER_VERIFY};
use anyhow::{anyhow, Error};
use common::component::ComponentType;
use common::job_manage::JobRole;
use common::{ComponentId, Deserialize, PlanId, Serialize, DEFAULT_HTTP_REQUEST_TIMEOUT};
use log::{debug, info};
use reqwest::Response;

use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const REPORT_PATH: &str = "logs/report.txt";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ReportRecord {
    pub report_time: String,
    provider_id: ComponentId,
    plan_id: PlanId,
    result: JudgmentsResult,
}

impl ReportRecord {
    pub fn new(
        report_time: String,
        provider_id: ComponentId,
        plan_id: PlanId,
        result: JudgmentsResult,
    ) -> Self {
        ReportRecord {
            report_time,
            provider_id,
            plan_id,
            result,
        }
    }
}

impl ToString for ReportRecord {
    fn to_string(&self) -> String {
        format!(
            "{} {} {} {}",
            self.report_time, self.provider_id, self.plan_id, self.result
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct ReportFailedReason {
    job_name: String,
    failed_detail: String,
}

impl ReportFailedReason {
    pub fn new(job_name: String, failed_detail: String) -> Self {
        ReportFailedReason {
            job_name,
            failed_detail,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct ReportFailedReasons {
    inner: Vec<ReportFailedReason>,
}

impl Deref for ReportFailedReasons {
    type Target = Vec<ReportFailedReason>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ReportFailedReasons {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ReportFailedReasons {
    pub(crate) fn new(reasons: Vec<ReportFailedReason>) -> Self {
        ReportFailedReasons { inner: reasons }
    }
    pub(crate) fn new_with_single_reason(job_name: String, failed_detail: String) -> Self {
        let reasons = vec![ReportFailedReason::new(job_name, failed_detail)];
        ReportFailedReasons { inner: reasons }
    }
    pub fn into_inner(self) -> Vec<ReportFailedReason> {
        self.inner
    }
}

impl ToString for ReportFailedReasons {
    fn to_string(&self) -> String {
        let mut msg = String::new();
        for reason in self.inner.iter() {
            msg.push_str(&format!("{}: {}; ", reason.job_name, reason.failed_detail));
        }
        msg
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct StoreReport {
    pub reporter: String,
    pub reporter_role: JobRole,
    #[serde(skip_deserializing)]
    pub domain: String,
    #[serde(skip_deserializing)]
    pub authorization: String,
    #[serde(skip_deserializing)]
    pub provider_id: String,
    pub average_latency: f32,
    pub total_req: usize,
    pub total_duration: f32,
    pub total_read_byte: u64,
    pub non_2xx_3xx_req: usize,
    pub percent_low_latency: f32,
    pub is_data_correct: bool,
    pub provider_type: ComponentType,
    pub report_time: u128,
    pub status_detail: String,
    pub report_type: String,

    pub request_rate: f32,
    pub transfer_rate: f32,
    pub histogram_90: f32,
    pub histogram_95: f32,
    pub histogram_99: f32,
    pub stdev_latency: f32,
    pub max_latency: f32,
}

impl StoreReport {
    pub fn build(
        reporter: &String,
        reporter_role: &JobRole,
        authorization: &String,
        domain: &String,
    ) -> StoreReport {
        let report_type = match reporter_role {
            JobRole::Verification => "Benchmark".to_string(),
            JobRole::Regular => "ReportProvider".to_string(),
        };
        StoreReport {
            reporter: reporter.clone(),
            reporter_role: reporter_role.clone(),
            authorization: authorization.clone(),
            domain: domain.clone(),
            report_type,
            ..Default::default()
        }
    }

    // Short store before report
    pub fn set_report_data_short(
        &mut self,
        judge_result: &JudgmentsResult,
        component_id: &ComponentId,
        component_type: &ComponentType,
    ) {
        match judge_result {
            JudgmentsResult::Pass => {
                self.is_data_correct = true;
                self.status_detail = "Pass".to_string();
            }
            JudgmentsResult::Failed(reasons) => {
                self.is_data_correct = false;
                self.status_detail = reasons.to_string();
            }
            JudgmentsResult::Unfinished => {
                self.is_data_correct = false;
                self.status_detail = "Internal Error: Report Unfinished task".to_string();
            }
        };

        self.provider_id = component_id.clone();
        self.provider_type = component_type.clone();
        self.report_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
    }

    fn create_body(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(&self)?)
    }

    fn get_url(&self) -> String {
        match self.reporter_role {
            JobRole::Verification => {
                format!(
                    "{}/{}",
                    URL_PORTAL_PROVIDER_VERIFY.as_str(),
                    self.provider_id
                )
            }
            JobRole::Regular => {
                format!(
                    "{}/{}",
                    URL_PORTAL_PROVIDER_REPORT.as_str(),
                    self.provider_id
                )
            }
        }
    }

    pub async fn send_data(&self) -> Result<Response, Error> {
        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        // create body
        let body = self.create_body()?;
        // get url
        let url = self.get_url();
        info!("body send_data to {}: {:?}", url, body);

        let request_builder = client
            .post(url)
            .header("content-type", "application/json")
            .header("Authorization", &self.authorization)
            .body(body)
            .timeout(Duration::from_millis(DEFAULT_HTTP_REQUEST_TIMEOUT));
        debug!("request_builder: {:?}", request_builder);
        let response = request_builder.send().await?;
        Ok(response)
    }

    // For testing only
    pub fn write_data(&self, data: ReportRecord) -> Result<String, Error> {
        let data = serde_json::to_string(&data)?;
        let report_file = std::path::Path::new(REPORT_PATH);
        if let Some(path) = report_file.parent() {
            match fs::create_dir_all(path) {
                Ok(_) => {}
                Err(err) => {
                    Err(anyhow!("{:?}", err))?;
                }
            }
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(report_file)?;
        writeln!(file, "{}", data)?;
        Ok(data)
    }
}
