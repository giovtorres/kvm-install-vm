use anyhow::Result;
use clap::Parser;
use kvm_install_vm::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let cli = Cli::parse();

    println!("Starting kvm-install-vm Rust implementation...");
    println!("VM Name: {}", cli.name);
    println!("Distribution: {}", cli.distro);

    println!("Configuration:");
    println!("  vCPUs: {}", cli.vcpus);
    println!("  Memory: {} MB", cli.memory_mb);
    println!("  Disk Size: {} GB", cli.disk_size_gb);

    if cli.dry_run {
        println!("Dry run mode - no VM will be created");
        return Ok(());
    }

    println!("VM creation would start here...");

    Ok(())
}
