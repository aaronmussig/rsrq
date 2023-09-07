use log::info;
use redis::Commands;

use crate::model::error::RsrqError;
use crate::model::job::rsrq_job::Job;
use crate::model::queue::queue_type::QueueType;
use crate::model::queue::rsrq_queue::Queue;
use crate::model::types::RsrqResult;
use crate::util::redis::redis_conn;

pub fn purge_queue(queue: &Option<String>, queue_type: QueueType) -> RsrqResult<()> {
    let mut con = redis_conn()?;

    let queue_obj = match queue {
        Some(q) => Queue::new(queue_type, q),
        None => Queue::new(queue_type, "*"),
    };

    // If no queue name was supplied, obtain all queue names
    let keys_to_search = if queue.is_none() {
        let queue_names: Vec<String> = con.scan_match(queue_obj.key).map_err(RsrqError::RedisOpError)?.collect();
        queue_names
    } else {
        vec![queue_obj.key]
    };

    // Iterate over each queue and collect the job ids
    let mut pipe = redis::pipe();
    pipe.atomic();
    for key in &keys_to_search {
        let values: Vec<usize> = match queue_obj.q_type {
            QueueType::Queued => con.lrange(key, 0, -1).map_err(RsrqError::RedisOpError)?,
            QueueType::Running => con.lrange(key, 0, -1).map_err(RsrqError::RedisOpError)?,
            QueueType::Finished => con.smembers(key).map_err(RsrqError::RedisOpError)?,
            QueueType::Failed => con.smembers(key).map_err(RsrqError::RedisOpError)?,
        };

        // Delete each id
        for value in &values {
            let job_key = Job::get_redis_key(*value);
            pipe.del(&job_key);
        }
        info!("Removed {} jobs from queue {}", values.len(), key);

        // Delete the queue
        pipe.del(key);
    }
    pipe.query(&mut con).map_err(RsrqError::RedisOpError)?;

    Ok(())
}


