/*
Configuration for Redis key values.
 */


// Environment variable for Redis connection string
pub static REDIS_ENV_URL: &str = "REDIS_URL";

// Lists are prefixed with the following
pub static Q_RUNNING: &str = "rsrq:running";
pub static Q_QUEUED: &str = "rsrq:queued";
pub static Q_FINISHED: &str = "rsrq:finished";
pub static Q_FAILED: &str = "rsrq:failed";

// Hash prefixed
pub static JOB_KEY: &str = "rsrq:job";
pub static WORKER_KEY: &str = "rsrq:worker";

// Auto-incrementing UID for worker and jobs
pub static UID_KEY_JOB: &str = "rsrq:uid:job";
pub static UID_KEY_WORKER: &str = "rsrq:uid:worker";


// TODO: SET TTL VALUES & redis timeout
