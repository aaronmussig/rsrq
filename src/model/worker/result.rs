use crate::model::command::RsrqCommandResult;
use crate::model::job::status::JobStatus;
use crate::model::queue::queue_type::QueueType;

pub struct WorkerResult {
    pub q_target: QueueType,
    pub job_status: JobStatus,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u128,
    pub exit_code: i32,
}

impl WorkerResult {
    pub fn from_failed() -> WorkerResult {
        WorkerResult {
            q_target: QueueType::Failed,
            job_status: JobStatus::Failed,
            stdout: "".to_string(),
            stderr: "Unable to parse command.".to_string(),
            duration_ms: 0,
            exit_code: 1,
        }
    }

    pub fn from_command(result: &RsrqCommandResult) -> WorkerResult {
        let (job_status, q_target) = match result.exit_code {
            0 => (JobStatus::Finished, QueueType::Finished),
            _ => (JobStatus::Failed, QueueType::Failed),
        };
        WorkerResult {
            q_target,
            job_status,
            stdout: result.stdout.to_string(),
            stderr: result.stderr.to_string(),
            duration_ms: result.duration_ms,
            exit_code: result.exit_code,
        }
    }
}

