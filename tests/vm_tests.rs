#[cfg(test)]
mod tests {
    use anyhow::Result;
    use kvm_install_vm::vm::VirtualMachine;
    use std::fs;
    use std::process::Command;
    use virt::domain::Domain;

    // Test fixture struct that implements Drop for guaranteed cleanup
    struct VMTestFixture {
        name: String,
        disk_path: String,
    }

    impl VMTestFixture {
        fn new(name: &str) -> Self {
            let temp_dir = std::env::temp_dir();
            let disk_path = temp_dir
                .join(format!("{}.qcow2", name))
                .to_string_lossy()
                .to_string();

            // Perform initial cleanup to ensure we start with a clean state
            Self::cleanup_vm(name, &disk_path);

            VMTestFixture {
                name: name.to_string(),
                disk_path,
            }
        }

        // Static method to clean up VM and disk
        fn cleanup_vm(name: &str, disk_path: &str) {
            println!("Cleaning up VM: {}", name);

            // Force destroy the domain if it's running
            let _ = Command::new("virsh").args(["destroy", name]).output();

            // Undefine the domain with all related resources
            let _ = Command::new("virsh")
                .args([
                    "undefine",
                    name,
                    "--managed-save",
                    "--snapshots-metadata",
                    "--nvram",
                ])
                .output();

            // Remove disk if it exists
            if std::path::Path::new(disk_path).exists() {
                let _ = fs::remove_file(disk_path);
                println!("Removed disk: {}", disk_path);
            }
        }
    }

    // Drop implementation ensures cleanup happens even if tests panic or fail
    impl Drop for VMTestFixture {
        fn drop(&mut self) {
            Self::cleanup_vm(&self.name, &self.disk_path);
        }
    }

    // Helper function to check if a VM with a given name exists
    fn domain_exists(name: &str) -> bool {
        let output = Command::new("virsh")
            .args(["list", "--all", "--name"])
            .output()
            .expect("Failed to execute virsh command");

        let output_str = String::from_utf8_lossy(&output.stdout);
        output_str.lines().any(|line| line.trim() == name)
    }

    // Custom function to generate domain XML for testing
    fn generate_test_domain_xml(vm: &VirtualMachine) -> String {
        format!(
            r#"
        <domain type='kvm'>
          <name>{}</name>
          <memory unit='MiB'>{}</memory>
          <vcpu>{}</vcpu>
          <os>
            <type arch='x86_64'>hvm</type>
            <boot dev='hd'/>
          </os>
          <features>
            <acpi/>
            <apic/>
          </features>
          <devices>
            <disk type='file' device='disk'>
              <driver name='qemu' type='qcow2'/>
              <source file='{}'/>
              <target dev='vda' bus='virtio'/>
            </disk>
            <interface type='network'>
              <source network='default'/>
              <model type='virtio'/>
            </interface>
            <console type='pty'/>
            <graphics type='vnc' port='-1'/>
          </devices>
        </domain>
        "#,
            vm.name, vm.memory_mb, vm.vcpus, vm.disk_path
        )
    }

    // Extract disk paths function for testing
    fn extract_disk_paths_from_xml(xml: &str) -> Vec<String> {
        let mut disk_paths = Vec::new();

        for line in xml.lines() {
            if line.contains("<source file=") {
                if let Some(start) = line.find("file='") {
                    if let Some(end) = line[start + 6..].find('\'') {
                        disk_paths.push(line[start + 6..start + 6 + end].to_string());
                    }
                } else if let Some(start) = line.find("file=\"") {
                    if let Some(end) = line[start + 6..].find('\"') {
                        disk_paths.push(line[start + 6..start + 6 + end].to_string());
                    }
                }
            }
        }

        disk_paths
    }

    // Helper function to create a test VM for later destruction tests
    fn create_test_vm(name: &str, fixture: &VMTestFixture) -> Result<()> {
        // Skip if domain already exists
        if domain_exists(name) {
            return Ok(());
        }

        // Create a minimal VM for testing
        let mut vm = VirtualMachine::new(name.to_string(), 1, 512, 1, fixture.disk_path.clone());

        vm.connect(None)?;

        // Create disk if it doesn't exist
        if !std::path::Path::new(&fixture.disk_path).exists() {
            Command::new("qemu-img")
                .args(["create", "-f", "qcow2", &fixture.disk_path, "1G"])
                .output()
                .expect("Failed to create test disk image");
        }

        // Define the domain but don't start it (to keep tests faster)
        let conn = vm.connection.as_ref().unwrap();
        let xml = generate_test_domain_xml(&vm);
        Domain::define_xml(conn, &xml)?;

        Ok(())
    }

