use lazy_static::lazy_static;
use regex::Regex;

use crate::model::error::RsrqError;
use crate::model::types::RsrqResult;
use crate::util::time::get_ms_since;

lazy_static! {
    static ref RE_CMD: Regex =  Regex::new(r#"("[^"]+"|'[^']+'|\S+)"#).unwrap();
}


fn parse_cmd(input: &str) -> Vec<String> {
    RE_CMD.captures_iter(input)
        .map(|cap| {
            cap[1].trim_matches('"').trim_matches('\'').to_string()
        })
        .collect()
}

pub struct RsrqCommandResult {
    pub cmd: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u128,
}

pub struct RsrqCommand {
    pub cmd: String,
    pub args: Vec<String>,
}

impl RsrqCommand {
    pub fn new(cmd: &str) -> RsrqResult<RsrqCommand> {
        let parsed_cmd = parse_cmd(cmd);
        if let Some((command, args)) = parsed_cmd.split_first() {
            return Ok(RsrqCommand {
                cmd: command.to_string(),
                args: args.to_vec(),
            });
        }
        Err(RsrqError::CmdParserError(format!("Could not parse command {}", cmd)))
    }

    pub async fn run(&self) -> RsrqCommandResult {
        let start_time = std::time::Instant::now();
        let cmd_result = tokio::process::Command::new(&self.cmd).args(&self.args).output().await;
        let duration_ms = get_ms_since(&start_time);
        return match cmd_result {
            Ok(output) => {
                RsrqCommandResult {
                    cmd: self.cmd.to_string(),
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    exit_code: output.status.code().unwrap_or(1),
                    duration_ms,
                }
            }
            Err(err) => {
                RsrqCommandResult {
                    cmd: self.cmd.to_string(),
                    stdout: "".to_string(),
                    stderr: err.to_string(),
                    exit_code: 1,
                    duration_ms,
                }
            }
        };
    }
}