use std::io::BufRead;
use log::error;

pub struct EnqueueFile {
    pub jobs: Vec<String>,
}

impl EnqueueFile {
    pub fn load(filename: &str) -> EnqueueFile {
        let mut jobs: Vec<String> = Vec::new();
        let file = match std::fs::File::open(filename) {
            Ok(file) => file,
            Err(error) => {
                error!("Unable to open file: {}", error);
                std::process::exit(1);
            }
        };

        let reader = std::io::BufReader::new(file);
        for line in reader.lines() {
            let job_str = match line {
                Ok(line) => line,
                Err(error) => {
                    error!("Unable to read line: {}", error);
                    std::process::exit(1);
                }
            };
            jobs.push(job_str);
        }
        EnqueueFile {
            jobs
        }
    }
}