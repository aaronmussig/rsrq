use std::fs::File;
use std::io::{BufRead, BufReader};

use serde_json::Value;

use crate::model::error::RsrqError;
use crate::model::job::rsrq_job::Job;
use crate::model::types::RsrqResult;
use crate::util::redis::redis_con_manager;

fn read_properties_from_jobscript(path: &str) -> RsrqResult<Value> {
    let file = File::open(path).map_err(RsrqError::FileNotFound)?;
    for line in BufReader::new(file).lines() {
        let line = line.map_err(RsrqError::FileReadError)?;
        if line.starts_with("# properties =") {
            let json_str = line.trim_start_matches("# properties =").trim();
            let resp: Value = serde_json::from_str(json_str).map_err(|e| RsrqError::InvalidJson(e.to_string()))?;
            return Ok(resp);
        }
    }
    Err(RsrqError::SnakemakeInvalidJobScript(format!("No properties section found in job script: {}", path)))
}

pub async fn snakemake_submit(path: &str) -> Result<(), RsrqError> {

    // Parse the properties from the file
    let properties = read_properties_from_jobscript(path)?;

    // Get the queue from the properties (if none are provided, then use default)
    let property_queue = &properties["resources"]["queue"];
    let queue = match &properties["resources"]["queue"] {
        Value::Null => "default".to_string(),
        _ => property_queue.as_str().unwrap_or("default").to_string(),
    };

    // Connect to redis
    let mut con = redis_con_manager().await?;

    // Create the job
    let job = Job::new(&queue, path, &mut con).await?;

    // Display the job id to the user
    println!("{}", job.id);
    Ok(())
}
