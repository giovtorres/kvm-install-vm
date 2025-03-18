use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Create {
        #[arg(short = 'n', long)]
        name: String,

        #[arg(short = 't', long, default_value = "centos8")]
        distro: String,

        #[arg(short = 'c', long, default_value_t = 1)]
        vcpus: u32,

        #[arg(short = 'm', long, default_value_t = 1024)]
        memory_mb: u32,

        #[arg(short = 'd', long, default_value_t = 10)]
        disk_size_gb: u32,

        #[arg(long)]
        graphics: bool,

        #[arg(long)]
        dry_run: bool,
    },
    Destroy {
        #[arg(short = 'n', long)]
        name: String,

        #[arg(short = 'r', long)]
        remove_disk: bool,
    },
}
