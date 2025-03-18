use clap::Parser;
use kvm_install_vm::{Cli, cli::Commands, vm::VirtualMachine};
use std::process;

fn main() {
    env_logger::init();

    let cli = Cli::parse();

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
            println!("Starting kvm-install-vm Rust implementation...");
            println!("VM Name: {}", name);
            println!("Distribution: {}", distro);

            println!("Configuration:");
            println!("  vCPUs: {}", vcpus);
            println!("  Memory: {} MB", memory_mb);
            println!("  Disk Size: {} GB", disk_size_gb);
            println!("  Graphics: {}", graphics);

            if *dry_run {
                println!("Dry run mode - not creating VM");
                return;
            }

            let disk_path = format!("/home/giovanni/virt/images/{}.qcow2", name);
            let vm_name = name.clone();

            let mut vm =
                VirtualMachine::new(name.clone(), *vcpus, *memory_mb, *disk_size_gb, disk_path);

            if let Err(e) = vm.connect(None) {
                eprintln!("Failed to connect to libvirt: {}", e);
                process::exit(1);
            }

            match vm.create() {
                Ok(domain) => {
                    println!("Successfully created VM: {}", vm_name);
                    println!("Domain ID: {}", domain.get_id().unwrap_or(0));
                }
                Err(e) => {
                    eprintln!("Failed to create VM: {}", e);
                    process::exit(1);
                }
            }
        }

        Commands::Destroy { name, remove_disk } => {
            println!("Destroying VM: {}", name);

            match VirtualMachine::destroy(name, None, *remove_disk) {
                Ok(()) => {
                    println!("VM '{}' destroy operation completed successfully", name);
                }
                Err(e) => {
                    eprintln!("Failed to destroy VM '{}': {}", name, e);
                    process::exit(1);
                }
            }
        }

        Commands::List {
            all,
            running,
            inactive,
        } => {
            println!("Listing virtual machines...");

            // Determine which types of domains to list
            let filters = (*all, *running, *inactive);

            // If no specific flags are provided, default to showing all domains
            let show_all = filters == (false, false, false) || *all;

            match VirtualMachine::list_domains(None) {
                Ok(domains) => {
                    // Filter domains based on flags
                    let filtered_domains: Vec<_> = domains
                        .into_iter()
                        .filter(|domain| {
                            if show_all {
                                return true;
                            }

                            if *running && domain.state == kvm_install_vm::vm::DomainState::Running
                            {
                                return true;
                            }

                            if *inactive && domain.id.is_none() {
                                return true;
                            }

                            false
                        })
                        .collect();

                    // Print header
                    println!("{:<5} {:<30} {:<10}", "ID", "Name", "State");
                    println!("{:-<5} {:-<30} {:-<10}", "", "", "");

                    // Print domains
                    if filtered_domains.is_empty() {
                        println!("No domains found matching the specified criteria");
                    } else {
                        for domain in filtered_domains {
                            let id_str = match domain.id {
                                Some(id) => id.to_string(),
                                None => "-".to_string(),
                            };

                            println!("{:<5} {:<30} {:<10}", id_str, domain.name, domain.state);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to list domains: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}
