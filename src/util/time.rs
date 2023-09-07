use crate::model::error::RsrqError;
use crate::model::types::RsrqResult;

pub fn get_timestamp_s() -> RsrqResult<u64> {
    let now = std::time::SystemTime::now();
    let delta = now.duration_since(std::time::UNIX_EPOCH).
        map_err(RsrqError::SystemTimeError)?;
    Ok(delta.as_secs())
}


pub fn get_ms_since(start_time: &std::time::Instant) -> u128 {
    let current_time = std::time::Instant::now();
    current_time.duration_since(*start_time).as_millis()
}

pub fn get_timestamp_string() -> String {
    let local_time = chrono::Local::now();
    format!("{}", local_time.format("%Y-%m-%d %H:%M:%S"))
}