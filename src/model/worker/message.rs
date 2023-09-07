#[derive(Debug)]
pub enum WorkerMessageReason {
    Sigint,
    TimeExceeded,
    CheckForJobs,
    MaxJobs,
    BurstNoJobs,
}

/// Wraps messages sent through tokio channels.
#[derive(Debug)]
pub struct WorkerMessage {
    pub job_id: Option<usize>,
    pub reason: WorkerMessageReason,
}

impl WorkerMessage {
    pub fn exit_sigint() -> WorkerMessage {
        WorkerMessage {
            job_id: None,
            reason: WorkerMessageReason::Sigint,
        }
    }

    pub fn exit_time() -> WorkerMessage {
        WorkerMessage {
            job_id: None,
            reason: WorkerMessageReason::TimeExceeded,
        }
    }

    pub fn check_for_jobs() -> WorkerMessage {
        WorkerMessage {
            job_id: None,
            reason: WorkerMessageReason::CheckForJobs,
        }
    }


    pub fn finished_job(job_id: usize) -> WorkerMessage {
        WorkerMessage {
            job_id: Some(job_id),
            reason: WorkerMessageReason::CheckForJobs,
        }
    }

    pub fn exit_max_jobs() -> WorkerMessage {
        WorkerMessage {
            job_id: None,
            reason: WorkerMessageReason::MaxJobs,
        }
    }

    pub fn burst_no_jobs() -> WorkerMessage {
        WorkerMessage {
            job_id: None,
            reason: WorkerMessageReason::BurstNoJobs,
        }
    }
}