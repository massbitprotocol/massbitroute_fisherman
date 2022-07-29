mod http_request;
mod websocket_request;
use common::tasks::eth::benchmark::executor::BenchmarkExecutor;
// use common::tasks::eth::latest_block::executor::LatestBlockExecutor;
use common::tasks::executor::TaskExecutor;
use common::tasks::ping::executor::PingExecutor;
use common::WorkerId;
pub use http_request::*;
use std::sync::Arc;
pub use websocket_request::*;

pub fn get_executors(worker_id: WorkerId, benchmark_wrk_path: &str) -> Vec<Arc<dyn TaskExecutor>> {
    let result: Vec<Arc<dyn TaskExecutor>> = vec![
        Arc::new(HttpRequestExecutor::new(worker_id.clone())),
        Arc::new(PingExecutor::new(worker_id.clone())),
        Arc::new(BenchmarkExecutor::new(
            worker_id.clone(),
            benchmark_wrk_path,
        )),
        Arc::new(WebsocketRequestExecutor::new(worker_id.clone())),
    ];
    result
}
