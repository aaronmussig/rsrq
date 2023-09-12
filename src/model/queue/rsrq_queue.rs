use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::model::error::RsrqError;
use crate::model::queue::queue_type::QueueType;
use crate::model::types::{OptUsizeFuture, RsrqResult};

pub struct Queue {
    pub key: String,
    pub name: String,
    pub q_type: QueueType,
}

impl Queue {
    pub fn new(queue_type: QueueType, name: &str) -> Queue {
        let key = format!("{}:{}", queue_type.to_string(), name);
        Queue {
            key,
            name: name.to_string(),
            q_type: queue_type,
        }
    }

    pub fn from_key(key: &str) -> RsrqResult<Queue> {
        let mut splits = key.split(":");
        let prefix = splits.next().unwrap_or("");
        let q_type = splits.next().unwrap_or("");
        let name = splits.next().unwrap_or("");
        if prefix != "rsrq" {
            return Err(RsrqError::ParserError("Invalid prefix.".to_string()));
        }
        if q_type.is_empty() {
            return Err(RsrqError::ParserError("Invalid queue type.".to_string()));
        }
        if name.is_empty() {
            return Err(RsrqError::ParserError("Invalid queue name.".to_string()));
        }
        let queue_type = QueueType::from_string(&format!("{prefix}:{q_type}"))?;
        Ok(Queue::new(queue_type, name))
    }

    pub async fn length(&self, con: &mut ConnectionManager) -> RsrqResult<usize> {
        match self.q_type {
            QueueType::Queued => {
                con.llen(&self.key).await.map_err(RsrqError::RedisOpError)
            }
            QueueType::Running => {
                con.llen(&self.key).await.map_err(RsrqError::RedisOpError)
            }
            QueueType::Finished => {
                con.scard(&self.key).await.map_err(RsrqError::RedisOpError)
            }
            QueueType::Failed => {
                con.scard(&self.key).await.map_err(RsrqError::RedisOpError)
            }
        }
    }

    pub fn pipe_remove_job_id(&self, job_id: usize, pipe: &mut redis::Pipeline) {
        match self.q_type {
            QueueType::Queued => {
                pipe.lrem(&self.key, 1, job_id);
            }
            QueueType::Running => {
                pipe.lrem(&self.key, 1, job_id);
            }
            QueueType::Finished => {
                pipe.srem(&self.key, job_id);
            }
            QueueType::Failed => {
                pipe.srem(&self.key, job_id);
            }
        }
    }

    pub fn pipe_add_job_id(&self, job_id: usize, pipe: &mut redis::Pipeline) {
        match self.q_type {
            QueueType::Queued => {
                pipe.lpush(&self.key, job_id);
            }
            QueueType::Running => {
                pipe.lpush(&self.key, job_id);
            }
            QueueType::Finished => {
                pipe.sadd(&self.key, job_id);
            }
            QueueType::Failed => {
                pipe.sadd(&self.key, job_id);
            }
        }
    }

    pub async fn get_next_job_id(&self, con: &mut ConnectionManager) -> RsrqResult<Option<usize>> {
        match self.q_type {
            QueueType::Queued => {}
            _ => {
                return Err(RsrqError::ParserError(format!("Cannot get next job from queue type: {}", self.q_type)));
            }
        };
        let q_target = Queue::new(QueueType::Running, &self.name);
        let job_id: Option<usize> = con.lmove(&self.key, &q_target.key, redis::Direction::Right, redis::Direction::Left).await.map_err(RsrqError::RedisOpError)?;
        Ok(job_id)
    }

    pub async fn get_n_next_job_ids(&self, n: usize, con: &mut ConnectionManager) -> RsrqResult<Vec<usize>> {
        let mut futures = vec![];
        for _ in 0..n {
            let queue_clone = self.name.to_string();
            let mut con_clone = con.clone();
            let thread: OptUsizeFuture = tokio::spawn(async move {
                let q = Queue::new(QueueType::Queued, &queue_clone);
                let next_job = q.get_next_job_id(&mut con_clone).await?;
                Ok(next_job)
            });
            futures.push(thread);
        }
        // Await for the jobs to complete
        let mut out = vec![];
        for future in futures {
            if let Ok(cur_future) = future.await {
                if let Ok(result) = cur_future {
                    if let Some(job_id) = result {
                        out.push(job_id);
                    }
                }
            }
        }
        Ok(out)
    }
}

