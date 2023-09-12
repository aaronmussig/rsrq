use std::env;

use clap::Parser;
use log::{error, info};

use crate::command::enqueue::main::enqueue_file;
use crate::command::purge::all::purge_all;
use crate::command::purge::queue::purge_queue;
use crate::command::snakemake::cancel::snakemake_cancel;
use crate::command::snakemake::config::snakemake_config;
use crate::command::snakemake::status::snakemake_status;
use crate::command::snakemake::submit::snakemake_submit;
use crate::command::status::check_status::check_status;
use crate::command::worker::main::run_workers;
use crate::model::cli::{Cli, Commands, PurgeCommands, SnakemakeCommands};
use crate::model::queue::queue_type::QueueType;

mod util;
mod model;
mod config;
mod command;


#[tokio::main]
async fn main() {
    // Initialise the logger at the default level of info
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::init();

    // Initialise the CLI and parse the arguments
    let cli = Cli::parse();
    match &cli.command {

        // Run the enqueue workflow
        Commands::Enqueue { path, queue } => {
            match enqueue_file(path, queue).await {
                Ok(_) => info!("Successfully enqueued jobs."),
                Err(e) => {
                    error!("Error enqueuing jobs: {}", e);
                    std::process::exit(1);
                }
            }
        }

        // Run the workers workflow
        Commands::Worker { queue, workers, max_duration, max_jobs, burst, poll } => {
            match run_workers(queue, *workers, max_duration, *max_jobs, *burst, *poll).await {
                Ok(_) => info!("Workers stopped successfully."),
                Err(e) => {
                    error!("Error running workers: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Status { queue } => {
            match check_status(queue).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Error running workers: {}", e);
                    std::process::exit(1);
                }
            }
        }

        // Run a snakemake subcommand
        Commands::Snakemake(snakemake) => {
            match &snakemake.command {
                SnakemakeCommands::Submit { job_script } => {
                    if let Err(err) = snakemake_submit(job_script).await {
                        error!("Error submitting job: {}", err);
                        std::process::exit(1);
                    }
                }
                SnakemakeCommands::Status { job_id } => {
                    if let Err(err) = snakemake_status(*job_id) {
                        error!("Error checking job status: {}", err);
                        std::process::exit(1);
                    }
                }
                SnakemakeCommands::Cancel { job_ids } => {
                    if let Err(err) = snakemake_cancel(job_ids) {
                        error!("Error cancelling job: {}", err);
                        std::process::exit(1);
                    }
                }
                SnakemakeCommands::Config { directory } => {
                    if let Err(err) = snakemake_config(directory) {
                        error!("Error creating Snakemake cluster config: {}", err);
                        std::process::exit(1);
                    }
                }
            }
        }

        // Run a purge subcommand
        Commands::Purge(purge) => {
            match &purge.command {
                PurgeCommands::All => {
                    if let Err(err) = purge_all() {
                        error!("Error purging data: {}", err);
                        std::process::exit(1);
                    }
                }
                PurgeCommands::Failed { queue } => {
                    if let Err(err) = purge_queue(queue, QueueType::Failed) {
                        error!("Error purging data: {}", err);
                        std::process::exit(1);
                    }
                }
                PurgeCommands::Finished { queue } => {
                    if let Err(err) = purge_queue(queue, QueueType::Finished) {
                        error!("Error purging data: {}", err);
                        std::process::exit(1);
                    }
                }
                PurgeCommands::Queued { queue } => {
                    if let Err(err) = purge_queue(queue, QueueType::Queued) {
                        error!("Error purging data: {}", err);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
