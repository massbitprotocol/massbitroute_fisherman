use crate::report_processors::adapters::Appender;
use anyhow::Error;
use serde_json::{Map, Value};

pub struct CsvAppender {}

impl CsvAppender {
    pub fn new() -> Self {
        CsvAppender {}
    }
}
impl Appender for CsvAppender {
    fn append(&self, channel: String, report: &Map<String, Value>) -> Result<(), Error> {
        todo!()
    }
}
