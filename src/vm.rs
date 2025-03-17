use anyhow::{Result, Context};
use std::path::Path;
use virt::connect::Connect;
use virt::domain::Domain;

pub struct VirtualMachine {
    name: String,
    vcpus: u32,
    memory_mb: u32,
    disk_size_gb: u32,
    disk_path: String,
    // distro: String,
    connection: Option<Connect>,
}

impl VirtualMachine {
    pub fn new(name: String, vcpus: u32, memory_mb: u32, disk_size_gb: u32, disk_path: String) -> Self {
        VirtualMachine {
            name,
            vcpus,
            memory_mb,
            disk_path,
            disk_size_gb,
            // distro,
            connection: None,
        }
    }

    pub fn connect(&mut self, uri: Option<&str>) -> Result<()> {
        // Connect to libvirt daemon, default to "qemu:///system" if no URI provided
        let uri = uri.or(Some("qemu:///system"));
        self.connection = Some(Connect::open(uri).context("Failed to connect to libvirt")?);
        Ok(())
    }

    pub fn create(&mut self) -> Result<Domain> {
        // Ensure connection is established
        let conn = match &self.connection {
            Some(c) => c,
            None => return Err(anyhow::anyhow!("Connection not established")),
        };

        if !Path::new(&self.disk_path).exists() {
            self.create_disk_image()?;
        }

        let xml = self.generate_domain_xml()?;

        let domain = Domain::define_xml(&conn, &xml).context("Failed to define domain from XML")?;
        domain.create().context("Failed to start the domain")?;

        Ok(domain)
    }

    fn create_disk_image(&self) -> Result<()> {
        // Create disk image using qemu-img
        let output = std::process::Command::new("qemu-img")
            .args(&["create", "-f", "qcow2", &self.disk_path, &format!("{}G", self.disk_size_gb)])
            .output()
            .context("Failed to execute qemu-img command")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to create disk image: {:?}", output.stderr));
        }

        Ok(())
    }

    fn generate_domain_xml(&self) -> Result<String> {
        // Generate domain XML
        let xml = format!( r#"
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
        "#, self.name, self.memory_mb, self.vcpus, self.disk_path);

        Ok(xml)
    }
}
