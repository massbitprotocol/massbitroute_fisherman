use crate::models::job_result::StoredJobResult;
use crate::report_processors::{get_report_processors, ReportProcessor};
use common::job_manage::JobResult;
use diesel::PgArrayExpressionMethods;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct ProcessorState {
    connection: Arc<DatabaseConnection>,
    processors: Vec<Arc<dyn ReportProcessor>>,
}

impl ProcessorState {
    pub fn new(connection: Arc<DatabaseConnection>) -> ProcessorState {
        let processors = get_report_processors(connection.clone());
        ProcessorState {
            connection,
            processors,
        }
    }
    pub async fn process_results(
        &mut self,
        job_results: &Vec<JobResult>,
    ) -> Result<HashMap<String, StoredJobResult>, anyhow::Error> {
        let mut map_processor_reports = HashMap::<usize, Vec<JobResult>>::new();
        //Group result by processor then process result by list
        for report in job_results.iter() {
            for (ind, processor) in self.processors.iter().enumerate() {
                if processor.can_apply(&report) {
                    if let Some(mut list) = map_processor_reports.get_mut(&ind) {
                        list.push(report.clone())
                    } else {
                        map_processor_reports.insert(ind.clone(), vec![report.clone()]);
                    }
                }
            }
        }
        let mut stored_results = HashMap::<String, StoredJobResult>::new();
        for (ind, jobs) in map_processor_reports {
            let connection = self.connection.clone();
            if let Some(processor) = self.processors.get(ind) {
                processor.process_jobs(jobs, connection).await;
            };
        }
        Ok(stored_results)
    }
}
