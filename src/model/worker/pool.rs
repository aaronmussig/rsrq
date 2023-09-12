use std::cmp;
use std::collections::HashMap;

use log::{debug, warn};
use redis::aio::ConnectionManager;

use crate::command::worker::run_on_job::worker_async_on_job_id;
use crate::model::job::rsrq_job::Job;
use crate::model::job::status::JobStatus;
use crate::model::progress_bar::RsrqProgressBar;
use crate::model::queue::queue_type::QueueType;
use crate::model::queue::rsrq_queue::Queue;
use crate::model::types::{JobFuture, RsrqResult, WorkerMsgSend};
use crate::model::worker::message::WorkerMessage;

pub struct WorkerPool {
    pub proc_id: usize,
    pub last_check_time: std::time::Instant,
    pub futures: HashMap<usize, JobFuture>,
    pub tx: WorkerMsgSend,
    pub con: ConnectionManager,
    pub poll_ms: u128,
    pub n_jobs_started: usize,
    pub max_workers: usize,
    pub max_jobs: Option<usize>,
    pub queue: Queue,
    pub progress: RsrqProgressBar,
    pub burst: bool,
    has_run_once: bool,
}

impl WorkerPool {
    pub async fn new(proc_id: usize, queue: &str, max_jobs: Option<u32>, max_runtime_secs: Option<u64>, max_workers: u32, poll_ms: u64, burst: bool, tx: &WorkerMsgSend, con: &ConnectionManager) -> RsrqResult<WorkerPool> {
        let mut con = con.clone();
        let q = Queue::new(QueueType::Queued, queue);

        let progress = {
            let n_queued = q.length(&mut con).await?;
            let init_qty: u64 = match max_jobs {
                None => n_queued as u64,
                Some(max_jobs) => cmp::min(max_jobs as u64, n_queued as u64)
            };
            let mut bar = RsrqProgressBar::new(init_qty, max_jobs, max_runtime_secs)?;
            if n_queued == 0 {
                bar.tick_waiting()?;
            }
            bar
        };

        Ok(WorkerPool {
            proc_id,
            last_check_time: std::time::Instant::now(),
            futures: HashMap::new(),
            tx: tx.clone(),
            con,
            poll_ms: poll_ms as u128,
            n_jobs_started: 0,
            max_workers: max_workers as usize,
            max_jobs: max_jobs.map(|x| x as usize),
            queue: q,
            progress,
            burst,
            has_run_once: false,
        })
    }

    pub async fn maybe_start_new_jobs(&mut self) -> RsrqResult<()> {
        if self.has_run_once {
            let ms_since_last_check = self.last_check_time.elapsed().as_millis();
            if ms_since_last_check < self.poll_ms {
                return Ok(());
            }
        } else {
            self.has_run_once = true
        }

        let n_queued = self.queue.length(&mut self.con).await?;

        let n_jobs_to_add = {
            let max_workers_to_add = cmp::min(self.max_workers - self.futures.len(), n_queued);
            if let Some(max_jobs) = self.max_jobs {
                cmp::min(max_workers_to_add, max_jobs - self.n_jobs_started)
            } else {
                max_workers_to_add
            }
        };

        let new_job_ids = self.queue.get_n_next_job_ids(n_jobs_to_add, &mut self.con).await?;
        let n_new_jobs_added = new_job_ids.len();
        for cur_job_id in new_job_ids {
            // Here the actual method thread is spawned
            self.run(cur_job_id);
        }
        self.n_jobs_started += n_new_jobs_added;

        // Update the progress bar
        let remaining_tasks = n_queued - n_new_jobs_added + self.futures.len();
        self.update_remaining_tasks(n_new_jobs_added, remaining_tasks).await?;

        // Send a message to stop the loop
        if let Some(max_jobs) = self.max_jobs {
            if self.n_jobs_started >= max_jobs {
                let _ = self.tx.send(WorkerMessage::exit_max_jobs()).await;
            }
        }

        // Reset the last check time
        self.last_check_time = std::time::Instant::now();
        Ok(())
    }


    pub async fn update_remaining_tasks(&mut self, new_jobs_added: usize, remaining_tasks: usize) -> RsrqResult<()> {
        if remaining_tasks == 0 {
            // Exit if we are running in burst mode
            if self.burst {
                let _ = self.tx.send(WorkerMessage::burst_no_jobs()).await;
            } else {
                debug!("start tick waiting");
                self.progress.tick_waiting()?;
                debug!("end tick waiting")
            }
        } else {
            self.progress.tick_running(new_jobs_added as u64, remaining_tasks as u64)?;
        }
        Ok(())
    }

    pub fn run(&mut self, job_id: usize) {
        let queue_clone = self.queue.name.to_string();
        let mut manager_copy = self.con.clone();
        let tx = self.tx.clone();
        let proc_id = self.proc_id;
        let thread = tokio::spawn(async move {
            let res = worker_async_on_job_id(proc_id, job_id, queue_clone, &mut manager_copy).await;
            let _ = tx.send(WorkerMessage::finished_job(job_id)).await;
            res
        });
        self.futures.insert(job_id, thread);
    }

    pub fn remove_job(&mut self, job_id: usize) {
        self.futures.remove(&job_id);
    }

    pub async fn abort_cancelled(&mut self) -> RsrqResult<()> {
        debug!("Checking if any jobs were marked as cancelled.");

        // Store the job ids in a vector to preserve ordering
        let job_keys: Vec<usize> = self.futures.keys().copied().collect();

        // Collect the statuses
        let result: Vec<JobStatus> = Job::get_status_many(job_keys.clone(), &mut self.con).await?;
        debug!("Job statuses: {:?}", result);

        // Abort any failed jobs that should be cancelled
        let mut cancelled_ids: Vec<usize> = Vec::new();
        for (cur_job_id, status) in job_keys.iter().zip(result) {
            let to_cancel = match status {
                JobStatus::Queued => false,
                JobStatus::Running => false,
                JobStatus::Finished => false,
                JobStatus::Failed => false,
                JobStatus::Cancelled => true
            };
            debug!("Job ID {} (status={}) is marked as cancelled: {}", cur_job_id, status, to_cancel);
            if to_cancel {
                debug!("Aborting job ID {}", cur_job_id);
                if let Some(future) = self.futures.get(cur_job_id) {
                    debug!("Found future");
                    cancelled_ids.push(*cur_job_id);
                    future.abort();
                    self.remove_job(*cur_job_id);
                }
            }
        }
        if !cancelled_ids.is_empty() {
            cancelled_ids.sort();
            let cancel_str = cancelled_ids.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", ");
            warn!("The following Job IDs were cancelled and marked as failed: {}", cancel_str)
        }
        Ok(())
    }
}