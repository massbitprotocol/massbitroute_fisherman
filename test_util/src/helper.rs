use common::job_manage::{JobBenchmark, JobDetail};
use common::jobs::Job;
use common::tasks::http_request::JobHttpRequest;
use common::ComponentInfo;

pub fn load_schedule_env() {
    dotenv::from_filename(".env_test").ok();
}

pub enum JobName {
    RoundTripTime,
    LatestBlock,
    Benchmark,
}

pub fn new_test_job(job_name: &JobName, component_url: &str) -> Job {
    let component = ComponentInfo {
        blockchain: "".to_string(),
        network: "".to_string(),
        id: "".to_string(),
        user_id: "".to_string(),
        ip: "".to_string(),
        zone: Default::default(),
        country_code: "".to_string(),
        token: "".to_string(),
        component_type: Default::default(),
        endpoint: None,
        status: "".to_string(),
    };

    let mut job = match job_name {
        JobName::Benchmark => {
            let job_detail = r###"
        {"rate": 50, "script": "massbit.lua", "thread": 20, "duration": 15000, "url_path": "/", "chain_type": "eth", "connection": 20, "histograms": [90, 95, 99, 100], "component_type": "Node"}
        "###;
            let job_detail: JobBenchmark = serde_json::from_str(job_detail).unwrap();
            Job::new(
                "benchmark".to_string(),
                "".to_string(),
                "".to_string(),
                &component,
                JobDetail::Benchmark(job_detail),
                Default::default(),
            )
        }
        JobName::RoundTripTime => {
            let job_detail = r###"
        {"url": "", "body": "", "method": "get", "headers": {}, "chain_info": {"chain": "eth", "network": "mainnet"}, "response_type": "text", "response_values": {}}
        "###;
            let mut job_http_request: JobHttpRequest = serde_json::from_str(job_detail).unwrap();
            job_http_request.url = component_url.to_string();
            Job::new(
                "http".to_string(),
                "HttpRequest".to_string(),
                "".to_string(),
                &component,
                JobDetail::HttpRequest(job_http_request),
                Default::default(),
            )
        }
        _ => Job::default(),
    };
    job.component_url = component_url.to_string();
    job
}
