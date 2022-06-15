use anyhow::Error;
use common::component::{ComponentInfo, ComponentType};
use common::job_manage::JobRole;
use common::{ComponentId, Deserialize, Serialize};
use log::debug;
use reqwest::Response;
use std::time::{SystemTime, UNIX_EPOCH};

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
        reporter_role: JobRole,
        authorization: &String,
        domain: &String,
    ) -> StoreReport {
        StoreReport {
            reporter: reporter.clone(),
            reporter_role,
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

    fn get_url(&self, job_role: JobRole) -> String {
        match job_role {
            JobRole::Verification => {
                format!(
                    "https://portal.{}/mbr/verify/{}",
                    self.domain, self.provider_id
                )
            }
            JobRole::Regular => {
                format!(
                    "https://portal.{}/mbr/benchmark/{}",
                    self.domain, self.provider_id
                )
            }
        }
    }

    pub async fn send_data(&self, send_purpose: JobRole) -> Result<Response, Error> {
        let client_builder = reqwest::ClientBuilder::new();
        let client = client_builder.danger_accept_invalid_certs(true).build()?;
        // create body
        let body = self.create_body()?;
        debug!("body send_data: {:?}", body);
        // get url
        let url = self.get_url(send_purpose);

        let request_builder = client
            .post(url)
            .header("content-type", "application/json")
            .header("Authorization", &self.authorization)
            .body(body);
        debug!("request_builder: {:?}", request_builder);
        let response = request_builder.send().await?;
        Ok(response)
    }
}
