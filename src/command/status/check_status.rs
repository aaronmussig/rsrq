use std::collections::HashMap;
use log::info;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use crate::model::error::RsrqError;
use crate::model::job::key::JobKey;
use crate::model::job::rsrq_job::Job;
use crate::model::job::status::JobStatus;
use crate::model::queue::queue_type::QueueType;
use crate::model::queue::rsrq_queue::Queue;
use crate::model::types::RsrqResult;
use crate::util::redis::redis_con_manager;


async fn get_queue_names(con: &mut ConnectionManager) -> RsrqResult<Vec<Queue>> {
    let mut out = Vec::new();
    let mut keys: redis::AsyncIter<String> = con.scan_match("rsrq:*").await.map_err(RsrqError::RedisOpError)?;
    while let Some(key) = keys.next_item().await {
        let queue = Queue::from_key(&key);
        if let Ok(queue) = queue {
            out.push(queue);
        }
    }
    Ok(out)
}

struct MinimalJob {
    status: JobStatus,
    queue: String,
}

pub async fn check_status(queue_name: &Option<String>) -> RsrqResult<()> {
    let mut con = redis_con_manager().await?;

    // Obtain either all queues, or those belonging to the specified
    let queues = match queue_name {
        None => {
            get_queue_names(&mut con).await?
        }
        Some(queue) => {
            let mut out = Vec::new();
            for queue_type in QueueType::get_types() {
                let cur_queue = Queue::new(queue_type, queue);
                out.push(cur_queue);
            }
            out
        }
    };

    // Iterate over each queue and collect the job ids
    let job_ids: Vec<Vec<usize>> = {
        let mut pipe = redis::pipe();
        for queue in &queues {
            match queue.q_type {
                QueueType::Queued => pipe.lrange(&queue.key, 0, -1),
                QueueType::Running => pipe.lrange(&queue.key, 0, -1),
                QueueType::Finished => pipe.smembers(&queue.key),
                QueueType::Failed => pipe.smembers(&queue.key),
            };
        }
        pipe.query_async(&mut con).await.map_err(RsrqError::RedisOpError)?
    };

    // Flatten the job ids into a single vector
    let job_ids: Vec<usize> = job_ids.into_iter().flatten().collect();

    // Iterate over each job id and print the job details
    let keys_to_get = vec![JobKey::Status, JobKey::Queue];
    let job_details: Vec<String> = {
        let mut pipe = redis::pipe();
        for job_id in job_ids {
            let job_key = Job::get_redis_key(job_id);
            for key_to_get in &keys_to_get {
                pipe.hget(&job_key, key_to_get);
            }
        }
        pipe.query_async(&mut con).await.map_err(RsrqError::RedisOpError)?
    };

    // Associate the retrieved values with each job
    let jobs: Vec<MinimalJob> = {
        let mut out = Vec::new();
        let mut job_details_iter = job_details.into_iter();
        while let Some(status) = job_details_iter.next() {
            let queue = job_details_iter.next().unwrap();
            let status = JobStatus::from_string(&status)?;
            let job = MinimalJob {
                status,
                queue,
            };
            out.push(job);
        }
        out
    };

    let mut queue_infos: HashMap<String, QueueInfo> = HashMap::new();
    for job in jobs {
        let queue_info = queue_infos.entry(job.queue.clone()).or_insert(QueueInfo::new(&job.queue));
        match job.status {
            JobStatus::Queued => queue_info.n_queued += 1,
            JobStatus::Running => queue_info.n_running += 1,
            JobStatus::Finished => queue_info.n_finished += 1,
            JobStatus::Failed => queue_info.n_failed += 1,
            JobStatus::Cancelled => queue_info.n_cancelled += 1,
        };
    }

    // Get the sorted keys
    let queue_keys = {
        let mut out = queue_infos.keys().map(|x| x.to_string()).collect::<Vec<String>>();
        out.sort();
        out
    };

    // Iterate over each value
    for queue_key in &queue_keys {
        let queue_info = queue_infos.get(queue_key).unwrap();
        info!(
            "Queue: {:<10} [Queued {:<5}] [Running {:<5}] [Finished {:<5}] [Failed {:<5}] [Cancelled {:<5}]",
            queue_info.name,
            queue_info.n_queued,
            queue_info.n_running,
            queue_info.n_finished,
            queue_info.n_failed,
            queue_info.n_cancelled
        );
    }

    Ok(())
}

struct QueueInfo {
    name: String,
    n_finished: usize,
    n_failed: usize,
    n_queued: usize,
    n_running: usize,
    n_cancelled: usize,
}

impl QueueInfo {
    pub fn new(name: &str) -> QueueInfo {
        QueueInfo {
            name: name.to_string(),
            n_finished: 0,
            n_failed: 0,
            n_queued: 0,
            n_running: 0,
            n_cancelled: 0,
        }
    }
}