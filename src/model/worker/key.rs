use std::fmt;

use redis::{RedisWrite, ToRedisArgs};

#[derive(Debug)]
pub enum WorkerKey {
    Id,
    Hostname,
    Pid,
    CurrentJob,
    CurrentJobStart,
    Birth,
    Queue,
}

impl fmt::Display for WorkerKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WorkerKey::Id => write!(f, "id"),
            WorkerKey::Hostname => write!(f, "hostname"),
            WorkerKey::Pid => write!(f, "pid"),
            WorkerKey::CurrentJob => write!(f, "current_job"),
            WorkerKey::CurrentJobStart => write!(f, "current_job_start"),
            WorkerKey::Birth => write!(f, "birth"),
            WorkerKey::Queue => write!(f, "queue"),
        }
    }
}

impl ToRedisArgs for WorkerKey {
    fn write_redis_args<W>(&self, out: &mut W) where W: ?Sized + RedisWrite {
        ToRedisArgs::write_redis_args(&self.to_string(), out);
    }
}
