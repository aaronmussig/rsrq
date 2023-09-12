use std::fmt;

use crate::config::{Q_FAILED, Q_FINISHED, Q_QUEUED, Q_RUNNING};
use crate::model::error::RsrqError;
use crate::model::types::RsrqResult;

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

    pub fn from_string(string: &str) -> RsrqResult<QueueType> {
        match string {
            Q_QUEUED => Ok(QueueType::Queued),
            Q_RUNNING => Ok(QueueType::Running),
            Q_FINISHED => Ok(QueueType::Finished),
            Q_FAILED => Ok(QueueType::Failed),
            _ => Err(RsrqError::ParserError(format!("Invalid queue type: {}", string))),
        }
    }

    pub fn get_types() -> Vec<QueueType> {
        vec![
            QueueType::Queued,
            QueueType::Running,
            QueueType::Finished,
            QueueType::Failed,
        ]
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
