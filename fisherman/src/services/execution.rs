use crate::models::job::JobBuffer;
use crate::{
    BENCHMARK_WRK_PATH, JOB_EXECUTOR_PERIOD, MAX_THREAD_COUNTER, WAITING_TIME_FOR_EXECUTING_THREAD,
    WORKER_ID,
};
use common::job_manage::{Job, JobResult};
use common::tasks::executor::TaskExecutor;
use common::tasks::get_executors;
use common::util::get_current_time;
use log::{debug, info};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::runtime::{Builder, Handle, Runtime};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
/*
 * For repeated jobs, after execution executor generate new job with new parameters and push back to JobBuffer
 */
pub struct JobExecution {
    result_sender: Sender<JobResult>,
    job_sender: Sender<Job>,
    job_receiver: Receiver<Job>,
    job_buffers: Arc<Mutex<JobBuffer>>,
    executors: Vec<Arc<dyn TaskExecutor>>,
    //Runtime for parallelable jobs
    runtime: Runtime,
    thread_counter: Arc<AtomicUsize>,
}

impl JobExecution {
    pub fn new(result_sender: Sender<JobResult>, job_buffers: Arc<Mutex<JobBuffer>>) -> Self {
        let executors = get_executors(WORKER_ID.as_str().to_string(), BENCHMARK_WRK_PATH.as_str());
        let (job_sender, mut job_receiver): (Sender<Job>, Receiver<Job>) = channel(1024);

        let runtime = Builder::new_multi_thread()
            .worker_threads(*MAX_THREAD_COUNTER)
            .enable_time()
            .enable_io()
            .build()
            .unwrap();
        JobExecution {
            result_sender,
            job_sender,
            job_receiver,
            job_buffers,
            executors,
            runtime,
            thread_counter: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub async fn run(&mut self) {
        //main thread
        loop {
            while let Some(next_job) = self.job_buffers.lock().await.pop_job() {
                info!("Execute job: {:?}", &next_job);
                let rt_handle = self.runtime.handle();

                if next_job.parallelable {
                    for executor in self.executors.iter() {
                        if !executor.can_apply(&next_job) {
                            continue;
                        }
                        let result_sender = self.result_sender.clone();
                        let job_sender = self.job_sender.clone();
                        let clone_executor = executor.clone();
                        let clone_job = next_job.clone();
                        let counter = self.thread_counter.clone();
                        counter.fetch_add(1, Ordering::SeqCst);
                        rt_handle.spawn(async move {
                            debug!("Execute job on a worker thread");
                            clone_executor.execute(&clone_job, result_sender).await;
                            //Fixme: Program will hang if it panic before fetch_sub is executed.
                            counter.fetch_sub(1, Ordering::SeqCst);
                        });
                    }
                } else {
                    //wait until all task in  parallelable runtime pool is terminated
                    while self.thread_counter.load(Ordering::Relaxed) > 0 {
                        sleep(Duration::from_millis(*WAITING_TIME_FOR_EXECUTING_THREAD))
                    }
                    for executor in self.executors.iter() {
                        let can_apply = executor.can_apply(&next_job);
                        if !can_apply {
                            continue;
                        }
                        let result_sender = self.result_sender.clone();
                        let job_sender = self.job_sender.clone();
                        debug!("Execute job on main execution thread");
                        executor.execute(&next_job, result_sender).await;
                    }
                }
            }
            //Jun 13 - Don't use this anymore
            //Get new generated jobs
            //self.get_jobs_from_executions().await;
            sleep(Duration::from_millis(JOB_EXECUTOR_PERIOD));
        }
    }
    pub async fn get_jobs_from_executions(&mut self) {
        let mut jobs = Vec::new();
        while let Ok(job) = self.job_receiver.try_recv() {
            debug!("Received job: {:?} from executors", &job);
            jobs.push(job);
        }
        self.job_buffers.lock().await.add_jobs(jobs);
    }
}
