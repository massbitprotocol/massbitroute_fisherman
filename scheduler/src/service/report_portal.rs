use crate::URL_PORTAL_PROVIDER_REPORT;
use anyhow::{anyhow, Error};
use common::component::ComponentType;
use common::job_manage::JobRole;
use common::{ComponentId, Deserialize, Serialize};
use log::debug;
use reqwest::Response;
use serde_json::Value;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

const REPORT_PATH: &str = "logs/report.txt";

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
    pub report_type: JobRole,

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
        StoreReport {
            reporter: reporter.clone(),
            reporter_role: reporter_role.clone(),
            authorization: authorization.clone(),
            domain: domain.clone(),
            ..Default::default()
        }
    }

    // Short store before report
    pub fn set_report_data_short(
        &mut self,
        is_data_correct: bool,
        component_id: &ComponentId,
        component_type: &ComponentType,
    ) {
        self.is_data_correct = is_data_correct;
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

    // fn get_url(&self, job_role: &JobRole) -> String {
    //     match job_role {
    //         JobRole::Verification => {
    //             format!(
    //                 "https://portal.{}/mbr/verify/{}",
    //                 self.domain, self.provider_id
    //             )
    //         }
    //         JobRole::Regular => {
    //             format!(
    //                 "https://portal.{}/mbr/benchmark/{}",
    //                 self.domain, self.provider_id
    //             )
    //         }
    //     }
    // }
    fn get_url(&self) -> String {
        format!(
            "{}/{}",
            URL_PORTAL_PROVIDER_REPORT.as_str(),
            self.provider_id
        )
    }

    pub async fn send_data(&self) -> Result<Response, Error> {
        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        // create body
        let body = self.create_body()?;
        debug!("body send_data: {:?}", body);
        // get url
        let url = self.get_url();

        let request_builder = client
            .post(url)
            .header("content-type", "application/json")
            .header("Authorization", &self.authorization)
            .body(body);
        debug!("request_builder: {:?}", request_builder);
        let response = request_builder.send().await?;
        Ok(response)
    }

    // For testing only
    pub fn write_data(&self, data: Value) -> Result<String, Error> {
        let data = serde_json::to_string(&data)?;
        let report_file = std::path::Path::new(REPORT_PATH);
        if let Some(path) = report_file.parent() {
            match fs::create_dir_all(path) {
                Ok(_) => {}
                Err(err) => {
                    anyhow!("{:?}", err);
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
