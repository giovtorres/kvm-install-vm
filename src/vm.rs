use anyhow::{Context, Result};
use std::fmt;
use std::path::Path;
use virt::connect::Connect;
use virt::domain::Domain;
use virt::sys;

pub struct VirtualMachine {
    pub name: String,
    pub vcpus: u32,
    pub memory_mb: u32,
    pub disk_size_gb: u32,
    pub disk_path: String,
    // distro: String,
    pub connection: Option<Connect>,
}

#[derive(Debug)]
pub struct DomainInfo {
    pub id: Option<u32>, // None if domain is inactive
    pub name: String,
    pub state: DomainState,
}

#[derive(Debug, PartialEq)]
pub enum DomainState {
    Running,
    Paused,
    Shutdown,
    Shutoff,
    Crashed,
    Unknown,
}

// Implement Display for DomainState for nice formatting
impl fmt::Display for DomainState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainState::Running => write!(f, "running"),
            DomainState::Paused => write!(f, "paused"),
            DomainState::Shutdown => write!(f, "shutdown"),
            DomainState::Shutoff => write!(f, "shut off"),
            DomainState::Crashed => write!(f, "crashed"),
            DomainState::Unknown => write!(f, "unknown"),
        }
    }
}

impl VirtualMachine {
    pub fn new(
        name: String,
        vcpus: u32,
        memory_mb: u32,
        disk_size_gb: u32,
        disk_path: String,
    ) -> Self {
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
        // Connect to libvirt daemon, default to "qemu:///session" if no URI provided
        let uri = uri.or(Some("qemu:///session"));
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
            .args(&[
                "create",
                "-f",
                "qcow2",
                &self.disk_path,
                &format!("{}G", self.disk_size_gb),
            ])
            .output()
            .context("Failed to execute qemu-img command")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to create disk image: {:?}",
                output.stderr
            ));
        }

        Ok(())
    }

    fn generate_domain_xml(&self) -> Result<String> {
        // Generate domain XML
        let xml = format!(
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
            self.name, self.memory_mb, self.vcpus, self.disk_path
        );

        Ok(xml)
    }

    // Destroy method for instance
    pub fn destroy_instance(&mut self, remove_disk: bool) -> Result<()> {
        // Ensure connection is established
        if self.connection.is_none() {
            return Err(anyhow::anyhow!("Connection not established"));
        }

        Self::destroy(&self.name, Some("qemu:///session"), remove_disk)
    }

    // Static destroy method
    pub fn destroy(name: &str, uri: Option<&str>, remove_disk: bool) -> Result<()> {
        let uri = uri.or(Some("qemu:///session"));
        let conn = Connect::open(uri).context("Failed to connect to libvirt")?;

        let domain = match Domain::lookup_by_name(&conn, name) {
            Ok(dom) => dom,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to find domain {}: {}", name, e));
            }
        };

        // Extract disk paths before destroying the domain
        let xml = domain.get_xml_desc(0).context("Failed to get domain XML")?;
        let disk_paths = extract_disk_paths_from_xml(&xml);

        // Check domain state first
        if domain.is_active().context("Failed to check domain state")? {
            println!("Stopping running domain '{}'...", name);
            match domain.destroy() {
                Ok(_) => println!("Domain stopped successfully"),
                Err(e) => println!(
                    "Warning: Failed to stop domain cleanly: {}. Continuing with undefine...",
                    e
                ),
            }
        } else {
            println!("Domain '{}' is already stopped", name);
        }

        let flags = virt::sys::VIR_DOMAIN_UNDEFINE_MANAGED_SAVE
            | virt::sys::VIR_DOMAIN_UNDEFINE_SNAPSHOTS_METADATA
            | virt::sys::VIR_DOMAIN_UNDEFINE_NVRAM;

        unsafe {
            let result = sys::virDomainUndefineFlags(domain.as_ptr(), flags);
            if result < 0 {
                return Err(anyhow::anyhow!("Failed to undefine domain"));
            }
        }

        println!("Domain {} successfully undefined", name);

        // Handle disk removal if requested
        if remove_disk && !disk_paths.is_empty() {
            println!("Removing disk images...");
            for path in &disk_paths {
                match std::fs::remove_file(path) {
                    Ok(_) => println!("Successfully removed disk: {}", path),
                    Err(e) => println!("Warning: Failed to remove disk {}: {}", path, e),
                }
            }
        } else if !disk_paths.is_empty() {
            println!("Note: The following disk images were not deleted:");
            for path in &disk_paths {
                println!("  - {}", path);
            }
        }

        println!("Domain {} completely destroyed", name);
        Ok(())
    }

    pub fn list_domains(uri: Option<&str>) -> Result<Vec<DomainInfo>> {
        let uri = uri.or(Some("qemu:///session"));
        let conn = Connect::open(uri).context("Failed to connect to libvirt")?;

        let mut domain_infos = Vec::new();

        // Get active domains
        let active_domains = conn
            .list_all_domains(virt::sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE)
            .context("Failed to list active domains")?;

        // Get inactive domains
        let inactive_domains = conn
            .list_all_domains(virt::sys::VIR_CONNECT_LIST_DOMAINS_INACTIVE)
            .context("Failed to list inactive domains")?;

        // Process active domains
        for domain in active_domains {
            let name = domain.get_name().context("Failed to get domain name")?;
            // domain.get_id() already returns an Option<u32>, so we don't need .ok()
            let id = domain.get_id();

            // Get domain state
            let state = match domain.get_state() {
                Ok((state, _reason)) => match state {
                    virt::sys::VIR_DOMAIN_RUNNING => DomainState::Running,
                    virt::sys::VIR_DOMAIN_PAUSED => DomainState::Paused,
                    virt::sys::VIR_DOMAIN_SHUTDOWN => DomainState::Shutdown,
                    virt::sys::VIR_DOMAIN_SHUTOFF => DomainState::Shutoff,
                    virt::sys::VIR_DOMAIN_CRASHED => DomainState::Crashed,
                    _ => DomainState::Unknown,
                },
                Err(_) => DomainState::Unknown,
            };

            domain_infos.push(DomainInfo { id, name, state });
        }

        // Process inactive domains
        for domain in inactive_domains {
            let name = domain.get_name().context("Failed to get domain name")?;

            domain_infos.push(DomainInfo {
                id: None,
                name,
                state: DomainState::Shutoff,
            });
        }

        // Sort domains by name for consistent output
        domain_infos.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(domain_infos)
    }

    /// Pretty print the list of domains with filtering options
    pub fn print_domain_list(
        uri: Option<&str>,
        show_all: bool,
        show_running: bool,
        show_inactive: bool,
    ) -> Result<()> {
        let domains = Self::list_domains(uri)?;

        if domains.is_empty() {
            println!("No domains found");
            return Ok(());
        }

        // Determine filtering logic
        let use_filters = !show_all && (show_running || show_inactive);

        // Filter domains based on flags if needed
        let filtered_domains: Vec<_> = if use_filters {
            domains
                .into_iter()
                .filter(|domain| {
                    (show_running && domain.state == DomainState::Running)
                        || (show_inactive && domain.id.is_none())
                })
                .collect()
        } else {
            domains
        };

        if filtered_domains.is_empty() {
            println!("No domains found matching the specified criteria");
            return Ok(());
        }

        // Print header
        println!("{:<5} {:<30} {:<10}", "ID", "Name", "State");
        println!("{:-<5} {:-<30} {:-<10}", "", "", "");

        // Print domains
        for domain in filtered_domains {
            let id_str = match domain.id {
                Some(id) => id.to_string(),
                None => "-".to_string(),
            };

            println!("{:<5} {:<30} {:<10}", id_str, domain.name, domain.state);
        }

        Ok(())
    }
}

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
