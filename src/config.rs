/*
Configuration for Redis key values.
 */


// Environment variable for Redis connection string
pub const REDIS_ENV_URL: &str = "REDIS_URL";

// Lists are prefixed with the following
pub const Q_RUNNING: &str = "rsrq:running";
pub const Q_QUEUED: &str = "rsrq:queued";
pub const Q_FINISHED: &str = "rsrq:finished";
pub const Q_FAILED: &str = "rsrq:failed";

// Hash prefixed
pub const JOB_KEY: &str = "rsrq:job";
pub const WORKER_KEY: &str = "rsrq:worker";

// Auto-incrementing UID for worker and jobs
pub const UID_KEY_JOB: &str = "rsrq:uid:job";
pub const UID_KEY_WORKER: &str = "rsrq:uid:worker";


// TODO: SET TTL VALUES & redis timeout