    #[test]
    fn test_create_new_vm_instance() {
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

    #[test]
    fn test_extract_disk_paths() {
        let xml = r#"
        <domain type='kvm'>
          <name>test-vm</name>
          <devices>
            <disk type='file' device='disk'>
              <driver name='qemu' type='qcow2'/>
              <source file='/path/to/disk1.qcow2'/>
              <target dev='vda' bus='virtio'/>
            </disk>
            <disk type='file' device='disk'>
              <driver name='qemu' type='qcow2'/>
              <source file="/path/to/disk2.qcow2"/>
              <target dev='vdb' bus='virtio'/>
            </disk>
          </devices>
        </domain>
        "#;

        let disk_paths = extract_disk_paths_from_xml(xml);

        assert_eq!(disk_paths.len(), 2);
        assert!(disk_paths.contains(&"/path/to/disk1.qcow2".to_string()));
        assert!(disk_paths.contains(&"/path/to/disk2.qcow2".to_string()));
    }

    // This test requires libvirt to be running
    // Use #[ignore] to skip it in normal test runs
    #[test]
    #[ignore]
    fn test_connect_to_libvirt() -> Result<()> {
        // Create fixture first to ensure cleanup
        let fixture = VMTestFixture::new("test-connect-vm");

        let mut vm = VirtualMachine::new(
            "test-connect-vm".to_string(),
            1,
            512,
            1,
            fixture.disk_path.clone(),
        );

        vm.connect(None)?;
        assert!(vm.connection.is_some());

        Ok(())
    }

    // This test creates and then destroys a VM
    // It's marked as ignored because it makes actual system changes
    #[test]
    #[ignore]
    fn test_create_and_destroy_vm() -> Result<()> {
        let test_name = "test-create-destroy-vm";
        // Create fixture first to ensure cleanup happens automatically
        let fixture = VMTestFixture::new(test_name);

        // Create a new VM
        let mut vm =
            VirtualMachine::new(test_name.to_string(), 1, 512, 1, fixture.disk_path.clone());

        vm.connect(None)?;

        // Create disk if it doesn't exist
        if !std::path::Path::new(&fixture.disk_path).exists() {
            Command::new("qemu-img")
                .args(["create", "-f", "qcow2", &fixture.disk_path, "1G"])
                .output()
                .expect("Failed to create test disk image");
        }

        // Create the VM via helper function since we can't call the private method
        let conn = vm.connection.as_ref().unwrap();
        let xml = generate_test_domain_xml(&vm);
        let domain = Domain::define_xml(conn, &xml)?;
        domain.create()?;

        // Verify it exists
        assert!(domain_exists(test_name));

        // Now destroy it
        vm.destroy_instance(false)?;

        // Verify it no longer exists
        assert!(!domain_exists(test_name));

        // Disk should still exist since we used remove_disk=false
        assert!(std::path::Path::new(&fixture.disk_path).exists());

        // Note: We don't need manual cleanup here,
        // the fixture's Drop implementation will handle it

        Ok(())
    }

    // Test destroying a VM with the static method
    #[test]
    #[ignore]
    fn test_destroy_static_method() -> Result<()> {
        let test_name = "test-static-destroy-vm";
        // Create fixture first to ensure cleanup happens automatically
        let fixture = VMTestFixture::new(test_name);

        // Create a test VM first
        create_test_vm(test_name, &fixture)?;

        // Verify it exists
        assert!(domain_exists(test_name));

        // Destroy it with the static method
        VirtualMachine::destroy(test_name, None, true)?;

        // Verify it no longer exists
        assert!(!domain_exists(test_name));

        // Disk should be gone since we used remove_disk=true
        assert!(!std::path::Path::new(&fixture.disk_path).exists());

        Ok(())
    }

    // Test destroying a non-existent VM (should return error)
    #[test]
    fn test_destroy_nonexistent_vm() {
        // Still create a fixture to ensure any leftovers are cleaned up
        let fixture = VMTestFixture::new("definitely-nonexistent-vm");

        let result = VirtualMachine::destroy(&fixture.name, None, false);
        assert!(result.is_err());
    }
}
