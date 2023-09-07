use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};

use crate::model::error::RsrqError;
use crate::model::types::RsrqResult;
use crate::util::time::get_timestamp_string;

pub fn get_progress_style_running() -> RsrqResult<ProgressStyle> {
    let template = "{wide_bar} {msg} [ETA: {eta_precise}]";
    Ok(ProgressStyle::with_template(template).map_err(RsrqError::ProgressBarError)?.progress_chars("##-"))
}

pub fn get_process_style_waiting() -> RsrqResult<ProgressStyle> {
    let template = "[{msg}] - Waiting for new jobs...";
    Ok(ProgressStyle::with_template(template).map_err(RsrqError::ProgressBarError)?.progress_chars("##-"))
}


pub fn get_progress_bar(len: u64) -> RsrqResult<ProgressBar> {
    let bar = ProgressBar::new(len);
    bar.set_style(get_progress_style_running()?);
    // bar.enable_steady_tick(Duration::from_secs(BAR_TICK_SEC));
    Ok(bar)
}

pub struct RsrqProgressBar {
    pub bar: ProgressBar,
    style_running: bool,
    pub max_jobs: Option<u32>,
    pub max_runtime_secs: Option<u64>,
    pub start_time: Instant,
    pub jobs_started: usize,
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

fn format_remaining_jobs_as_str(max_jobs: u64, started_jobs: u64) -> String {
    if started_jobs > max_jobs {
        "0".to_string()
    } else {
        (max_jobs - started_jobs).to_string()
    }
}


impl RsrqProgressBar {
    pub fn new(init: u64, max_jobs: Option<u32>, max_runtime_secs: Option<u64>) -> RsrqResult<RsrqProgressBar> {
        let bar = get_progress_bar(init)?;
        bar.set_message(format!("[Queued: {}]", init));
        Ok(RsrqProgressBar {
            bar,
            style_running: true,
            max_jobs,
            max_runtime_secs,
            start_time: Instant::now(),
            jobs_started: 0,
        })
    }

    fn set_message(&self, remaining: u64) {
        let message = match (self.max_jobs, self.max_runtime_secs) {
            (Some(max_jobs), Some(max_runtime_secs)) => {
                let jobs_str = format_remaining_jobs_as_str(max_jobs as u64, self.jobs_started as u64);
                let secs_str = format_remaining_secs_as_str(max_runtime_secs, self.start_time.elapsed().as_secs());
                format!("[Queued: {}] [Max jobs remaining: {}] [Shutdown: {}]", remaining, jobs_str, secs_str)
            }
            (Some(max_jobs), None) => {
                let jobs_str = format_remaining_jobs_as_str(max_jobs as u64, self.jobs_started as u64);
                format!("[Queued: {}] [Max jobs remaining: {}]", remaining, jobs_str)
            }
            (None, Some(max_runtime_secs)) => {
                let secs_str = format_remaining_secs_as_str(max_runtime_secs, self.start_time.elapsed().as_secs());
                format!("[Queued: {}] [Shutdown: {}]", remaining, secs_str)
            }
            (None, None) => {
                format!("[Queued: {}]", remaining)
            }
        };
        self.bar.set_message(message);
    }


    fn maybe_set_style_waiting(&mut self) -> RsrqResult<()> {
        if self.style_running {
            self.bar.set_style(get_process_style_waiting()?);
            self.style_running = false;
        }
        Ok(())
    }

    fn maybe_set_style_running(&mut self) -> RsrqResult<()> {
        if !self.style_running {
            self.bar.set_style(get_progress_style_running()?);
            self.style_running = true;
        }
        Ok(())
    }

    pub fn tick_waiting(&mut self) -> RsrqResult<()> {
        self.maybe_set_style_waiting()?;
        self.bar.tick();
        self.bar.set_message(get_timestamp_string());
        Ok(())
    }

    pub fn tick_running(&mut self, increment: u64, remaining: u64) -> RsrqResult<()> {
        self.maybe_set_style_running()?;
        self.jobs_started += increment as usize;
        self.set_message(remaining);
        self.bar.inc(increment);
        self.bar.set_length(remaining + self.bar.position());
        Ok(())
    }


    pub fn finish_and_clear(&self) {
        if self.style_running {
            self.set_message(0);
        }
        self.bar.finish_and_clear();
    }
}

