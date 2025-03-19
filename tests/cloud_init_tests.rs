// tests/cloud_init_tests.rs
use anyhow::Result;
use kvm_install_vm::cloudinit::CloudInitManager;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_create_cloud_init_config() -> Result<()> {
    let (user_data, meta_data) = CloudInitManager::create_cloud_init_config(
        "test-vm",
        "example.local",
        "ssh-rsa AAAAB3NzaC1yc2E... test-key",
        "testuser",
        "UTC",
        "wheel",
        "systemctl disable cloud-init"
    )?;
    
    // Verify user_data contains expected elements
    assert!(user_data.contains("hostname: test-vm"));
    assert!(user_data.contains("fqdn: test-vm.example.local"));
    assert!(user_data.contains("name: testuser"));
    assert!(user_data.contains("groups: ['wheel']"));
    assert!(user_data.contains("ssh-rsa AAAAB3NzaC1yc2E... test-key"));
    assert!(user_data.contains("timezone: UTC"));
    
    // Verify meta_data contains expected elements
    assert!(meta_data.contains("instance-id: test-vm"));
    assert!(meta_data.contains("local-hostname: test-vm"));
    
    Ok(())
}

#[test]
fn test_create_cloud_init_iso() -> Result<()> {
    // Skip if neither genisoimage nor mkisofs is available
    let has_genisoimage = std::process::Command::new("genisoimage")
        .arg("--version")
        .output()
        .is_ok();
    
    let has_mkisofs = std::process::Command::new("mkisofs")
        .arg("--version")
        .output()
        .is_ok();
    
    if !has_genisoimage && !has_mkisofs {
        println!("Skipping test_create_cloud_init_iso: neither genisoimage nor mkisofs available");
        return Ok(());
    }
    
    let temp_dir = tempdir()?;
    
    // Create test cloud-init data
    let user_data = "#cloud-config\nhostname: test-vm\n";
    let meta_data = "instance-id: test-vm\nlocal-hostname: test-vm\n";
    
    // Create ISO
    let iso_path = CloudInitManager::create_cloud_init_iso(
        temp_dir.path(),
        "test-vm",
        user_data,
        meta_data
    )?;
    
    // Verify ISO was created
    assert!(iso_path.exists());
    assert!(fs::metadata(&iso_path)?.len() > 0);
    
    // Verify the temporary files were cleaned up
    assert!(!temp_dir.path().join("user-data").exists());
    assert!(!temp_dir.path().join("meta-data").exists());
    
    Ok(())
}

#[test]
fn test_find_ssh_public_key() {
    // This test is tricky because it depends on the local environment
    // Just check that the function either succeeds or fails with a reasonable error
    match CloudInitManager::find_ssh_public_key() {
        Ok(key) => {
            assert!(!key.is_empty());
            assert!(key.starts_with("ssh-"));
        },
        Err(e) => {
            // The error should mention SSH keys
            assert!(format!("{}", e).contains("SSH"));
        }
    }
}