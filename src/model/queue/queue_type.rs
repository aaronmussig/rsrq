use std::fmt;

use crate::config::{Q_FAILED, Q_FINISHED, Q_QUEUED, Q_RUNNING};

#[derive(Debug)]
pub enum QueueType {
    Queued,
    Running,
    Finished,
    Failed,
}

impl QueueType {
    pub fn to_string(&self) -> &str {
        match self {
            QueueType::Queued => Q_QUEUED,
            QueueType::Running => Q_RUNNING,
            QueueType::Finished => Q_FINISHED,
            QueueType::Failed => Q_FAILED,
        }
    }
}


impl fmt::Display for QueueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QueueType::Queued => write!(f, "{}", Q_QUEUED),
            QueueType::Running => write!(f, "{}", Q_RUNNING),
            QueueType::Finished => write!(f, "{}", Q_FINISHED),
            QueueType::Failed => write!(f, "{}", Q_FAILED),
        }
    }
}
