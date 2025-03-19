// tests/config_tests.rs
use anyhow::Result;
use kvm_install_vm::config::{Config, DefaultConfig};
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn test_config_defaults() {
    let config = Config::default();
    
    // Check that default distributions exist
    assert!(config.distros.contains_key("centos8"));
    assert!(config.distros.contains_key("ubuntu2004"));
    
    // Check default settings
    assert_eq!(config.defaults.memory_mb, 1024);
    assert_eq!(config.defaults.vcpus, 1);
    assert_eq!(config.defaults.disk_size_gb, 10);
}

#[test]
fn test_config_serialization_deserialization() -> Result<()> {
    // Create a temporary directory for the test
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("test-config.toml");
    
    // Create a test configuration
    let mut test_distros = HashMap::new();
    test_distros.insert("test-distro".to_string(), kvm_install_vm::vm::DistroInfo {
        qcow_filename: "test-image.qcow2".to_string(),
        os_variant: "test-os".to_string(),
        image_url: "https://example.com/images".to_string(),
        login_user: "testuser".to_string(),
        sudo_group: "wheel".to_string(),
        cloud_init_disable: "systemctl disable cloud-init".to_string(),
    });
    
    let test_defaults = DefaultConfig {
        memory_mb: 2048,
        vcpus: 2,
        disk_size_gb: 20,
        image_dir: "/tmp/test-images".to_string(),
        vm_dir: "/tmp/test-vms".to_string(),
        dns_domain: "test.local".to_string(),
        timezone: "UTC".to_string(),
    };
    
    let original_config = Config {
        distros: test_distros,
        defaults: test_defaults,
    };
    
    // Save and then reload the configuration
    original_config.save_to_file(&config_path)?;
    let loaded_config = Config::from_file(&config_path)?;
    
    // Verify the loaded configuration matches the original
    assert_eq!(loaded_config.distros.len(), original_config.distros.len());
    assert!(loaded_config.distros.contains_key("test-distro"));
    assert_eq!(
        loaded_config.distros["test-distro"].qcow_filename,
        "test-image.qcow2"
    );
    assert_eq!(loaded_config.defaults.memory_mb, 2048);
    assert_eq!(loaded_config.defaults.vcpus, 2);
    
    Ok(())
}

#[test]
fn test_config_from_toml_string() -> Result<()> {
    // Define a test TOML configuration
    let toml_str = r#"
        [distros.test-distro]
        qcow_filename = "test-image.qcow2"
        os_variant = "test-os"
        image_url = "https://example.com/images"
        login_user = "testuser"
        sudo_group = "wheel"
        cloud_init_disable = "systemctl disable cloud-init"

        [defaults]
        memory_mb = 4096
        vcpus = 4
        disk_size_gb = 40
        image_dir = "/custom/images"
        vm_dir = "/custom/vms"
        dns_domain = "custom.local"
        timezone = "America/New_York"
    "#;
    
    // Parse the TOML string
    let config: Config = toml::from_str(toml_str)?;
    
    // Verify the configuration
    assert_eq!(config.distros.len(), 1);
    assert!(config.distros.contains_key("test-distro"));
    assert_eq!(config.defaults.memory_mb, 4096);
    assert_eq!(config.defaults.vcpus, 4);
    assert_eq!(config.defaults.timezone, "America/New_York");
    
    Ok(())
}

#[test]
fn test_get_distro() -> Result<()> {
    let config = Config::default();
    
    // Test existing distro
    let centos = config.get_distro("centos8")?;
    assert_eq!(centos.os_variant, "centos8");
    
    // Test non-existent distro
    let result = config.get_distro("nonexistent-distro");
    assert!(result.is_err());
    
    Ok(())
}