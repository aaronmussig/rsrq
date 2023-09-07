use redis::Connection;

use crate::model::error::RsrqError;
use crate::model::job::key::JobKey;
use crate::model::job::rsrq_job::Job;
use crate::model::job::status::JobStatus;
use crate::model::queue::queue_type::QueueType;
use crate::model::queue::rsrq_queue::Queue;
use crate::model::types::RsrqResult;
use crate::util::collection::deduplicate;
use crate::util::redis::redis_conn;

struct MinimalJob {
    pub id: usize,
    pub status: JobStatus,
    pub queue: String,
}

impl MinimalJob {
    pub fn is_cancellable(&self) -> bool {
        match self.status {
            JobStatus::Queued => true,
            JobStatus::Running => true,
            JobStatus::Finished => false,
            JobStatus::Failed => false,
            JobStatus::Cancelled => false,
        }
    }
}

/// Collect the status of specified jobs.
fn get_job_statuses(job_ids: &[usize], con: &mut Connection) -> RsrqResult<Vec<MinimalJob>> {

    // Setup shared variables
    let mut jobs: Vec<MinimalJob> = Vec::with_capacity(job_ids.len());

    // Connect to redis
    let mut pipe = redis::pipe();

    // Obtain the status and queue of each job
    for job_id in job_ids {
        let job_key = Job::get_redis_key(*job_id);
        pipe.hget(&job_key, JobKey::Status);
        pipe.hget(&job_key, JobKey::Queue);
    }
    let results: Vec<Option<String>> = pipe.query(con).map_err(RsrqError::RedisOpError)?;

    // Results will be returned in the order specified
    for (i, job_id) in job_ids.iter().enumerate() {
        let status_opt = &results[i * 2];
        let queue_opt = &results[i * 2 + 1];
        if let Some(status_str) = status_opt {
            if let Some(queue_str) = queue_opt {
                jobs.push(MinimalJob {
                    id: *job_id,
                    status: JobStatus::from_string(status_str)?,
                    queue: queue_str.clone(),
                });
            }
        }
    }
    Ok(jobs)
}

/// Sets the Job status to cancelled, removes it from the queue and adds it to the failed queue.
fn cancel_jobs(jobs: &[MinimalJob], con: &mut Connection) -> RsrqResult<()> {
    let mut pipe = redis::pipe();
    pipe.atomic();
    for job in jobs {
        if job.is_cancellable() {
            // Update the job status
            let job_key = Job::get_redis_key(job.id);
            pipe.hset(&job_key, JobKey::Status, JobStatus::Cancelled);

            // Remove it from the queue it's currently in and place it into the failed queue
            let from_q = Queue::new(job.status.to_queue_type(), &job.queue);
            let to_q = Queue::new(QueueType::Failed, &job.queue);
            from_q.pipe_remove_job_id(job.id, &mut pipe);
            to_q.pipe_add_job_id(job.id, &mut pipe);
        }
    }
    pipe.query(con).map_err(RsrqError::RedisOpError)?;
    Ok(())
}

pub fn snakemake_cancel(job_ids: &[usize]) -> RsrqResult<()> {

    // Deduplicate the job ids
    let job_ids: Vec<usize> = deduplicate(job_ids);

    // Connect to Redis
    let mut con = redis_conn()?;

    // Collect the status and queue of each job
    let job_info = get_job_statuses(&job_ids, &mut con)?;

    // Cancel the jobs that should be cancelled
    if !job_info.is_empty() {
        cancel_jobs(&job_info, &mut con)?;
    }

    Ok(())
}
