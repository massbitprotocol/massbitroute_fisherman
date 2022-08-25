extern crate core;

pub mod models;
pub mod server_builder;
pub mod server_config;
pub mod services;
pub mod state;
pub mod tasks;

use lazy_static::lazy_static;
use std::env;
use uuid::Uuid;
//pub const CONFIG_FILE: &str = "config_check_component.json";
pub const JOB_EXECUTOR_PERIOD: u64 = 1000; //In milliseconds
                                           //pub const JOB_RESULT_REPORTER_PERIOD: u64 = 2000; //In milliseconds

lazy_static! {
    pub static ref SCHEDULER_ENDPOINT: String = env::var("SCHEDULER_ENDPOINT")
        .expect("There is no env var SCHEDULER_ENDPOINT");
    pub static ref WORKER_ID: String =
        env::var("WORKER_ID").unwrap_or(Uuid::new_v4().to_string());
    pub static ref ZONE: String = env::var("ZONE")
        .unwrap()
        .trim_start_matches("\"").trim_end_matches("\"").to_uppercase();
    pub static ref ENVIRONMENT: String = env::var("ENVIRONMENT").unwrap_or(String::from("unset"));
    //Endpoint for scheduler push jobs
    pub static ref WORKER_ENDPOINT: String =
        env::var("WORKER_ENDPOINT").unwrap_or(String::from("http://127.0.0.1:4040"));
    //For init webservice
    pub static ref WORKER_SERVICE_ENDPOINT: String =
        env::var("WORKER_SERVICE_ENDPOINT").unwrap_or(String::from("127.0.0.1:4040"));
    pub static ref WORKER_IP: String =
        env::var("WORKER_IP").unwrap_or(String::from("127.0.0.1"));
    pub static ref MAX_THREAD_COUNTER: usize = env::var("MAX_THREAD_COUNTER").ok().and_then(|val|{ val.parse::<usize>().ok()}).unwrap_or(4);
    //Waiting time period for parallelable threads in milliseconds
    pub static ref WAITING_TIME_FOR_EXECUTING_THREAD: u64 = env::var("WAITING_TIME_FOR_EXECUTING_THREAD").ok().and_then(|val|{ val.parse::<u64>().ok()}).unwrap_or(100);
        //.unwrap_or(Strin"4").parse::<usize>().unwrap();
    pub static ref JOB_RESULT_REPORTER_PERIOD: u64 = env::var("JOB_RESULT_REPORTER_PERIOD").ok().and_then(|val|{ val.parse::<u64>().ok()}).unwrap_or(2000);
    pub static ref LOCAL_IP: String = local_ip_address::local_ip().unwrap().to_string();
    pub static ref HASH_TEST_20K: String = "95c5679435a0a714918dc92b546dc0ba".to_string();
    //pub(crate) static ref CONFIG: Config = get_config();
    pub static ref DOMAIN: String = env::var("DOMAIN").expect("There is no env var DOMAIN");
    pub static ref BENCHMARK_WRK_PATH: String =
        env::var("BENCHMARK_WRK_PATH").expect("There is no env var BENCHMARK_WRK_PATH");
    pub static ref SCHEDULER_AUTHORIZATION: String =
        env::var("SCHEDULER_AUTHORIZATION").expect("There is no env var SCHEDULER_AUTHORIZATION");
    pub static ref BUILD_VERSION: String = format!("{}", env!("BUILD_VERSION"));
}
