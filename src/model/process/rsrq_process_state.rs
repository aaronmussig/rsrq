use std::fmt;

use redis::{ErrorKind, FromRedisValue, RedisResult, RedisWrite, ToRedisArgs, Value};

use crate::model::error::RsrqError;
use crate::model::types::RsrqResult;

pub enum ProcessState {
    Idle,
    Running,
}


impl ProcessState {
    pub fn from_string(value: &str) -> RsrqResult<ProcessState> {
        match value {
            "idle" => Ok(ProcessState::Idle),
            "running" => Ok(ProcessState::Running),
            _ => Err(RsrqError::ParserError(value.to_string())),
        }
    }
}

impl fmt::Display for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessState::Idle => write!(f, "idle"),
            ProcessState::Running => write!(f, "running"),
        }
    }
}

impl ToRedisArgs for ProcessState {
    fn write_redis_args<W>(&self, out: &mut W) where W: ?Sized + RedisWrite {
        ToRedisArgs::write_redis_args(&self.to_string(), out);
    }
}

impl FromRedisValue for ProcessState {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let string_value: String = FromRedisValue::from_redis_value(v)?;
        let res = ProcessState::from_string(&string_value);
        match res {
            Ok(job_key) => Ok(job_key),
            Err(_) => Err((ErrorKind::TypeError, "Unable to convert value.").into())
        }
    }
}
