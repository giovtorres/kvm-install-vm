use clap::Parser;
use kvm_install_vm::{Cli, cli::Commands};
use std::ffi::OsString;

fn get_args(args: &[&str]) -> Vec<OsString> {
    vec![OsString::from("kvm-install-vm")]
        .into_iter()
        .chain(args.iter().map(|s| OsString::from(s)))
        .collect()
}

#[test]
fn test_cli_create_defaults() {
    let args = get_args(&["create", "--name", "test-vm"]);
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Create { 
            name, 
            distro, 
            vcpus, 
            memory_mb, 
            disk_size_gb, 
            graphics, 
            dry_run 
        } => {
            assert_eq!(name, "test-vm");
            assert_eq!(distro, "centos8");
            assert_eq!(vcpus, 1);
            assert_eq!(disk_size_gb, 10);
            assert_eq!(memory_mb, 1024);
            assert_eq!(graphics, false);
            assert_eq!(dry_run, false);
        },
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_cli_create_custom_values() {
    let args = get_args(&[
        "create",
        "--name", "custom-vm",
        "--distro", "ubuntu2004",
        "--vcpus", "4",
        "--memory-mb", "4096",
        "--disk-size-gb", "50",
        "--graphics",
        "--dry-run",
    ]);

    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Create { 
            name, 
            distro, 
            vcpus, 
            memory_mb, 
            disk_size_gb, 
            graphics, 
            dry_run 
        } => {
            assert_eq!(name, "custom-vm");
            assert_eq!(distro, "ubuntu2004");
            assert_eq!(vcpus, 4);
            assert_eq!(disk_size_gb, 50);
            assert_eq!(memory_mb, 4096);
            assert_eq!(graphics, true);
            assert_eq!(dry_run, true);
        },
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_cli_destroy_defaults() {
    let args = get_args(&["destroy", "--name", "test-vm"]);
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Destroy { name, remove_disk } => {
            assert_eq!(name, "test-vm");
            assert_eq!(remove_disk, false);
        },
        _ => panic!("Expected Destroy command"),
    }
}

#[test]
fn test_cli_destroy_with_disk_removal() {
    let args = get_args(&["destroy", "--name", "test-vm", "--remove-disk"]);
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Destroy { name, remove_disk } => {
            assert_eq!(name, "test-vm");
            assert_eq!(remove_disk, true);
        },
        _ => panic!("Expected Destroy command"),
    }
}