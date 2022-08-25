use crate::server_builder::SimpleResponse;
use crate::service::ProcessorService;
use crate::state::ProcessorState;
use crate::SCHEDULER_AUTHORIZATION;
use common::jobs::JobResult;
use common::task_spawn::spawn;
use log::info;
use std::convert::Infallible;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::Instant;
use warp::{http::StatusCode, reject, Rejection, Reply};

#[derive(Debug)]
pub struct UnAuthorization;
impl reject::Reject for UnAuthorization {}

pub static PROCESS_THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);
pub async fn handle_route_reports(
    service: Arc<ProcessorService>,
    state: Arc<ProcessorState>,
    job_results: Vec<JobResult>,
    authorization: String,
) -> Result<impl Reply, warp::Rejection> {
    info!(
        "#### Received {:?} reports request body  ####",
        &job_results.len()
    );
    if authorization == *SCHEDULER_AUTHORIZATION {
        let clone_service = service.clone();
        let clone_state = state.clone();
        spawn(async move {
            PROCESS_THREAD_COUNT.fetch_add(1, Ordering::Relaxed);
            let job_results_len = job_results.len();
            let now = Instant::now();
            let thread_id = PROCESS_THREAD_COUNT.load(Ordering::Relaxed);
            info!(
                "** Start {}th thread to process {} job results **",
                thread_id, job_results_len
            );
            let res = clone_service.process_report(job_results, clone_state).await;
            info!(
                "** Finished {}th thread to process {} job results in {:.2?} with res: {:?} **",
                thread_id,
                job_results_len,
                now.elapsed(),
                res
            );
            PROCESS_THREAD_COUNT.fetch_sub(1, Ordering::Relaxed);
        });

        Ok(warp::reply::json(&SimpleResponse { success: true }))
    } else {
        Err(warp::reject::custom(UnAuthorization))
    }
}
pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (StatusCode::BAD_REQUEST, "Payload too large".to_string())
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (
            StatusCode::BAD_REQUEST,
            format!("Method Not Allowed error, {:?}", err),
        )
    } else if err.find::<UnAuthorization>().is_some() {
        (
            StatusCode::UNAUTHORIZED,
            format!("Authorization error, {:?}", err),
        )
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}
