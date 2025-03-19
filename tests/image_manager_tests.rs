// tests/image_manager_tests.rs - Fixed version
use anyhow::Result;
use kvm_install_vm::vm::{DistroInfo, ImageManager};
use std::fs;
use tempfile::tempdir;
use tokio::runtime::Runtime;

// Helper function to create a test distro info
fn create_test_distro() -> DistroInfo {
    DistroInfo {
        qcow_filename: "test-image.qcow2".to_string(),
        os_variant: "test-os".to_string(),
        image_url: "https://example.com/images".to_string(),
        login_user: "testuser".to_string(),
        sudo_group: "wheel".to_string(),
        cloud_init_disable: "systemctl disable cloud-init".to_string(),
    }
}

#[test]
fn test_image_manager_initialization() {
    let temp_dir = tempdir().unwrap();
    let image_manager = ImageManager::new(temp_dir.path());
    
    // Test that the image directory is set correctly
    assert_eq!(
        image_manager.get_image_path(&create_test_distro()),
        temp_dir.path().join("test-image.qcow2")
    );
}

#[test]
fn test_image_existence_check() -> Result<()> {
    let temp_dir = tempdir()?;
    let image_manager = ImageManager::new(temp_dir.path());
    let distro = create_test_distro();
    
    // Initially, the image should not exist
    assert!(!image_manager.image_exists(&distro));
    
    // Create an empty file at the image path
    let image_path = temp_dir.path().join("test-image.qcow2");
    fs::write(&image_path, b"test content")?;
    
    // Now the image should be reported as existing
    assert!(image_manager.image_exists(&distro));
    
    Ok(())
}

// This test is marked as ignored because it would attempt to download 
// a real cloud image, which we don't want in automated testing
#[test]
#[ignore]
fn test_ensure_image() -> Result<()> {
    let rt = Runtime::new()?;
    let temp_dir = tempdir()?;
    let image_manager = ImageManager::new(temp_dir.path());
    
    // This distro has a real URL that would be downloaded
    let distro = DistroInfo {
        qcow_filename: "cirros-0.5.1-x86_64-disk.img".to_string(),
        os_variant: "cirros".to_string(),
        image_url: "http://download.cirros-cloud.net/0.5.1".to_string(),
        login_user: "cirros".to_string(),
        sudo_group: "wheel".to_string(),
        cloud_init_disable: "systemctl disable cloud-init".to_string(),
    };
    
    // Use the tokio runtime to run the async function
    let image_path = rt.block_on(image_manager.ensure_image(&distro))?;
    
    // Verify the image was downloaded
    assert!(image_path.exists());
    assert!(fs::metadata(&image_path)?.len() > 0);
    
    Ok(())
}

#[test]
fn test_prepare_vm_disk() -> Result<()> {
    // Skip if qemu-img is not available
    if std::process::Command::new("qemu-img").arg("--version").output().is_err() {
        println!("Skipping test_prepare_vm_disk: qemu-img not available");
        return Ok(());
    }
    
    let temp_dir = tempdir()?;
    let image_manager = ImageManager::new(temp_dir.path());
    
    // Create a dummy base image
    let base_image = temp_dir.path().join("base.qcow2");
    std::process::Command::new("qemu-img")
        .args(&["create", "-f", "qcow2", &base_image.to_string_lossy(), "1G"])
        .output()?;
    
    // Create VM directory
    let vm_dir = temp_dir.path().join("vms/test-vm");
    fs::create_dir_all(&vm_dir)?;
    
    // Prepare VM disk
    let disk_path = image_manager.prepare_vm_disk(
        &base_image,
        &vm_dir,
        "test-vm",
        5 // 5GB disk
    )?;
    
    // Verify the disk was created
    assert!(disk_path.exists());
    
    // Check disk properties with qemu-img info
    let output = std::process::Command::new("qemu-img")
        .args(&["info", "--output=json", &disk_path.to_string_lossy()])
        .output()?;
    
    let info_str = String::from_utf8(output.stdout)?;
    assert!(info_str.contains("qcow2"));
    
    // Fix: Convert Cow to a regular str for comparison
    let base_path_str = base_image.to_string_lossy().to_string();
    assert!(info_str.contains(&base_path_str));
    
    Ok(())
}