use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version)]
#[command(about = "rsrq - a minimal Redis-backed job queue.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Enqueue a batch of commands to be run.
    #[command(arg_required_else_help = true)]
    Enqueue {
        /// The target queue to add jobs to.
        queue: String,

        /// Path to the file containing one command per-line.
        path: String,
    },

    /// Spawns worker processes to consume jobs from a queue.
    #[command(arg_required_else_help = true)]
    Worker {
        /// The target queue to process.
        queue: String,

        /// The number of workers to spawn.
        #[clap(long, default_value = "1")]
        workers: u16,

        /// Stop processing after (h)ours (m)inutes (s)econds (eg: 1h30m, 30m, 1h5s).
        #[clap(long)]
        max_duration: Option<String>,

        /// Stop processing after this many jobs have finished.
        #[clap(long)]
        max_jobs: Option<u32>,

        /// Stop processing once the queue is empty.
        #[clap(long, default_value = "false")]
        burst: bool,

        /// Interval to check for new jobs in milliseconds.
        #[clap(long, default_value = "1000")]
        poll: u64,
    },

    /// Commands that can be issued by Snakemake for cluster execution.
    Snakemake(SnakemakeArgs),

    /// Commands for removing data from Redis.
    Purge(PurgeArgs),
}

#[derive(Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct SnakemakeArgs {
    #[command(subcommand)]
    pub command: SnakemakeCommands,
}

#[derive(Debug, Subcommand)]
pub enum SnakemakeCommands {
    /// Submit a Snakemake job
    Submit {
        /// The path to the job script
        job_script: String
    },
    /// Check the status of a Snakemake job
    Status {
        /// The Job ID to check
        job_id: usize
    },
    /// Cancel a Snakemake job
    Cancel {
        /// The Jobs ID to cancel
        job_ids: Vec<usize>
    },
    /// Write the Snakemake cluster profile to the target directory.
    Config {
        /// The target directory to write the profile to.
        directory: String,
    }
}

#[derive(Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct PurgeArgs {
    #[command(subcommand)]
    pub command: PurgeCommands,
}

#[derive(Debug, Subcommand)]
pub enum PurgeCommands {
    /// Removes all data from Redis.
    All,
    /// Removes all failed jobs.
    Failed {
        /// The target queue to purge (default: all queues).
        #[clap(long)]
        queue: Option<String>,
    },
    /// Removes all finished jobs.
    Finished {
        /// The target queue to purge (default: all queues).
        #[clap(long)]
        queue: Option<String>,
    },
    /// Removes all queued jobs.
    Queued {
        /// The target queue to purge (default: all queues).
        #[clap(long)]
        queue: Option<String>,
    },
}
