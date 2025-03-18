#[cfg(test)]
mod domain_list_tests {
    use clap::Parser;
    use kvm_install_vm::vm::{DomainState, VirtualMachine};
    use kvm_install_vm::{Cli, cli::Commands};
    use std::ffi::OsString;

    // Helper function for CLI tests
    fn get_args(args: &[&str]) -> Vec<OsString> {
        vec![OsString::from("kvm-install-vm")]
            .into_iter()
            .chain(args.iter().map(|s| OsString::from(s)))
            .collect()
    }

    #[test]
    fn test_cli_list_defaults() {
        let args = get_args(&["list"]);
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::List {
                all,
                running,
                inactive,
            } => {
                assert_eq!(all, false);
                assert_eq!(running, false);
                assert_eq!(inactive, false);
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_cli_list_with_flags() {
        let args = get_args(&["list", "--all", "--running"]);
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::List {
                all,
                running,
                inactive,
            } => {
                assert_eq!(all, true);
                assert_eq!(running, true);
                assert_eq!(inactive, false);
            }
            _ => panic!("Expected List command"),
        }
    }

    // This test requires libvirt to be running
    // Use #[ignore] to skip it in normal test runs
    #[test]
    #[ignore]
    fn test_domain_listing() -> anyhow::Result<()> {
        // This will connect to libvirt and list domains
        let domains = VirtualMachine::list_domains(None)?;

        // We can't assert much about the actual domains without knowing the test environment,
        // but we can check that the function runs without errors and returns a Vec
        println!("Found {} domains", domains.len());

        for domain in &domains {
            println!(
                "Domain: {}, ID: {:?}, State: {:?}",
                domain.name, domain.id, domain.state
            );

            // Check that inactive domains have no ID
            if domain.state == DomainState::Shutoff {
                assert_eq!(domain.id, None);
            }

            // If it's running, it should have an ID
            if domain.state == DomainState::Running {
                assert!(domain.id.is_some());
            }
        }

        Ok(())
    }
}
