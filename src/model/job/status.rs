use std::fmt;

use redis::{ErrorKind, FromRedisValue, RedisResult, RedisWrite, ToRedisArgs, Value};

use crate::model::error::RsrqError;
use crate::model::queue::queue_type::QueueType;
use crate::model::types::RsrqResult;

#[derive(Debug)]
pub enum JobStatus {
    Queued,
    Running,
    Finished,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn from_string(s: &str) -> RsrqResult<JobStatus> {
        match s {
            "queued" => Ok(JobStatus::Queued),
            "running" => Ok(JobStatus::Running),
            "finished" => Ok(JobStatus::Finished),
            "failed" => Ok(JobStatus::Failed),
            "cancelled" => Ok(JobStatus::Cancelled),
            _ => Err(RsrqError::CmdParserError(format!("Invalid job status: {}", s))),
        }
    }

    pub fn to_queue_type(&self) -> QueueType {
        match self {
            JobStatus::Queued => QueueType::Queued,
            JobStatus::Running => QueueType::Running,
            JobStatus::Finished => QueueType::Finished,
            JobStatus::Failed => QueueType::Failed,
            JobStatus::Cancelled => QueueType::Failed,
        }
    }
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JobStatus::Queued => write!(f, "queued"),
            JobStatus::Running => write!(f, "running"),
            JobStatus::Finished => write!(f, "finished"),
            JobStatus::Failed => write!(f, "failed"),
            JobStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl ToRedisArgs for JobStatus {
    fn write_redis_args<W>(&self, out: &mut W) where W: ?Sized + RedisWrite {
        ToRedisArgs::write_redis_args(&self.to_string(), out);
    }
}

impl FromRedisValue for JobStatus {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let string_value: String = FromRedisValue::from_redis_value(v)?;
        let res = JobStatus::from_string(&string_value);
        match res {
            Ok(job_key) => Ok(job_key),
            Err(_) => Err((ErrorKind::TypeError, "Unable to convert value.").into())
        }
    }
}
