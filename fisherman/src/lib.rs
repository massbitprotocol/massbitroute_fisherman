pub mod models;
pub mod server_builder;
pub mod server_config;
pub mod services;
pub mod state;
use dotenv;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::env;

//pub const CONFIG_FILE: &str = "config_check_component.json";
pub const JOB_EXECUTOR_PERIOD: u64 = 10; //In seconds
pub const JOB_RESULT_REPORTER_PERIOD: u64 = 10; //In seconds
lazy_static! {
    pub static ref SCHEDULER_ENDPOINT: String = env::var("SCHEDULER_ENDPOINT")
        .unwrap_or(String::from("https://scheduler.massbitroute.net"));
    pub static ref WORKER_ID: String =
        env::var("WORKER_ID").unwrap_or(String::from("someworkerhash"));
    pub static ref ZONE: String = env::var("ZONE")
        .unwrap_or(String::from("AS"))
        .to_uppercase();
    pub static ref ENVIRONMENT: String = env::var("ENVIRONMENT").unwrap_or(String::from("local"));
    pub static ref FISHERMAN_ENDPOINT: String =
        env::var("FISHERMAN_ENDPOINT").unwrap_or(String::from("0.0.0.0:4040"));
    pub static ref CHECK_COMPONENT_ENDPOINT: String =
        env::var("CHECK_COMPONENT_ENDPOINT").unwrap_or(String::from("0.0.0.0:3030"));
    pub static ref BASE_ENDPOINT_JSON: String = env::var("BASE_ENDPOINT_JSON").unwrap();
    pub static ref BENCHMARK_WRK_PATH: String =
        env::var("BENCHMARK_WRK_PATH").unwrap_or("./".to_string());
    pub static ref PORTAL_AUTHORIZATION: String =
        env::var("PORTAL_AUTHORIZATION").expect("There is no env var PORTAL_AUTHORIZATION");
    pub static ref SIGNER_PHRASE: String =
        env::var("SIGNER_PHRASE").expect("There is no env var SIGNER_PHRASE");
    pub static ref LOCAL_IP: String = local_ip_address::local_ip().unwrap().to_string();
    pub static ref HASH_TEST_20K: String = "95c5679435a0a714918dc92b546dc0ba".to_string();
    //pub(crate) static ref CONFIG: Config = get_config();
    pub static ref DOMAIN: String = env::var("DOMAIN").expect("There is no env var DOMAIN");
}

// fn get_config() -> Config {
//     let json = std::fs::read_to_string(CONFIG_FILE)
//         .unwrap_or_else(|err| panic!("Unable to read config file `{}`: {}", CONFIG_FILE, err));
//     serde_json::from_str::<Config>(&*json).unwrap()
// }
