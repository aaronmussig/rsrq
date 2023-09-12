use std::{cmp, io};
use std::collections::HashMap;
use std::io::Write;
use std::time::Instant;

use log::error;
use terminal_size::{Height, terminal_size, Width};

use crate::model::types::RsrqResult;

pub struct RsrqProgressBar {
    max_jobs: Option<u32>,
    max_runtime_secs: Option<u64>,
    max_runtime_end_secs: Option<String>,
    start_time: Instant,
    workers: u32,

    // Track the start and end times of each job (for progress tracking)
    job_start: HashMap<usize, Instant>,
    job_duration: Vec<u128>,

    // Prevent flooding of the terminal
    last_stdout: Option<Instant>,
}


fn format_remaining_secs_as_str(max_runtime_secs: u64, elapsed_secs: u64) -> String {
    let remaining_secs: u64 = if elapsed_secs < max_runtime_secs {
        max_runtime_secs - elapsed_secs
    } else {
        0
    };
    let hours = remaining_secs / 3600;
    let minutes = (remaining_secs % 3600) / 60;
    let seconds = remaining_secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}


impl RsrqProgressBar {
    pub fn new(init: usize, workers: u32, max_jobs: Option<u32>, max_runtime_secs: Option<u64>) -> RsrqResult<RsrqProgressBar> {
        let start_time = Instant::now();

        let max_runtime_end_secs: Option<String> = if let Some(max_runtime_secs) = max_runtime_secs {
            let hours = max_runtime_secs / 3600;
            let minutes = (max_runtime_secs % 3600) / 60;
            let seconds = max_runtime_secs % 60;
            Some(format!("{:02}:{:02}:{:02}", hours, minutes, seconds))
        } else {
            None
        };

        let mut bar = RsrqProgressBar {
            max_jobs,
            max_runtime_secs,
            max_runtime_end_secs,
            start_time,
            workers,
            job_start: HashMap::new(),
            job_duration: Vec::new(),
            last_stdout: None,
        };

        bar.tick_running(init, 0)?;

        Ok(bar)
    }

    pub fn track_job_start(&mut self, job_id: usize) {
        self.job_start.insert(job_id, Instant::now());
    }

    pub fn track_job_end(&mut self, job_id: usize) {
        if let Some(job_start) = self.job_start.get(&job_id) {
            let delta = job_start.elapsed().as_millis();
            self.job_duration.push(delta);
            self.job_start.remove(&job_id);
        }
    }

    pub fn write(&mut self, msg: &str) {
        self.last_stdout = Some(Instant::now());

        // Pad the message to use the entire terminal width
        let size = terminal_size();
        let padding = if let Some((Width(w), Height(_h))) = size {
            w - msg.len() as u16
        } else {
            0
        };

        print!("{}{}", msg, " ".repeat(padding as usize));
        let res = io::stdout().flush();
        if res.is_err() {
            error!("Unable to flush stdout.");
        }
    }

    pub fn tick_waiting(&mut self) -> RsrqResult<()> {
        // To prevent flooding the terminal with messages
        if !self.allowed_to_check() {
            return Ok(());
        }

        let elapsed_secs = self.start_time.elapsed().as_secs();
        let elapsed_str = format_remaining_secs_as_str(elapsed_secs, 0);
        let remaining_str = if let Some(max_runtime_end_secs) = &self.max_runtime_end_secs {
            format!("/{}", max_runtime_end_secs)
        } else {
            "".to_string()
        };

        let msg = format!("\r[Elapsed: {}{}] - Waiting for new jobs...", elapsed_str, remaining_str);
        self.write(&msg);
        Ok(())
    }

    fn allowed_to_check(&self) -> bool {
        if let Some(last_stdout) = self.last_stdout {
            if last_stdout.elapsed().as_millis() <= 1000 {
                return false;
            }
        }
        true
    }

    fn get_average_duration(&self) -> Option<u128> {
        if self.job_duration.is_empty() {
            None
        } else {
            let sum: u128 = self.job_duration.iter().sum();
            Some(sum / self.job_duration.len() as u128)
        }
    }

