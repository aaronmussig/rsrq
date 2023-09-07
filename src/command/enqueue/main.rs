use log::info;

use crate::model::enqueue_file::EnqueueFile;
use crate::model::job::rsrq_job::Job;
use crate::model::queue::queue_type::QueueType;
use crate::model::queue::rsrq_queue::Queue;
use crate::model::types::RsrqResult;
use crate::util::redis::redis_con_manager;

/// Main method called to enqueue a collection of commands into a given queue
pub async fn enqueue_file(path: &str, queue: &str) -> RsrqResult<Vec<Job>> {
    // Read the file
    info!("Reading jobs from file: {}", &path);
    let enqueue_file = EnqueueFile::load(path);

    // Enqueue jobs in redis
    let q_target = Queue::new(QueueType::Queued, queue);
    info!("Enqueuing {} jobs to queue: {}", enqueue_file.jobs.len(), q_target.key);

    // Connect to Redis and execute
    let mut con = redis_con_manager().await?;

    // Create the pipeline of commands
    let mut created_jobs: Vec<Job> = Vec::with_capacity(enqueue_file.jobs.len());
    for job_cmd in &enqueue_file.jobs {
        let job = Job::new(queue, job_cmd, &mut con).await?;
        created_jobs.push(job);
    }

    // Report success
    info!("Successfully enqueued jobs.");

    Ok(created_jobs)
}

// #[tokio::test]
// async fn test_enqueue_file() {
//     use tempfile::NamedTempFile;
//     use std::io::Write;
//
//     let cmd_1 = "echo 'hello'";
//     let cmd_2 = "echo 'world'";
//
//     let mut file = NamedTempFile::new().unwrap();
//     writeln!(file, "{}", cmd_1).unwrap();
//     writeln!(file, "{}", cmd_2).unwrap();
//
//     let jobs = enqueue_file(file.path().to_str().unwrap(), "UNIT_TEST").await.unwrap();
//
//     assert_eq!(jobs.len(), 2);
//     assert_eq!(jobs[0].cmd, cmd_1);
//     assert_eq!(jobs[1].cmd, cmd_2);
// }
