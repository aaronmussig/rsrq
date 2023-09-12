use std::fmt;

use redis::{ErrorKind, FromRedisValue, RedisResult, RedisWrite, ToRedisArgs, Value};

use crate::model::error::RsrqError;
use crate::model::types::RsrqResult;

#[derive(Debug)]
pub enum ProcessKey {
    Id,
    Hostname,
    Pid,
    Birth,
    LastHeartbeat,
    State,
    NumRunning,
    Queue,
    Workers,
    MaxDurationSec,
    MaxJobs,
    Burst,
    PollMs,
}

impl ProcessKey {
    pub fn from_string(value: &str) -> RsrqResult<ProcessKey> {
        match value {
            "id" => Ok(ProcessKey::Id),
            "hostname" => Ok(ProcessKey::Hostname),
            "pid" => Ok(ProcessKey::Pid),
            "birth" => Ok(ProcessKey::Birth),
            "last_heartbeat" => Ok(ProcessKey::LastHeartbeat),
            "state" => Ok(ProcessKey::State),
            "num_running" => Ok(ProcessKey::NumRunning),
            "queue" => Ok(ProcessKey::Queue),
            "workers" => Ok(ProcessKey::Workers),
            "max_duration_sec" => Ok(ProcessKey::MaxDurationSec),
            "max_jobs" => Ok(ProcessKey::MaxJobs),
            "burst" => Ok(ProcessKey::Burst),
            "poll_ms" => Ok(ProcessKey::PollMs),
            _ => Err(RsrqError::ParserError(value.to_string())),
        }
    }
}

impl fmt::Display for ProcessKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessKey::Id => write!(f, "id"),
            ProcessKey::Hostname => write!(f, "hostname"),
            ProcessKey::Pid => write!(f, "pid"),
            ProcessKey::Birth => write!(f, "birth"),
            ProcessKey::LastHeartbeat => write!(f, "last_heartbeat"),
            ProcessKey::State => write!(f, "state"),
            ProcessKey::NumRunning => write!(f, "num_running"),
            ProcessKey::Queue => write!(f, "queue"),
            ProcessKey::Workers => write!(f, "workers"),
            ProcessKey::MaxDurationSec => write!(f, "max_duration_sec"),
            ProcessKey::MaxJobs => write!(f, "max_jobs"),
            ProcessKey::Burst => write!(f, "burst"),
            ProcessKey::PollMs => write!(f, "poll_ms"),
        }
    }
}

impl ToRedisArgs for ProcessKey {
    fn write_redis_args<W>(&self, out: &mut W) where W: ?Sized + RedisWrite {
        ToRedisArgs::write_redis_args(&self.to_string(), out);
    }
}

impl FromRedisValue for ProcessKey {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let string_value: String = FromRedisValue::from_redis_value(v)?;
        let res = ProcessKey::from_string(&string_value);
        match res {
            Ok(job_key) => Ok(job_key),
            Err(_) => Err((ErrorKind::TypeError, "Unable to convert value.").into())
        }
    }
}