    pub fn tick_running(&mut self, queue_len: usize, running_tasks: usize) -> RsrqResult<()> {
        // To prevent flooding the terminal with messages
        if !self.allowed_to_check() {
            return Ok(());
        }

        // Calculate the average job duration
        let avg_duration = self.get_average_duration();
        let avg_duration_str = format_avg_duration(avg_duration);

        // Display the number of max jobs (if provided)
        let max_jobs = if let Some(max_jobs) = self.max_jobs {
            format!("/{}", max_jobs)
        } else {
            "".to_string()
        };

        // If a max runtime duration is set, calculate that too
        let elapsed_secs = self.start_time.elapsed().as_secs();
        let elapsed_str = format_remaining_secs_as_str(elapsed_secs, 0);
        let remaining_str = if let Some(max_runtime_end_secs) = &self.max_runtime_end_secs {
            format!("/{}", max_runtime_end_secs)
        } else {
            "".to_string()
        };

        // Calculate the estimated runtime
        let runtime_est_ms = self.estimate_remaining_time(avg_duration, queue_len);
        let runtime_est = if let Some(runtime_est_ms) = runtime_est_ms {
            let fmt = format_ms_time_as_duration(runtime_est_ms);
            format!(" [ETA: {}] ", fmt)
        } else {
            "".to_string()
        };

        let msg = format!(
            "\r[Elapsed: {}{}] [Queued: {}] [Running: {}] [Completed: {}{}] {}{}",
            elapsed_str,
            remaining_str,
            queue_len,
            running_tasks,
            self.job_duration.len(),
            max_jobs,
            avg_duration_str,
            runtime_est
        );
        self.write(&msg);

        Ok(())
    }

    fn estimate_remaining_time(&self, avg_duration: Option<u128>, queue_len: usize) -> Option<u128> {
        if let Some(avg_duration) = avg_duration {
            let mut total_ms = 0;

            // These jobs will always contribute to the estimated running time
            for value in self.job_start.values() {
                let elapsed = value.elapsed().as_millis();
                let remaining = cmp::max(avg_duration - elapsed, 0);
                total_ms += remaining;
            }

            // If we have a max limit of jobs then only estimate on those
            if let Some(max_jobs) = self.max_jobs {
                let n_jobs = cmp::min(max_jobs as usize, queue_len);
                total_ms += avg_duration * n_jobs as u128;
            } else {
                total_ms += avg_duration * queue_len as u128;
            }
            let remaining_ms = total_ms / self.workers as u128;

            // If this will be longer than the max run duration then update it
            if let Some(max_duration) = self.max_runtime_secs {
                let elapsed_ms = self.start_time.elapsed().as_millis();
                let max_duration_ms = max_duration as u128 * 1000;
                if max_duration_ms < elapsed_ms {
                    return Some(0);
                } else {
                    let elapsed_remain = max_duration_ms - elapsed_ms;
                    if remaining_ms > elapsed_remain {
                        return Some(elapsed_remain);
                    }
                }
            }

            // Otherwise, return the computed value
            Some(remaining_ms)
        } else {
            None
        }
    }


    pub fn finish_and_clear(&self) {
        print!("\r");
        let res = io::stdout().flush();
        if res.is_err() {
            error!("Unable to flush stdout.");
        }
        // let duration_ms = format_ms_time_as_duration(self.start_time.elapsed().as_millis());
        // let n_jobs = self.job_duration.len();
        // let avg_ms_str = if let Some(avg_ms) = self.get_average_duration() {
        //     format!(" (average time per job: {})", format_ms_time_as_duration(avg_ms))
        // } else {
        //     "".to_string()
        // };
        // info!("Finished {} jobs in {}{}.", n_jobs, duration_ms, avg_ms_str);
    }
}

fn format_ms_time_as_duration(ms: u128) -> String {
    let hours = ms / 3600000;
    let minutes = (ms % 3600000) / 60000;
    let seconds = (ms % 60000) / 1000;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn format_avg_duration(avg_ms: Option<u128>) -> String {
    if let Some(avg_ms) = avg_ms {
        let fmt = format_ms_time_as_duration(avg_ms);
        format!("[avg. {}]", fmt)
    } else {
        "".to_string()
    }
}


