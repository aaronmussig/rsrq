# rsrq
**R**u**s**t **R**edis **q**ueue

[![BioConda](https://img.shields.io/conda/vn/bioconda/rsrq?color=43b02a)](https://anaconda.org/bioconda/rsrq)
[![Crates](https://img.shields.io/crates/v/rsrq?color=orange)](https://crates.io/crates/rsrq)

rsrq is a minimal Redis-backed queue system written in Rust and inspired by [RQ](https://python-rq.org/).

Overview:

* Queues are populated by a text file containing commands to be executed (each on a new line).
* Workers can be spawned in parallel to process the queue with various modes.
* Native integration with Snakemake cluster profiles.

Feedback and contributions are welcome!

## ⚙ Installation

*Requires Rust 1.70+*

```shell
cargo install rsrq
```

If the command `rsrq` cannot be found after installation, [follow these details](https://doc.rust-lang.org/book/ch14-04-installing-binaries.html).


## 🖥 CLI usage

This is not an exhaustive list of commands, help can be viewed by running the `rsrq --help` command.

```shell
# Export the Redis connection string
export REDIS_URL=redis://:your-password@your-endpoint-url

# Enqueue commands in "/tmp/cmds.txt" to the "test" queue.
rsrq enqueue test /tmp/cmds.txt

# Spawn 10 workers to process the "test" queue.
rsrq worker test --workers 10

# Purge all information from the redis database
rsrq purge all
```

## 🐍 Snakemake

When using Snakemake integration, a cluster profile will need to be created to map the commands for `submit`, `status`, and `cancel`. You are responsible for starting workers that will process the queue(s).

```shell
# Export the Snakemake profile
rsrq snakemake config /path/to/directory

# Export the Redis connection string
export REDIS_URL=redis://:your-password@your-endpoint-url

# Run snakemake with the cluster profile
snakemake --profile /path/to/directory
```

You can specify a queue for each rule using the `resources.queue` attribute. If none is provided `default` will be used, e.g.:

```Snakefile
rule foo:
    resources:
        queue='something'
```

Note: Workers do not check the CPU/memory usage of the server, it's entirely up to the user to decide how many workers can run concurrently.
