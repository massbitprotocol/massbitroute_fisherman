use crate::models::job_result::StoredJobResult;
use crate::models::job_result_cache::JobResultCache;
use crate::report_processors::{get_report_processors, ReportProcessor};
use common::job_manage::JobResultDetail;
use common::jobs::JobResult;
use diesel::PgArrayExpressionMethods;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct ProcessorState {
    connection: Arc<DatabaseConnection>,
    processors: Vec<Arc<dyn ReportProcessor>>,
}

impl ProcessorState {
    pub fn new(
        connection: Arc<DatabaseConnection>,
        result_cache: Arc<Mutex<JobResultCache>>,
    ) -> ProcessorState {
        let processors = get_report_processors(connection.clone(), result_cache);
        ProcessorState {
            connection,
            processors,
        }
    }
    pub async fn process_results(
        &mut self,
        job_results: &Vec<JobResult>,
    ) -> Result<HashMap<String, StoredJobResult>, anyhow::Error> {
        let mut map_processor_reports = HashMap::<usize, Vec<JobResultDetail>>::new();
        //Group result by processor then process result by list
        for report in job_results.iter() {
            /// add process for Regular job
            for (ind, processor) in self.processors.iter().enumerate() {
                if processor.can_apply(&report.result_detail) {
                    if let Some(mut list) = map_processor_reports.get_mut(&ind) {
                        list.push(report.result_detail.clone())
                    } else {
                        map_processor_reports
                            .insert(ind.clone(), vec![report.result_detail.clone()]);
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
