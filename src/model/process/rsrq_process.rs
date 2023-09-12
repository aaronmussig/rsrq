use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::config::{PROC_KEY, UID_KEY_PROC};
use crate::model::error::RsrqError;
use crate::model::process::rsrq_process_key::ProcessKey;
use crate::model::process::rsrq_process_state::ProcessState;
use crate::model::types::RsrqResult;
use crate::util::redis::get_next_uid;
use crate::util::system::{get_hostname, get_pid};
use crate::util::time::get_timestamp_s;

pub struct Process {
    pub key: String,

    pub id: usize,
    pub hostname: String,
    pub pid: u32,
    pub birth: u64,
    pub last_heartbeat: u64,
    pub state: ProcessState,
    pub n_running: usize,

    pub queue: String,
    pub workers: u32,
    pub max_duration_sec: Option<u64>,
    pub max_jobs: Option<u32>,
    pub burst: bool,
    pub poll_ms: u64,
}


impl Process {
    pub fn get_key(id: usize) -> String {
        format!("{}:{}", PROC_KEY, id)
    }

    pub async fn new(queue: &str, workers: u32, max_duration_sec: Option<u64>, max_jobs: Option<u32>, burst: bool, poll_ms: u64, con: &mut ConnectionManager) -> RsrqResult<Process> {
        // Assign the next worker id to this worker
        let worker_id = get_next_uid(UID_KEY_PROC, con).await?;

        // Get attributes for the worker
        let hostname = get_hostname();
        let pid = get_pid();
        let birth = get_timestamp_s()?;

        let proc = Process {
            id: worker_id,
            key: Process::get_key(worker_id),
            hostname,
            pid,
            birth,
            last_heartbeat: birth,
            state: ProcessState::Idle,
            n_running: 0,
            queue: queue.to_string(),
            workers,
            max_duration_sec,
            max_jobs,
            burst,
            poll_ms,
        };

        // Update Redis
        proc.push(con).await?;

        // Return the worker
        Ok(proc)
    }
    pub fn to_array(&self) -> [(ProcessKey, String); 13] {
        // Parse the optional attributes
        let max_jobs = {
            if let Some(max_jobs) = self.max_jobs {
                max_jobs.to_string()
            } else {
                "".to_string()
            }
        };
        let max_duration_sec = {
            if let Some(max_duration_sec) = self.max_duration_sec {
                max_duration_sec.to_string()
            } else {
                "".to_string()
            }
        };
        [
            (ProcessKey::Id, self.id.to_string()),
            (ProcessKey::Hostname, self.hostname.clone()),
            (ProcessKey::Pid, self.pid.to_string()),
            (ProcessKey::Birth, self.birth.to_string()),
            (ProcessKey::LastHeartbeat, self.last_heartbeat.to_string()),
            (ProcessKey::State, self.state.to_string()),
            (ProcessKey::NumRunning, self.n_running.to_string()),
            (ProcessKey::Queue, self.queue.to_string()),
            (ProcessKey::Workers, self.workers.to_string()),
            (ProcessKey::MaxDurationSec, max_duration_sec),
            (ProcessKey::MaxJobs, max_jobs),
            (ProcessKey::Burst, self.burst.to_string()),
            (ProcessKey::PollMs, self.poll_ms.to_string()),
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

    pub async fn update_running(&mut self, n_running: usize, con: &mut ConnectionManager) -> RsrqResult<()> {
        self.last_heartbeat = get_timestamp_s()?;
        self.state = if n_running > 0 {
            ProcessState::Running
        } else {
            ProcessState::Idle
        };
        self.n_running = n_running;

        let values = [
            (ProcessKey::LastHeartbeat, self.last_heartbeat.to_string()),
            (ProcessKey::State, self.state.to_string()),
            (ProcessKey::NumRunning, self.n_running.to_string()),
        ];
        con.hset_multiple(&self.key, &values).await.map_err(RsrqError::RedisOpError)?;
        Ok(())
    }
}

