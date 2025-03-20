// tests/integration_test.rs
use anyhow::Result;
use kvm_install_vm::{
    cloudinit::CloudInitManager,
    vm::{DistroInfo, ImageManager, VirtualMachine},
};
use std::fs;
use tempfile::tempdir;

// This test combines multiple components but doesn't actually create a VM
// It prepares everything up to the point of VM creation
#[test]
fn test_vm_preparation_flow() -> Result<()> {
    // Skip if qemu-img is not available
    if std::process::Command::new("qemu-img")
        .arg("--version")
        .output()
        .is_err()
    {
        println!("Skipping test_vm_preparation_flow: qemu-img not available");
        return Ok(());
    }

    // Skip if neither genisoimage nor mkisofs is available
    let has_iso_tool = std::process::Command::new("genisoimage")
        .arg("--version")
        .output()
        .is_ok()
        || std::process::Command::new("mkisofs")
            .arg("--version")
            .output()
            .is_ok();

    if !has_iso_tool {
        println!("Skipping test_vm_preparation_flow: neither genisoimage nor mkisofs available");
        return Ok(());
    }

    // Create temporary directories
    let temp_dir = tempdir()?;
    let image_dir = temp_dir.path().join("images");
    let vm_dir = temp_dir.path().join("vms");

    fs::create_dir_all(&image_dir)?;
    fs::create_dir_all(&vm_dir)?;

    // Create a test distro
    let distro = DistroInfo {
        qcow_filename: "test-integration.qcow2".to_string(),
        os_variant: "generic".to_string(),
        image_url: "http://example.com".to_string(),
        login_user: "testuser".to_string(),
        sudo_group: "wheel".to_string(),
        cloud_init_disable: "systemctl disable cloud-init".to_string(),
    };

    // Create a base image (since we won't download one)
    let base_image = image_dir.join(&distro.qcow_filename);
    std::process::Command::new("qemu-img")
        .args(&["create", "-f", "qcow2", &base_image.to_string_lossy(), "1G"])
        .output()?;

    // Initialize image manager
    let image_manager = ImageManager::new(&image_dir);

    // Prepare VM directory
    let vm_name = "test-integration-vm";
    let vm_dir_path = vm_dir.join(vm_name);
    fs::create_dir_all(&vm_dir_path)?;

    // Prepare VM disk
    let disk_path = image_manager.prepare_vm_disk(&base_image, &vm_dir_path, vm_name, 5)?;

    // Create mock SSH key
    let ssh_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC7RXIKhCmT... test@example.com";

    // Create cloud-init configuration
    let (user_data, meta_data) = CloudInitManager::create_cloud_init_config(
        vm_name,
        "test.local",
        ssh_key,
        "testuser",
        "UTC",
        "wheel",
        "systemctl disable cloud-init",
    )?;

    // Create cloud-init ISO
    let iso_path =
        CloudInitManager::create_cloud_init_iso(&vm_dir_path, vm_name, &user_data, &meta_data)?;

    // Initialize VM (but don't create it)
    let vm = VirtualMachine::new(
        vm_name.to_string(),
        2,
        1024,
        5,
        disk_path.to_string_lossy().to_string(),
    );

    // Since we can't call generate_domain_xml() directly, we'll verify the
    // components we've prepared instead

    // Verify the preparation was successful
    assert!(disk_path.exists());
    assert!(iso_path.exists());

    // We can verify the VM setup parameters
    assert_eq!(vm.name, "test-integration-vm");
    assert_eq!(vm.vcpus, 2);
    assert_eq!(vm.memory_mb, 1024);
    assert_eq!(vm.disk_size_gb, 5);

    // For a more complete test that would require libvirt, we could:
    // 1. Connect to libvirt
    // 2. Create the VM
    // 3. Inspect the actual XML via virsh
    // But we'll skip that to keep this a non-libvirt test

    Ok(())
}
