use clap::Parser;
use kvm_install_vm::{Cli, vm::VirtualMachine};
use std::process;

fn main() {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let args = Cli::parse();

    println!("Starting kvm-install-vm Rust implementation...");
    println!("VM Name: {}", args.name);
    println!("Distribution: {}", args.distro);

    println!("Configuration:");
    println!("  vCPUs: {}", args.vcpus);
    println!("  Memory: {} MB", args.memory_mb);
    println!("  Disk Size: {} GB", args.disk_size_gb);

    let disk_path = format!("/home/giovanni/virt/images/{}.qcow2", args.name);
    let vm_name = args.name.clone();

    let mut vm = VirtualMachine::new(
        args.name,
        args.vcpus,
        args.memory_mb,
        args.disk_size_gb,
        disk_path,
        // args.distro,
    );

    if let Err(e) = vm.connect(None) {
        eprintln!("Failed to connect to libvirt: {}", e);
        process::exit(1);
    }

    match vm.create() {
        Ok(domain) => {
            println!("Successfully created VM: {}", vm_name);
            println!("Domain ID: {}", domain.get_id().unwrap_or(0));
        },
        Err(e) => {
            eprintln!("Failed to create VM: {}", e);
            process::exit(1);
        }
    }
}
