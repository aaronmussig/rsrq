use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use log::info;

use crate::model::error::RsrqError;
use crate::model::types::RsrqResult;

pub fn snakemake_config(directory: &str) -> RsrqResult<()> {
    let cancel_name = "cancel.sh";
    let jobscript_name = "jobscript.sh";
    let status_name = "status.sh";
    let submit_name = "submit.sh";

    let config_lines: [&str; 4] = [
        &format!("jobscript: {}", jobscript_name),
        &format!("cluster: {}", submit_name),
        &format!("cluster-status: {}", status_name),
        &format!("cluster-cancel: {}", cancel_name)
    ];

    let jobscript_lines = [
        "#!/bin/bash",
        "# properties = {properties}",
        "",
        "set -o errexit",
        "{exec_job}",
        ""
    ];

    let cancel_lines = [
        "#!/bin/bash",
        "rsrq snakemake cancel \"$@\"",
        ""
    ];

    let status_lines = [
        "#!/bin/bash",
        "rsrq snakemake status \"$@\"",
        ""
    ];

    let submit_lines = [
        "#!/bin/bash",
        "rsrq snakemake submit \"$@\"",
        ""
    ];

    let dir = Path::new(directory);

    let config = dir.join("config.yaml");
    let cancel = dir.join(cancel_name);
    let jobscript = dir.join(jobscript_name);
    let status = dir.join(status_name);
    let submit = dir.join(submit_name);

    // Create the directory if it does not exist
    if dir.exists() {
        return Err(RsrqError::GeneralError(format!("Directory {} already exists", directory)));
    } else {
        fs::create_dir_all(directory).map_err(RsrqError::IOError)?;
    }

    // Write the files
    fs::write(config, config_lines.join("\n")).map_err(RsrqError::IOError)?;
    fs::write(&cancel, cancel_lines.join("\n")).map_err(RsrqError::IOError)?;
    fs::write(jobscript, jobscript_lines.join("\n")).map_err(RsrqError::IOError)?;
    fs::write(&status, status_lines.join("\n")).map_err(RsrqError::IOError)?;
    fs::write(&submit, submit_lines.join("\n")).map_err(RsrqError::IOError)?;

    // Set the permissions to be executable
    fs::set_permissions(cancel, fs::Permissions::from_mode(0o755)).map_err(RsrqError::IOError)?;
    fs::set_permissions(status, fs::Permissions::from_mode(0o755)).map_err(RsrqError::IOError)?;
    fs::set_permissions(submit, fs::Permissions::from_mode(0o755)).map_err(RsrqError::IOError)?;

    info!("Wrote files to: {}", directory);
    info!("Run Snakemake as: snakemake --profile {}", directory);
    Ok(())
}