use log::info;
use redis::Commands;

use crate::model::error::RsrqError;
use crate::model::types::RsrqResult;
use crate::util::redis::redis_conn;

pub fn purge_all() -> RsrqResult<()> {
    let mut con = redis_conn()?;

    // Obtain all keys
    let keys: Vec<String> = con.scan_match("rsrq:*").map_err(RsrqError::RedisOpError)?.collect();

    if keys.is_empty() {
        info!("No keys found.");
        return Ok(());
    }

    let mut pipe = redis::pipe();
    pipe.atomic();
    for key in &keys {
        pipe.del(key);
    }
    pipe.query(&mut con).map_err(RsrqError::RedisOpError)?;
    info!("Successfully removed {} keys.", keys.len());
    Ok(())
}
