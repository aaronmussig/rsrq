use log::{info, warn};
use tokio::sync::mpsc;

use crate::command::worker::util::{create_wake_thread, get_max_runtime, parse_num_workers};
use crate::model::shutdown_handler::ShutdownHandler;
use crate::model::types::{RsrqResult, WorkerMsgRec, WorkerMsgSend};
use crate::model::worker::message::{WorkerMessage, WorkerMessageReason};
use crate::model::worker::pool::WorkerPool;

/// The main entry point for running the worker.
pub async fn run_workers(queue: &str, workers: u16, max_secs: &Option<String>, max_jobs: Option<u32>, burst: bool, poll: u64) -> RsrqResult<()> {

    // Parse arguments
    let max_workers = parse_num_workers(workers, max_jobs);
    let max_runtime_secs = if let Some(max_secs) = max_secs {
        Some(get_max_runtime(max_secs)?)
    } else {
        None
    };

    // Display a message that the process is about to start
    if max_workers == 1 {
        info!("Starting 1 worker to process queue: {}", queue);
    } else {
        info!("Starting {} workers to process queue: {}", max_workers, queue);
    }

    // Finished jobs and exit handlers will communicate with the main loop via this channel
    let (tx, mut rx): (WorkerMsgSend, WorkerMsgRec) = mpsc::channel(workers as usize * 10);

    // Create a worker pool to start/end jobs
    let mut pool = WorkerPool::new(queue, max_jobs, max_runtime_secs, max_workers, poll, burst, &tx).await?;

    /*
    The ShutdownHandler is responsible for stopping the program by sending a message to the channel.
    1. SIGINT (the is_cancelled method will return true).
    2. The maximum number of hours has been reached (if provided).
     */
    let mut shutdown_thread = ShutdownHandler::new(&tx);
    shutdown_thread.start();
    if let Some(max_runtime_secs) = max_runtime_secs {
        shutdown_thread.start_shutdown_timer(max_runtime_secs);
    }

    // Asynchronously send a message to trigger a loop of the main loop every poll milliseconds
    let wake_thread = create_wake_thread(&tx, poll);

    // Instantaneously the main loop by sending a message to check for jobs
    let _ = tx.send(WorkerMessage::check_for_jobs()).await;

    // Actions to take when a message is received (main loop)
    while let Some(message) = rx.recv().await {

        // Take actions depending on the type of message received
        match message.reason {

            // Exit conditions
            WorkerMessageReason::Sigint => {
                pool.progress.finish_and_clear();
                warn!("Terminating program, the remaining {} jobs will be marked as failed.", pool.futures.len());
                // Note: The jobs are marked as failed by the worker thread
                break;
            }
            WorkerMessageReason::TimeExceeded => {
                pool.progress.finish_and_clear();
                warn!("The maximum time has been reached, waiting for {} running jobs to finish.", pool.futures.len());
                break;
            }
            WorkerMessageReason::MaxJobs => {
                pool.progress.finish_and_clear();
                warn!("Maximum number of jobs has been reached, waiting for {} running jobs to finish.", pool.futures.len());
                break;
            }
            WorkerMessageReason::BurstNoJobs => {
                pool.progress.finish_and_clear();
                info!("No jobs in queue, exiting.");
                break;
            }

            // Sent by the wake thread or finished jobs
            WorkerMessageReason::CheckForJobs => {

                // This message contains a job id, it was therefore sent by a job that has finished
                if let Some(finished_job_id) = message.job_id {
                    pool.remove_job(finished_job_id);
                }

                // Jobs that were updated via another process (i.e. snakemake) need to be cancelled
                pool.abort_cancelled().await?;

                // Start new jobs if possible
                pool.maybe_start_new_jobs().await?;
            }
        }
    }

    // Terminate asynchronous threads
    wake_thread.abort();
    shutdown_thread.abort();

    // Await any jobs that may still be running (not the case for SIGINT / time exceeded)
    for (_, future) in pool.futures {
        let _ = future.await;
    }

    Ok(())
}