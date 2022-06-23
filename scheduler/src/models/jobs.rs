use common::component::Zone;
use common::job_manage::JobResultDetail;
use common::jobs::JobAssignment;
use common::workers::Worker;
use common::workers::WorkerInfo;
use common::{JobId, WorkerId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AssignmentBuffer {
    list_assignments: Vec<JobAssignment>,
}
impl Default for AssignmentBuffer {
    fn default() -> Self {
        AssignmentBuffer {
            list_assignments: vec![],
        }
    }
}

impl AssignmentBuffer {
    pub fn add_assignments(&mut self, mut assignments: Vec<JobAssignment>) {
        self.list_assignments.append(&mut assignments);
    }
    pub fn push_back(&mut self, mut assignments: Vec<JobAssignment>) {
        for job in assignments.into_iter().rev() {
            self.list_assignments.insert(0, job);
        }
    }
    pub fn pop_all(&mut self) -> Vec<JobAssignment> {
        let mut res = Vec::default();
        res.append(&mut self.list_assignments);
        res
    }
}
