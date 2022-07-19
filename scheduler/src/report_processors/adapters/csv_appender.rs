use crate::report_processors::adapters::Appender;
use anyhow::Error;
use async_trait::async_trait;
use common::job_manage::JobBenchmarkResult;
use common::tasks::ping::JobPingResult;
use serde_json::{Map, Value};
pub struct CsvAppender {}

impl CsvAppender {
    pub fn new() -> Self {
        CsvAppender {}
    }
}
#[async_trait]
impl Appender for CsvAppender {
    fn get_name(&self) -> String {
        "CsvAppender".to_string()
    }
    async fn append(&self, _channel: String, _report: &Map<String, Value>) -> Result<(), Error> {
        Ok(())
    }
    async fn append_ping_results(
        &self,
        _results: &Vec<JobPingResult>,
    ) -> Result<(), anyhow::Error> {
        log::debug!("Csv append ping results");
        Ok(())
    }
    // async fn append_latest_block_results(
    //     &self,
    //     _result: &Vec<JobLatestBlockResult>,
    // ) -> Result<(), anyhow::Error> {
    //     log::debug!("Csv append lastest block results");
    //     Ok(())
    // }
    async fn append_benchmark_results(
        &self,
        _result: &Vec<JobBenchmarkResult>,
    ) -> Result<(), anyhow::Error> {
        log::debug!("Csv append benchmark results");
        Ok(())
    }
}
