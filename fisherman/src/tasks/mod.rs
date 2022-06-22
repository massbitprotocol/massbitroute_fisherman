mod http_request;

use common::tasks::eth::benchmark::executor::BenchmarkExecutor;
use common::tasks::eth::latest_block::executor::LatestBlockExecutor;
use common::tasks::executor::TaskExecutor;
use common::tasks::ping::executor::PingExecutor;
use common::WorkerId;
pub use http_request::*;
use std::sync::Arc;

pub fn get_executors(worker_id: WorkerId, benchmark_wrk_path: &str) -> Vec<Arc<dyn TaskExecutor>> {
    let mut result: Vec<Arc<dyn TaskExecutor>> = Default::default();
    result.push(Arc::new(HttpRequestExecutor::new(worker_id.clone())));
    result.push(Arc::new(PingExecutor::new(worker_id.clone())));
    result.push(Arc::new(BenchmarkExecutor::new(
        worker_id.clone(),
        benchmark_wrk_path,
    )));
    result.push(Arc::new(LatestBlockExecutor::new(worker_id.clone())));
    result
}
