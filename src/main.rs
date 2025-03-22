use clap::Parser;
use kvm_install_vm::{Cli, Commands, Config, VirtualMachine};
use std::io::Write;
use std::process;
use tracing::{debug, error, info};

/// Helper function to print status messages
fn _print_status(msg: &str, success: bool) {
    if success {
        println!("- {} ... \x1b[32mOK\x1b[0m", msg);
    } else {
        println!("- {} ... \x1b[31mFAILED\x1b[0m", msg);
    }
}

/// Helper to print a message with ellipsis without a newline
fn print_status_start(msg: &str) {
    print!("- {} ... ", msg);
    std::io::stdout().flush().unwrap_or(());
}

/// Simple progress message
fn print_progress(msg: &str) {
    println!("- {}", msg);
}

fn main() {
    let cli = Cli::parse();

    // Simple logging setup
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    match &cli.command {
        Commands::Create {
            name,
            distro,
            vcpus,
            memory_mb,
            disk_size_gb,
            graphics,
            dry_run,
        } => {
            info!("Starting VM creation process for: {}", name);
            print_progress(&format!("Starting kvm-install-vm for VM: {}", name));
            print_progress(&format!("Distribution: {}", distro));

            debug!(
                "Configuration: vCPUs={}, Memory={}MB, Disk={}GB, Graphics={}",
                vcpus, memory_mb, disk_size_gb, graphics
            );

            if *dry_run {
                print_progress("Dry run mode - not creating VM");
                return;
            }

            // Load configuration
            print_status_start("Loading configuration");
            let config = match Config::load() {
                Ok(config) => {
                    println!("\x1b[32mOK\x1b[0m");
                    config
                }
                Err(e) => {
                    println!("\x1b[31mFAILED\x1b[0m");
                    eprintln!("  Error: {}", e);
                    error!("Failed to load configuration: {}", e);
                    process::exit(1);
                }
            };

            // Initialize VM instance
            print_status_start("Creating VM instance");
            let vm_name = name.clone();
            let mut vm = VirtualMachine::new(
                name.clone(),
                *vcpus,
                *memory_mb,
                *disk_size_gb,
                String::new(),
            );
            println!("\x1b[32mOK\x1b[0m");

            // Connect to libvirt
            print_status_start("Connecting to libvirt");
            if let Err(e) = vm.connect(None) {
                println!("\x1b[31mFAILED\x1b[0m");
                eprintln!("  Error: {}", e);
                error!("Failed to connect to libvirt: {}", e);
                process::exit(1);
            }
            println!("\x1b[32mOK\x1b[0m");

            // Create VM with proper image handling
            // Set up a runtime for async operations
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(async {
                print_status_start("Preparing VM image");
                match vm.prepare_image(distro, &config).await {
                    Ok(_) => {
                        println!("\x1b[32mOK\x1b[0m");

                        print_status_start("Creating virtual machine");
                        match vm.create() {
                            Ok(domain) => {
                                println!("\x1b[32mOK\x1b[0m");
                                let domain_id = domain.get_id().unwrap_or(0);

                                info!("Successfully created VM: {}", vm_name);
                                info!("Domain ID: {}", domain_id);

                                println!("Successfully created VM: {}", vm_name);
                                println!("Domain ID: {}", domain_id);
                            }
                            Err(e) => {
                                println!("\x1b[31mFAILED\x1b[0m");
                                eprintln!("  Error: {}", e);
                                error!("Failed to create VM: {}", e);
                                process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        println!("\x1b[31mFAILED\x1b[0m");
                        eprintln!("  Error: {}", e);
                        error!("Failed to prepare VM image: {}", e);
                        process::exit(1);
                    }
                }
            });
        }

        Commands::Destroy { name, remove_disk } => {
            info!("Starting VM destruction process for: {}", name);
            print_progress(&format!("Destroying VM: {}", name));

            debug!(
                "Destroying parameters - Name: {}, Remove Disk: {}",
                name, remove_disk
            );

            print_status_start("Destroying virtual machine");
            match VirtualMachine::destroy(name, None, *remove_disk) {
                Ok(()) => {
                    println!("\x1b[32mOK\x1b[0m");
                    print_progress(&format!(
                        "VM '{}' destroy operation completed successfully",
                        name
                    ));
                    info!("VM '{}' destroyed successfully", name);
                }
                Err(e) => {
                    println!("\x1b[31mFAILED\x1b[0m");
                    eprintln!("  Error: {}", e);
                    error!("Failed to destroy VM '{}': {}", name, e);
                    process::exit(1);
                }
            }
        }

        Commands::List {
            all,
            running,
            inactive,
        } => {
            info!("Listing VMs");
            print_progress("Listing virtual machines...");

            // Determine which types of domains to list
            let show_all = *all || (!*running && !*inactive);

            debug!(
                "List parameters - All: {}, Running: {}, Inactive: {}, Show all: {}",
                all, running, inactive, show_all
            );

            match VirtualMachine::print_domain_list(None, show_all, *running, *inactive) {
                Ok(()) => {
                    info!("VM listing completed successfully");
                }
                Err(e) => {
                    error!("Failed to list domains: {}", e);
                    eprintln!("Failed to list domains: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}
