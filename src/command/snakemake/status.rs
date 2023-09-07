use std::fmt;
use redis::Commands;

use crate::model::error::RsrqError;
use crate::model::job::rsrq_job::Job;
use crate::model::job::key::JobKey;
use crate::model::job::status::JobStatus;
use crate::model::types::RsrqResult;
use crate::util::redis::redis_conn;

pub enum SnakemakeStatus {
    Running,
    Failed,
    Success,
}

impl SnakemakeStatus {
    pub fn from_job_status(status: JobStatus) -> SnakemakeStatus {
        match status {
            JobStatus::Queued => SnakemakeStatus::Running,
            JobStatus::Running => SnakemakeStatus::Running,
            JobStatus::Finished => SnakemakeStatus::Success,
            JobStatus::Failed => SnakemakeStatus::Failed,
            JobStatus::Cancelled => SnakemakeStatus::Failed,
        }
    }
}

impl fmt::Display for SnakemakeStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SnakemakeStatus::Running => write!(f, "running"),
            SnakemakeStatus::Failed => write!(f, "failed"),
            SnakemakeStatus::Success => write!(f, "success"),
        }
    }
}


pub fn snakemake_status(job_id: usize) -> RsrqResult<SnakemakeStatus> {
    let mut con = redis_conn()?;

    let job_key = Job::get_redis_key(job_id);
    let result: Option<JobStatus> = con.hget(job_key, JobKey::Status).map_err(RsrqError::RedisOpError)?;

    if let Some(status) = result {
        let snakemake_status = SnakemakeStatus::from_job_status(status);
        println!("{}", snakemake_status);
        Ok(snakemake_status)
    } else {
        Err(RsrqError::JobNotFound(job_id))
    }
}


