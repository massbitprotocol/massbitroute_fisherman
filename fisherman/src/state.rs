use common::job_manage::Job;
use tokio::sync::mpsc::Sender;

pub struct FishermanState {}

impl FishermanState {
    pub(crate) fn new() -> Self {
        FishermanState {}
    }
}
