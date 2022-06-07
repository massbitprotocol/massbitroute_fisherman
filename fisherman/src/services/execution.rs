use crate::models::job::JobBuffer;
use crate::{JOB_EXECUTOR_PERIOD, MAX_THREAD_COUNTER};
use common::job_manage::{Job, JobResult};
use common::tasks::executor::TaskExecutor;
use common::tasks::get_executors;
use log::{debug, info};
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
    rt_handle: Handle,
}

impl JobExecution {
    pub fn new(result_sender: Sender<JobResult>, job_buffers: Arc<Mutex<JobBuffer>>) -> Self {
        let executors = get_executors();
        let (job_sender, mut job_receiver): (Sender<Job>, Receiver<Job>) = channel(1024);
        //let parallelable_excutor = ParallelableExecutor::new(*MAX_THREAD_COUNTER);
        let runtime = Builder::new_multi_thread()
            .worker_threads(pool_size)
            .build()
            .unwrap();
        let rt_handle = runtime.handle();
        JobExecution {
            result_sender,
            job_sender,
            job_receiver,
            job_buffers,
            executors,
            runtime,
            rt_handle: runtime.hande(),
        }
    }
    pub async fn run(&mut self) {
        //main thread
        loop {
            while let Some(next_job) = self.job_buffers.lock().await.pop_job() {
                info!("Execute job: {:?}", job);
                if next_job.parallelable {
                    let rt_handle = self.rt_handle.clone();
                    for executor in self.executors.iter() {
                        let result_sender = self.result_sender.clone();
                        let job_sender = self.job_sender.clone();
                        let clone_executor = executor.clone();
                        rt_handle.spawn(async move {
                            debug!("Execute job on a worker thread");
                            clone_executor
                                .execute(&job, result_sender, job_sender)
                                .await;
                        });
                    }
                } else {
                    for executor in self.executors.iter() {
                        let result_sender = self.result_sender.clone();
                        let job_sender = self.job_sender.clone();
                        debug!("Execute job o main execution thread");
                        executor.execute(&job, sender, job_sender).await;
                    }
                }
            }
            //Get new generated jobs
            self.get_jobs_from_executions().await;
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
/*
 * for execution parallel jobs
 */
pub struct ParallelableExecutor {
    runtime: Runtime,
}

impl ParallelableExecutor {
    pub fn new(pool_size: usize) -> Self {
        ParallelableExecutor {
            runtime: Builder::new_multi_thread()
                .worker_threads(pool_size)
                .build()
                .unwrap()?,
        }
    }
    pub fn start(&self) {}
}
