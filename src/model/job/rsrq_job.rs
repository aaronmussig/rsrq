use std::collections::BTreeMap;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::config::{JOB_KEY, UID_KEY_JOB};
use crate::model::error::RsrqError;
use crate::model::job::key::JobKey;
use crate::model::job::status::JobStatus;
use crate::model::queue::queue_type::QueueType;
use crate::model::queue::rsrq_queue::Queue;
use crate::model::types::RsrqResult;
use crate::util::parsing::{btree_get, btree_get_opt};
use crate::util::redis::get_next_uid;
use crate::util::time::get_timestamp_s;

#[derive(Debug)]
pub struct Job {
    pub key: String,

    // Attributes that are stored in redis
    pub id: usize,
    pub cmd: String,
    pub status: JobStatus,
    pub queue: String,
    pub created: u64,
    pub started: Option<u64>,
    pub finished: Option<u64>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u128>,
}

impl Job {
    /// Retrieves the formatted Redis key given a job identifier.
    pub fn get_redis_key(id: usize) -> String {
        format!("{}:{}", JOB_KEY, id)
    }

    pub async fn new<C: redis::aio::ConnectionLike>(queue: &str, cmd: &str, con: &mut C) -> RsrqResult<Job> {
        let job = {
            let id = get_next_uid(UID_KEY_JOB, con).await?;
            Job {
                id,
                key: Job::get_redis_key(id),
                cmd: cmd.to_string(),
                status: JobStatus::Queued,
                queue: queue.to_string(),
                created: get_timestamp_s()?,
                started: None,
                finished: None,
                stdout: None,
                stderr: None,
                exit_code: None,
                duration_ms: None,
            }
        };

        // Create the target queue object
        let queue = Queue::new(QueueType::Queued, queue);

        // Create the Job and Enqueue it
        let pipe = {
            let mut pipe = redis::pipe();
            pipe.atomic();
            pipe.hset_multiple(&job.key, &job.to_array());
            queue.pipe_add_job_id(job.id, &mut pipe);
            pipe
        };
        pipe.query_async(con).await.map_err(RsrqError::RedisOpError)?;

        // Return the Job
        Ok(job)
    }

    pub fn to_array(&self) -> [(JobKey, String); 11] {
        [
            (JobKey::Id, self.id.to_string()),
            (JobKey::Cmd, self.cmd.clone()),
            (JobKey::Status, self.status.to_string()),
            (JobKey::Queue, self.queue.clone()),
            (JobKey::Created, self.created.to_string()),
            (JobKey::Started, self.started.map(|x| x.to_string()).unwrap_or("".to_string())),
            (JobKey::Finished, self.finished.map(|x| x.to_string()).unwrap_or("".to_string())),
            (JobKey::Stdout, self.stdout.clone().unwrap_or("".to_string())),
            (JobKey::Stderr, self.stderr.clone().unwrap_or("".to_string())),
            (JobKey::ExitCode, self.exit_code.map(|x| x.to_string()).unwrap_or("".to_string())),
            (JobKey::DurationMs, self.duration_ms.map(|x| x.to_string()).unwrap_or("".to_string())),
        ]
    }

    pub async fn get_status_many(ids: Vec<usize>, con: &mut ConnectionManager) -> RsrqResult<Vec<JobStatus>> {
        let mut pipe = redis::pipe();
        for id in ids {
            let key = Job::get_redis_key(id);
            pipe.hget(&key, JobKey::Status);
        }
        let statuses: Vec<JobStatus> = pipe.query_async(con).await.map_err(RsrqError::RedisOpError)?;
        Ok(statuses)
    }

    pub async fn load(id: usize, con: &mut ConnectionManager) -> RsrqResult<Job> {
        // Set the parameters
        let key = Job::get_redis_key(id);

        // Load the job
        let map: BTreeMap<String, String> = con.hgetall(&key).await.map_err(RsrqError::RedisOpError)?;

        // Convert them to the expected data types
        let job_id = btree_get(&map, JobKey::Id)?;
        let job_cmd = btree_get(&map, JobKey::Cmd)?;
        let job_status_str: String = btree_get(&map, JobKey::Status)?;
        let job_status = JobStatus::from_string(&job_status_str)?;
        let job_queue = btree_get(&map, JobKey::Queue)?;
        let job_created = btree_get(&map, JobKey::Created)?;
        let job_started = btree_get_opt(&map, JobKey::Started)?;
        let job_finished = btree_get_opt(&map, JobKey::Finished)?;
        let job_stdout = btree_get_opt(&map, JobKey::Stdout)?;
        let job_stderr = btree_get_opt(&map, JobKey::Stderr)?;
        let job_exit_code = btree_get_opt(&map, JobKey::ExitCode)?;
        let job_duration_ms = btree_get_opt(&map, JobKey::DurationMs)?;

        // Create the job
        let job = Job {
            id: job_id,
            key,
            cmd: job_cmd,
            status: job_status,
            queue: job_queue,
            created: job_created,
            started: job_started,
            finished: job_finished,
            stdout: job_stdout,
            stderr: job_stderr,
            exit_code: job_exit_code,
            duration_ms: job_duration_ms,
        };
        Ok(job)
    }
}

