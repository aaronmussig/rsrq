use std::env;

use log::error;
use redis::Connection;
use redis::aio::ConnectionManager;

use crate::config::REDIS_ENV_URL;
use crate::model::error::RsrqError;
use crate::model::types::RsrqResult;

pub fn get_redis_conn_string() -> RsrqResult<String> {
    let res = env::var(REDIS_ENV_URL).map_err(RsrqError::EnvError);
    match res {
        Ok(conn_string) => Ok(conn_string),
        Err(_) => {
            error!("Environment variable {} not set (e.g. export {}=redis://localhost:9001)", REDIS_ENV_URL, REDIS_ENV_URL);
            Err(RsrqError::EnvError(env::VarError::NotPresent))
        }
    }
}

pub fn redis_conn() -> RsrqResult<Connection> {
    let conn_string = get_redis_conn_string()?;
    let client = redis::Client::open(conn_string).map_err(RsrqError::RedisConnError)?;
    let con = client.get_connection().map_err(RsrqError::RedisConnError)?;
    Ok(con)
}

pub async fn redis_con_manager() -> RsrqResult<ConnectionManager> {
    let conn_string = get_redis_conn_string()?;
    let client = redis::Client::open(conn_string).map_err(RsrqError::RedisConnError)?;
    let manager = client.get_tokio_connection_manager_with_backoff(2, 1, 5).await.map_err(RsrqError::RedisConnError)?;
    Ok(manager)
}

// pub async fn get_con_async() -> RsrqResult<redis::aio::Connection> {
//     let conn_string = get_redis_conn_string()?;
//     let client = redis::Client::open(conn_string).map_err(RsrqError::RedisConnError)?;
//     let con = client.get_async_connection().await.map_err(RsrqError::RedisConnError)?;
//     Ok(con)
// }

pub async fn get_next_uid<C: redis::aio::ConnectionLike>(key: &str, con: &mut C) -> RsrqResult<usize> {
    let id: usize = redis::cmd("INCR").arg(key).query_async(con).await.map_err(RsrqError::RedisOpError)?;
    Ok(id)
}
