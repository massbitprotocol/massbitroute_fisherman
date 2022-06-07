use tokio::task::JoinHandle;

pub struct WorkerThreadPool {
    runners: Vec<Runner>,
}

impl WorkerThreadPool {
    pub fn new(size: usize) -> WorkerThreadPool {
        assert!(size > 0);
        let mut runners = Vec::with_capacity(size);

        for id in 0..size {
            runners.push(Runner::new(id));
        }
        WorkerThreadPool { runners }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
    }
}

struct Runner {
    id: usize,
    thread: JoinHandle<()>,
}

// impl Runner {
//     fn new(id: usize) -> Runner {
//         let thread = tokio::spawn(|| {});
//
//         Runner { id, thread }
//     }
// }
