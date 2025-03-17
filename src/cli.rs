use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(short = 'n', long)]
    pub name: String,

    #[arg(short = 't', long, default_value = "centos8")]
    pub distro: String,

    #[arg(short = 'c', long, default_value_t = 1)]
    pub vcpus: u32,

    #[arg(short = 'm', long, default_value_t = 1024)]
    pub memory: u32,

    #[arg(short = 'd', long, default_value_t = 10)]
    pub disk: u32,

    #[arg(long)]
    pub graphics: bool,

    #[arg(long)]
    pub dry_run: bool,
}
