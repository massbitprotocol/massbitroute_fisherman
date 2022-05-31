use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    //Protal url for getting nodes/gateqay
    pub url_list_nodes: String,
    pub url_list_gateways: String,
    pub response_time_key_name: String,
    pub number_of_samples: u64,
    pub sample_interval_ms: u64,
    pub delay_between_check_loop_ms: u64,
    pub success_percent_threshold: u32,
    pub node_response_time_threshold: u32,
    pub gateway_response_time_threshold: u32,
    pub node_response_failed_number: i32,
    pub gateway_response_failed_number: i32,
    pub reports_history_queue_length_max: usize,
    pub check_task_list_fisherman: Vec<String>,
    pub checking_component_status: String,

    // for submit report
    pub mvp_extrinsic_submit_provider_report: String,
    pub mvp_extrinsic_dapi: String,
    pub mvp_extrinsic_submit_project_usage: String,
    pub mvp_event_project_registered: String,

    // for ping pong check
    pub ping_parallel_requests: usize,
    pub ping_success_ratio_threshold: f32,
    pub ping_sample_number: u64,
    pub ping_request_response: String,
    pub ping_timeout_ms: u64,

    pub check_ping_pong_interval: u64,
    pub check_logic_interval: u64,
    pub check_benchmark_interval: u64,
    pub update_provider_list_interval: u64,
}

impl Config {
    pub fn load(file_path: &str) -> Self {
        let json = std::fs::read_to_string(file_path)
            .unwrap_or_else(|err| panic!("Unable to read config file `{}`: {}", file_path, err));
        let config: Config = serde_json::from_str(&*json).unwrap();
        config
    }
}
#[derive(Debug, Clone)]
pub struct AccessControl {
    pub access_control_allow_headers: String,
    pub access_control_allow_origin: String,
    pub access_control_allow_methods: String,
    pub content_type: String,
}

impl Default for AccessControl {
    fn default() -> Self {
        AccessControl {
            access_control_allow_headers:
                "Content-Type, User-Agent, Authorization, Access-Control-Allow-Origin".to_string(),
            access_control_allow_origin: "*".to_string(),
            access_control_allow_methods: "text/html".to_string(),
            content_type: "application/json".to_string(),
        }
    }
}

impl AccessControl {
    pub fn get_access_control_allow_headers(&self) -> Vec<String> {
        self.access_control_allow_headers
            .split(",")
            .into_iter()
            .map(|header| header.replace(" ", ""))
            .collect()
    }
}
