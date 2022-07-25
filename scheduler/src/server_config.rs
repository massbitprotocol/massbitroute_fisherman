use common::Timestamp;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub checking_component_status: String,

    pub check_ping_pong_interval: Timestamp,
    pub check_logic_interval: Timestamp,
    pub check_benchmark_interval: Timestamp,
    pub update_provider_list_interval: Timestamp,
    pub regular_plan_generate_interval: Timestamp,
    pub generate_new_regular_timeout: Timestamp,
    pub plan_expiry_time: Timestamp, //Expiry time in second
    pub update_worker_list_interval: Timestamp,
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
