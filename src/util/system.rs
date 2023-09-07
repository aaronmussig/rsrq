/// Return the hostname of the machine, will return "N/A" if not possible.
pub fn get_hostname() -> String {
    let hostname = hostname::get();
    match hostname {
        Ok(hostname) => hostname.to_str().unwrap_or("N/A").to_string(),
        Err(_) => "N/A".to_string(),
    }
}

/// Return the PID of the main process (does not work on child threads).
pub fn get_pid() -> u32 {
    std::process::id()
}

