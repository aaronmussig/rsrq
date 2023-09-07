use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::config::{UID_KEY_WORKER, WORKER_KEY};
use crate::model::error::RsrqError;
use crate::model::types::RsrqResult;
use crate::model::worker::key::WorkerKey;
use crate::util::redis::get_next_uid;
use crate::util::system::{get_hostname, get_pid};
use crate::util::time::get_timestamp_s;

#[derive(Debug)]
pub struct Worker {
    pub id: usize,
    pub key: String,
    pub hostname: String,
    pub pid: u32,
    pub current_job: Option<usize>,
    pub current_job_start: Option<u64>,
    pub birth: u64,
    pub queue: String,
}

impl Worker {
    pub fn get_key(id: usize) -> String {
        format!("{}:{}", WORKER_KEY, id)
    }
    pub async fn new(queue: &str, con: &mut ConnectionManager) -> RsrqResult<Worker> {
        // Assign the next worker id to this worker
        let worker_id = get_next_uid(UID_KEY_WORKER, con).await?;

        // Get attributes for the worker
        let hostname = get_hostname();
        let pid = get_pid();
        let birth = get_timestamp_s()?;

        let worker = Worker {
            id: worker_id,
            key: Worker::get_key(worker_id),
            hostname,
            pid,
            current_job: None,
            current_job_start: None,
            birth,
            queue: queue.to_string(),
        };

        // Update Redis
        worker.push(con).await?;

        // Return the worker
        Ok(worker)
    }

    pub fn to_array(&self) -> [(WorkerKey, String); 7] {
        // Parse the optional attributes
        let current_job = match self.current_job {
            Some(job_id) => job_id.to_string(),
            None => "".to_string()
        };
        let current_job_start = match self.current_job_start {
            Some(job_start) => job_start.to_string(),
            None => "".to_string()
        };
        [
            (WorkerKey::Id, self.id.to_string()),
            (WorkerKey::Hostname, self.hostname.clone()),
            (WorkerKey::Pid, self.pid.to_string()),
            (WorkerKey::CurrentJob, current_job),
            (WorkerKey::CurrentJobStart, current_job_start),
            (WorkerKey::Birth, self.birth.to_string()),
            (WorkerKey::Queue, self.queue.clone()),
        ]
    }

    pub async fn push(&self, con: &mut ConnectionManager) -> RsrqResult<()> {
        let arr = self.to_array();
        con.hset_multiple(&self.key, &arr).await.map_err(RsrqError::RedisOpError)?;
        Ok(())
    }

    pub async fn delete(&self, con: &mut ConnectionManager) -> RsrqResult<()> {
        con.del(&self.key).await.map_err(RsrqError::RedisOpError)?;
        Ok(())
    }
}