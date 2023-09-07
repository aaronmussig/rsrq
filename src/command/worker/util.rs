use lazy_static::lazy_static;
use log::warn;
use redis::aio::ConnectionManager;
use regex::Regex;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::model::error::RsrqError;
use crate::model::job::rsrq_job::Job;
use crate::model::job::key::JobKey;
use crate::model::job::status::JobStatus;
use crate::model::types::RsrqResult;
use crate::model::worker::key::WorkerKey;
use crate::model::worker::message::WorkerMessage;
use crate::model::worker::rsrq_worker::Worker;

lazy_static! {
    static ref RE_MAX_DURATION: Regex =  Regex::new(r"(\d+)(\w)").unwrap();
}


/// Parses the worker arguments to ensure that the number of workers is less than the maximum number of iterations (if provided).
pub fn parse_num_workers(num_workers: u16, max_iter: Option<u32>) -> u32 {
    if let Some(max_it) = max_iter {
        if max_it < num_workers as u32 {
            warn!("The maximum number of iterations ({}) is less than the number of workers ({}), scaling workers down.", max_it, num_workers);
            return max_it;
        }
    }
    num_workers as u32
}

pub async fn update_redis_start_job(worker: &Worker, job: &Job, start_ts: &str, con: &mut ConnectionManager) -> RsrqResult<()> {
    let worker_arr = [
        (WorkerKey::CurrentJob, &job.id.to_string()),
        (WorkerKey::CurrentJobStart, &start_ts.to_string()),
    ];
    let job_arr = [
        (JobKey::Status, &JobStatus::Running.to_string()),
        (JobKey::Started, &start_ts.to_string())
    ];
    let mut pipe = redis::pipe();
    pipe.atomic();
    pipe.hset_multiple(&worker.key, &worker_arr);
    pipe.hset_multiple(&job.key, &job_arr);
    pipe.query_async(con).await.map_err(RsrqError::RedisOpError)?;
    Ok(())
}


pub fn create_wake_thread(tx: &mpsc::Sender<WorkerMessage>, poll_ms: u64) -> JoinHandle<()> {
    let tx = tx.clone();
    let thread = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(poll_ms));
        loop {
            interval.tick().await;
            let _ = tx.send(WorkerMessage::check_for_jobs()).await;
        }
    });
    thread
}

fn parse_time_unit(unit: &str, num: &str) -> RsrqResult<usize> {
    let num_res = num.parse::<usize>();
    if num_res.is_err() {
        return Err(RsrqError::CmdParserError(format!("Invalid number format: {}", num)));
    }
    let num = num_res.unwrap();

    match unit {
        "h" => Ok(num * 60 * 60),
        "m" => Ok(num * 60),
        "s" => Ok(num),
        _ => Err(RsrqError::CmdParserError(format!("Invalid time unit provided: {}", unit)))
    }
}

pub fn get_max_runtime(max_duration: &str) -> RsrqResult<u64> {
    let hits = RE_MAX_DURATION.captures_iter(max_duration).map(|hit| {
        let (_, [num, unit]) = hit.extract();
        parse_time_unit(unit, num)
    });
    let mut total_seconds = 0;
    let mut hits_seen = 0;
    for hit in hits {
        total_seconds += hit?;
        hits_seen += 1;
    }
    if hits_seen == 0 {
        return Err(RsrqError::CmdParserError(format!("Invalid max duration provided: {}", max_duration)));
    }
    Ok(total_seconds as u64)
}


#[test]
fn test_get_max_runtime() {
    fn h2s(hours: u64) -> u64 {
        hours * 60 * 60
    }

    fn m2s(minutes: u64) -> u64 {
        minutes * 60
    }

    assert_eq!(get_max_runtime("1h2m3s").unwrap(), h2s(1) + m2s(2) + 3);
    assert_eq!(get_max_runtime("10h20m30s").unwrap(), h2s(10) + m2s(20) + 30);
    assert_eq!(get_max_runtime("100h200m300s").unwrap(), h2s(100) + m2s(200) + 300);

    // Test only seconds
    assert_eq!(get_max_runtime("1s").unwrap(), h2s(0) + m2s(0) + 1);

    // Test only minutes
    assert_eq!(get_max_runtime("2m").unwrap(), h2s(0) + m2s(2) + 0);

    // Test only hours
    assert_eq!(get_max_runtime("1h").unwrap(), h2s(1) + m2s(0) + 0);

    // Test hours and minutes
    assert_eq!(get_max_runtime("1h2m").unwrap(), h2s(1) + m2s(2) + 0);

    // Test minutes and seconds
    assert_eq!(get_max_runtime("2m3s").unwrap(), h2s(0) + m2s(2) + 3);

    // Test hours and seconds
    assert_eq!(get_max_runtime("1h3s").unwrap(), h2s(1) + m2s(0) + 3);

    // Test hours, minutes, and seconds
    assert_eq!(get_max_runtime("1h2m3s").unwrap(), h2s(1) + m2s(2) + 3);

    // Test with leading zeros
    assert_eq!(get_max_runtime("01h02m03s").unwrap(), h2s(1) + m2s(2) + 3);

    // Test with different orders
    assert_eq!(get_max_runtime("2m1h").unwrap(), h2s(1) + m2s(2) + 0);
    assert_eq!(get_max_runtime("3s1h").unwrap(), h2s(1) + m2s(0) + 3);
    assert_eq!(get_max_runtime("3s2m").unwrap(), h2s(0) + m2s(2) + 3);

    // Test with invalid inputs
    assert!(get_max_runtime("1h2m3x").is_err());
    assert!(get_max_runtime("1j2k3l").is_err());
    assert!(get_max_runtime("").is_err());

    assert_eq!(get_max_runtime("1h2m3s").unwrap(), h2s(1) + m2s(2) + 3);
    assert_eq!(get_max_runtime("1h3s2m").unwrap(), h2s(1) + m2s(2) + 3);
    assert_eq!(get_max_runtime("2m1h3s").unwrap(), h2s(1) + m2s(2) + 3);
    assert_eq!(get_max_runtime("2m3s1h").unwrap(), h2s(1) + m2s(2) + 3);
    assert_eq!(get_max_runtime("2m3s1h").unwrap(), h2s(1) + m2s(2) + 3);

    assert_eq!(get_max_runtime("2m1h3s").unwrap(), h2s(1) + m2s(2) + 3);

    assert_eq!(get_max_runtime("2m3s").unwrap(), h2s(0) + m2s(2) + 3);
    assert_eq!(get_max_runtime("3s2m").unwrap(), h2s(0) + m2s(2) + 3);

    assert_eq!(get_max_runtime("1h2m").unwrap(), h2s(1) + m2s(2) + 0);
    assert_eq!(get_max_runtime("1h2s").unwrap(), h2s(1) + m2s(0) + 2);
    assert_eq!(get_max_runtime("1m2s").unwrap(), h2s(0) + m2s(1) + 2);

    assert_eq!(get_max_runtime("1s2m3h").unwrap(), h2s(3) + m2s(2) + 1);
    assert_eq!(get_max_runtime("2m3h").unwrap(), h2s(3) + m2s(2) + 0);
    assert_eq!(get_max_runtime("2m3h").unwrap(), h2s(3) + m2s(2) + 0);
}