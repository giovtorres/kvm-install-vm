use clap::Parser;
use kvm_install_vm::Cli;
use std::ffi::OsString;

fn get_args(args: &[&str]) -> Vec<OsString> {
    vec![OsString::from("kvm-install-vm")]
        .into_iter()
        .chain(args.iter().map(|s| OsString::from(s)))
        .collect()
}

#[test]
fn test_cli_defaults() {
    let args = get_args(&["--name", "test-vm"]);
    let cli = Cli::parse_from(args);

    assert_eq!(cli.name, "test-vm");
    assert_eq!(cli.distro, "centos8");
    assert_eq!(cli.vcpus, 1);
    assert_eq!(cli.disk, 10);
    assert_eq!(cli.memory, 1024);
    assert_eq!(cli.graphics, false);
    assert_eq!(cli.dry_run, false);
}

#[test]
fn test_cli_custom_values() {
    let args = get_args(&[
        "--name",
        "custom-vm",
        "--distro",
        "ubuntu2004",
        "--vcpus",
        "4",
        "--memory",
        "4096",
        "--disk",
        "50",
        "--graphics",
        "--dry-run",
    ]);

    let cli = Cli::parse_from(args);

    assert_eq!(cli.name, "custom-vm");
    assert_eq!(cli.distro, "ubuntu2004");
    assert_eq!(cli.vcpus, 4);
    assert_eq!(cli.disk, 50);
    assert_eq!(cli.memory, 4096);
    assert_eq!(cli.graphics, true);
    assert_eq!(cli.dry_run, true);
}
