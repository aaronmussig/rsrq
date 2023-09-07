use std::fmt;

use redis::{ErrorKind, FromRedisValue, RedisResult, RedisWrite, ToRedisArgs, Value};

use crate::model::error::RsrqError;
use crate::model::types::RsrqResult;

#[derive(Debug)]
pub enum JobKey {
    Id,
    Cmd,
    Status,
    Queue,
    Created,
    Started,
    Finished,
    Stdout,
    Stderr,
    ExitCode,
    DurationMs,
}

impl JobKey {
    pub fn from_string(value: &str) -> RsrqResult<JobKey> {
        match value {
            "id" => Ok(JobKey::Id),
            "cmd" => Ok(JobKey::Cmd),
            "status" => Ok(JobKey::Status),
            "queue" => Ok(JobKey::Queue),
            "created" => Ok(JobKey::Created),
            "started" => Ok(JobKey::Started),
            "finished" => Ok(JobKey::Finished),
            "stdout" => Ok(JobKey::Stdout),
            "stderr" => Ok(JobKey::Stderr),
            "exit_code" => Ok(JobKey::ExitCode),
            "duration_ms" => Ok(JobKey::DurationMs),
            _ => Err(RsrqError::ParserError(value.to_string())),
        }
    }
}

impl fmt::Display for JobKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JobKey::Id => write!(f, "id"),
            JobKey::Cmd => write!(f, "cmd"),
            JobKey::Status => write!(f, "status"),
            JobKey::Queue => write!(f, "queue"),
            JobKey::Created => write!(f, "created"),
            JobKey::Started => write!(f, "started"),
            JobKey::Finished => write!(f, "finished"),
            JobKey::Stdout => write!(f, "stdout"),
            JobKey::Stderr => write!(f, "stderr"),
            JobKey::ExitCode => write!(f, "exit_code"),
            JobKey::DurationMs => write!(f, "duration_ms"),
        }
    }
}

impl ToRedisArgs for JobKey {
    fn write_redis_args<W>(&self, out: &mut W) where W: ?Sized + RedisWrite {
        ToRedisArgs::write_redis_args(&self.to_string(), out);
    }
}

impl FromRedisValue for JobKey {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let string_value: String = FromRedisValue::from_redis_value(v)?;
        let res = JobKey::from_string(&string_value);
        match res {
            Ok(job_key) => Ok(job_key),
            Err(_) => Err((ErrorKind::TypeError, "Unable to convert value.").into())
        }
    }
}
