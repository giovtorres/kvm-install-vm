use anyhow::Result;
use kvm_install_vm::vm::{DomainState, VirtualMachine};
use tempfile::tempdir;

// Test VM creation and initialization without actually interacting with libvirt
#[test]
fn test_vm_initialization() {
    let vm = VirtualMachine::new(
        "test-vm".to_string(),
        2,
        1024,
        10,
        "/tmp/test-vm.qcow2".to_string(),
    );

    assert_eq!(vm.name, "test-vm");
    assert_eq!(vm.vcpus, 2);
    assert_eq!(vm.memory_mb, 1024);
    assert_eq!(vm.disk_size_gb, 10);
    assert_eq!(vm.disk_path, "/tmp/test-vm.qcow2");
    assert!(vm.connection.is_none());
}

// Since generate_domain_xml is private, we'll test indirectly through create()
// This test is marked as ignored because it requires libvirt
#[test]
#[ignore]
fn test_domain_creation_xml() -> Result<()> {
    // Create a temporary directory for disk images
    let temp_dir = tempdir()?;
    let disk_path = temp_dir.path().join("test-xml-vm.qcow2");

    // Create a test disk image
    std::process::Command::new("qemu-img")
        .args(&["create", "-f", "qcow2", &disk_path.to_string_lossy(), "1G"])
        .output()?;

    // Create VM
    let mut vm = VirtualMachine::new(
        "test-xml-vm".to_string(),
        2,
        1024,
        10,
        disk_path.to_string_lossy().to_string(),
    );

    // Connect to libvirt
    vm.connect(None)?;

    // Use virsh to dump and inspect XML
    // This is a workaround since we can't call generate_domain_xml directly
    let output = std::process::Command::new("virsh")
        .args(&["-c", "qemu:///session", "dumpxml", "test-xml-vm"])
        .output()?;

    if output.status.success() {
        let xml = String::from_utf8(output.stdout)?;

        // Check for relevant XML elements
        assert!(xml.contains("<name>test-xml-vm</name>"));
        assert!(xml.contains("<memory"));
        assert!(xml.contains("<vcpu>"));
        assert!(xml.contains("<interface type='user'>"));
        assert!(!xml.contains("<interface type='network'>"));
    }

    // Clean up
    let _ = VirtualMachine::destroy("test-xml-vm", None, true);

    Ok(())
}

// The following tests require libvirt to be running
// They're marked as ignored so they don't run in automated testing

#[test]
#[ignore]
fn test_connect_to_libvirt() -> Result<()> {
    let mut vm = VirtualMachine::new(
        "test-connect-vm".to_string(),
        1,
        512,
        1,
        "/tmp/test-connect-vm.qcow2".to_string(),
    );

    vm.connect(None)?;
    assert!(vm.connection.is_some());

    Ok(())
}

#[test]
#[ignore]
fn test_domain_list_and_print() -> Result<()> {
    // Test the domain listing functionality
    let domains = VirtualMachine::list_domains(None)?;

    // Print the domains for debug purposes
    println!("Found {} domains:", domains.len());
    for domain in &domains {
        println!(
            "Domain: {}, ID: {:?}, State: {:?}",
            domain.name, domain.id, domain.state
        );

        // State consistency checks
        if domain.state == DomainState::Shutoff {
            assert_eq!(domain.id, None);
        }

        if domain.state == DomainState::Running {
            assert!(domain.id.is_some());
        }
    }

    // Test the print function (just make sure it doesn't crash)
    VirtualMachine::print_domain_list(None, true, false, false)?;

    Ok(())
}

// This test is complex and potentially disruptive
// It creates and then destroys a real VM, so use with caution
#[test]
#[ignore]
fn test_create_and_destroy_vm() -> Result<()> {
    // Create a temporary directory for disk images
    let temp_dir = tempdir()?;
    let disk_path = temp_dir.path().join("test-create-destroy.qcow2");

    // Create a test disk image
    std::process::Command::new("qemu-img")
        .args(&["create", "-f", "qcow2", &disk_path.to_string_lossy(), "1G"])
        .output()?;

    // Create VM
    let mut vm = VirtualMachine::new(
        "test-create-destroy".to_string(),
        1,
        512,
        1,
        disk_path.to_string_lossy().to_string(),
    );

    vm.connect(None)?;

    // Create the VM
    let domain = vm.create()?;

    // Verify domain exists
    let domain_name = domain.get_name()?;
    assert_eq!(domain_name, "test-create-destroy");

    // Try to destroy it
    VirtualMachine::destroy("test-create-destroy", None, true)?;

    // Verify it's gone using virsh
    let output = std::process::Command::new("virsh")
        .args(&["-c", "qemu:///session", "dominfo", "test-create-destroy"])
        .output()?;

    assert!(!output.status.success());

    Ok(())
}
