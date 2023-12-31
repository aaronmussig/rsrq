use log::debug;
use redis::aio::ConnectionManager;

use crate::command::worker::util::update_redis_start_job;
use crate::model::command::RsrqCommand;
use crate::model::error::RsrqError;
use crate::model::job::key::JobKey;
use crate::model::job::rsrq_job::Job;
use crate::model::queue::queue_type::QueueType;
use crate::model::queue::rsrq_queue::Queue;
use crate::model::types::RsrqResult;
use crate::model::worker::result::WorkerResult;
use crate::util::time::get_timestamp_s;

/// This is the main method called by the worker to wrap all logic.
pub async fn worker_async_on_job_id(proc_id: usize, job_id: usize, queue_name: String, con: &mut ConnectionManager) -> RsrqResult<()> {
    // Register the worker class
    debug!("Process {} is now listening on {}", proc_id, &queue_name);

    let job = Job::load(job_id, con).await?;
    debug!("Process {} has obtained job {}", proc_id, job.id);
    process_new_job(proc_id, &queue_name, &job, con).await?;

    debug!("Process {} is now done on {}", proc_id, &queue_name);
    Ok(())
}


/// This is where the thread calls the command.
pub async fn process_new_job(proc_id: usize, queue_name: &str, job: &Job, con: &mut ConnectionManager) -> RsrqResult<()> {
    let start_ts = get_timestamp_s()?.to_string();

    // Update the job and worker in the database to be in a running state
    update_redis_start_job(proc_id, job, &start_ts, con).await?;

    // Run the actual job
    let command = RsrqCommand::new(&job.cmd);
    let job_res = match command {
        // There was no issue parsing the command, run it
        Ok(engine) => {
            let result = engine.run().await;
            WorkerResult::from_command(&result)
        }
        // The command could not be parsed
        Err(_) => WorkerResult::from_failed()
    };

    // The job has finished running (or didn't run if the parser failed)
    let end_ts = get_timestamp_s()?.to_string();

    // Set the target queues
    let q_source = Queue::new(QueueType::Running, queue_name);
    let q_target = Queue::new(job_res.q_target, queue_name);

    let job_update_arr = [
        (JobKey::Status, &job_res.job_status.to_string()),
        (JobKey::Finished, &end_ts),
        (JobKey::Stdout, &job_res.stdout),
        (JobKey::Stderr, &job_res.stderr),
        (JobKey::DurationMs, &job_res.duration_ms.to_string()),
        (JobKey::ExitCode, &job_res.exit_code.to_string())
    ];

    // Update the database with the job status
    let pipe = {
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.hset_multiple(&job.key, &job_update_arr);
        q_source.pipe_remove_job_id(job.id, &mut pipe);
        q_target.pipe_add_job_id(job.id, &mut pipe);
        pipe
    };
    pipe.query_async(con).await.map_err(RsrqError::RedisOpError)?;

    Ok(())
}