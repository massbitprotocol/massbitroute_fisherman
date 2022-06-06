use crate::report_processors::adapters::csv_appender::CsvAppender;
use serde_json::{Map, Value};
use std::sync::Arc;

pub mod csv_appender;

pub trait Appender: Sync + Send {
    fn append(&self, channel: String, report: &Map<String, Value>) -> Result<(), anyhow::Error>;
}

pub fn get_report_adapters() -> Vec<Arc<dyn Appender>> {
    let mut result: Vec<Arc<dyn Appender>> = Default::default();
    result.push(Arc::new(CsvAppender::new()));
    result
}
