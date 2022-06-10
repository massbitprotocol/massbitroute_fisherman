use crate::persistence::services::job_result_service::JobResultService;
use crate::service::judgment::{get_report_judgments, ReportCheck};
use crate::JUDGMENT_PERIOD;
use anyhow::Error;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Judgment {
    result_service: Arc<JobResultService>,
    judgments: Vec<Arc<dyn ReportCheck>>,
}

impl Judgment {
    pub fn new(connection: Arc<DatabaseConnection>) -> Self {
        let result_service = Arc::new(JobResultService::new(connection));
        let judgments = get_report_judgments(result_service.clone());
        Judgment {
            result_service,
            judgments,
        }
    }
    pub async fn run(&mut self) {
        loop {
            let now = Instant::now();
            for judg in self.judgments.iter() {
                match judg.apply().await {
                    Ok(res) => {}
                    Err(_) => {}
                }
            }
            if now.elapsed().as_secs() < JUDGMENT_PERIOD {
                sleep(Duration::from_secs(
                    JUDGMENT_PERIOD - now.elapsed().as_secs(),
                ));
            }
        }
    }
}
