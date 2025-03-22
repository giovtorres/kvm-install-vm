use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info, instrument, trace, warn};
use virt::connect::Connect;
use virt::domain::Domain;
use virt::sys;

use crate::config::Config;
use crate::domain::{DomainInfo, DomainState, extract_disk_paths_from_xml};
use crate::cloudinit::CloudInitManager;

pub struct VirtualMachine {
    pub name: String,
    pub vcpus: u32,
    pub memory_mb: u32,
    pub disk_size_gb: u32,
    pub disk_path: String,
    pub connection: Option<Connect>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DistroInfo {
    pub qcow_filename: String,
    pub os_variant: String,
    pub image_url: String,
    pub login_user: String,
    pub sudo_group: String,
    pub cloud_init_disable: String,
}

pub struct ImageManager {
    image_dir: PathBuf,
}

impl ImageManager {
    /// Create a new ImageManager with the specified image directory
    pub fn new<P: AsRef<Path>>(image_dir: P) -> Self {
        ImageManager {
            image_dir: image_dir.as_ref().to_path_buf(),
        }
    }

    /// Check if a cloud image exists locally
    pub fn image_exists(&self, distro_info: &DistroInfo) -> bool {
        let image_path = self.image_dir.join(&distro_info.qcow_filename);
        image_path.exists()
    }

    /// Get the full path to a cloud image (whether it exists or not)
    pub fn get_image_path(&self, distro_info: &DistroInfo) -> PathBuf {
        self.image_dir.join(&distro_info.qcow_filename)
    }

    /// Download a cloud image if it doesn't already exist locally
    pub async fn ensure_image(&self, distro_info: &DistroInfo) -> Result<PathBuf> {
        let image_path = self.get_image_path(distro_info);

        if image_path.exists() {
            info!("Cloud image already exists: {}", image_path.display());
            println!("Cloud image already exists: {}", image_path.display());
            return Ok(image_path);
        }

        // Create image directory if it doesn't exist
        if !self.image_dir.exists() {
            fs::create_dir_all(&self.image_dir).context("Failed to create image directory")?;
        }

        info!("Downloading cloud image: {}", distro_info.qcow_filename);
        println!("Downloading cloud image: {}", distro_info.qcow_filename);

        // Construct download URL
        let url = format!(
            "{}/{}",
            distro_info.image_url.trim_end_matches('/'),
            distro_info.qcow_filename
        );

        debug!("From URL: {}", url);
        println!("From URL: {}", url);

        // Download the file with progress indication
        self.download_file(&url, &image_path)
            .await
            .context("Failed to download cloud image")?;

        Ok(image_path)
    }

    /// Download a file with progress indication
    async fn download_file(&self, url: &str, dest: &Path) -> Result<PathBuf> {
        // Create a temporary file for downloading
        let temp_path = dest.with_extension("part");

        // Create parent directory if needed
        if let Some(parent) = temp_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Begin the download
        let res = reqwest::get(url).await?;
        let total_size = res.content_length().unwrap_or(0);

        // Setup progress bar
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"));

        // Download the file in chunks, writing each chunk to disk
        let mut file = File::create(&temp_path).await?;
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        // Ensure everything is written to disk
        file.flush().await?;

        // Finalize the download by renaming the temp file
        tokio::fs::rename(&temp_path, &dest).await?;

        pb.finish_with_message(format!("Downloaded {}", dest.display()));

        Ok(dest.to_path_buf())
    }
    
    /// Download a cloud image with resume capability
    #[instrument(skip(self), fields(distro = %distro_info.qcow_filename))]
    pub async fn download_image_with_resume(&self, distro_info: &DistroInfo) -> Result<PathBuf> {
        let image_path = self.image_dir.join(&distro_info.qcow_filename);
        let part_path = image_path.with_extension("part");
        
        // Create image directory if it doesn't exist
        if !self.image_dir.exists() {
            fs::create_dir_all(&self.image_dir).context("Failed to create image directory")?;
        }
        
        // Check if the image already exists
        if image_path.exists() {
            info!("Cloud image already exists: {}", image_path.display());
            println!("Cloud image already exists: {}", image_path.display());
            return Ok(image_path);
        }
        
        // Construct download URL
        let url = format!(
            "{}/{}",
            distro_info.image_url.trim_end_matches('/'),
            distro_info.qcow_filename
        );
        
        info!("Downloading cloud image: {}", distro_info.qcow_filename);
        println!("Downloading cloud image: {}", distro_info.qcow_filename);
        debug!("From URL: {}", url);
        
        // Check if partial download exists
        let resume_download = part_path.exists();
        if resume_download {
            info!("Partial download found. Resuming from previous download");
            println!("Partial download found. Resuming from previous download");
            
            let client = reqwest::Client::new();
            let file_size = part_path.metadata()?.len();
            
            debug!("Resuming from byte position: {}", file_size);
            
            // Create a request with Range header
            let mut req = client.get(&url);
            req = req.header("Range", format!("bytes={}-", file_size));
            
            // Download the rest of the file
            let res = req.send().await?;
            
            // Check if the server supports resume
            if res.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                let total_size = match res.content_length() {
                    Some(len) => file_size + len,
                    None => file_size, // Just show the current size if total is unknown
                };
                
                // Setup progress bar
                let pb = ProgressBar::new(total_size);
                pb.set_style(ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .unwrap()
                    .progress_chars("#>-"));
                pb.set_position(file_size);
                
                // Open the existing part file for appending
                let mut file = tokio::fs::OpenOptions::new()
                    .append(true)
                    .open(&part_path)
                    .await?;
                
                let mut downloaded = file_size;
                let mut stream = res.bytes_stream();
                
                while let Some(item) = stream.next().await {
                    let chunk = item?;
                    file.write_all(&chunk).await?;
                    downloaded += chunk.len() as u64;
                    pb.set_position(downloaded);
                }
                
                // Ensure everything is written to disk
                file.flush().await?;
                
                // Finalize the download by renaming the temp file
                tokio::fs::rename(&part_path, &image_path).await?;
                
                pb.finish_with_message(format!("Downloaded {}", image_path.display()));
                
                return Ok(image_path);
            } else {
                warn!("Server does not support resume. Starting a new download");
                println!("Server does not support resume. Starting a new download");
            }
        }
        
        // If we got here, we need to do a full download
        self.download_file(&url, &image_path).await?;
        
        Ok(image_path)
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
            connection: None,
        }
    }

    #[instrument(skip(self), fields(vm_name = %self.name))]
    pub fn connect(&mut self, uri: Option<&str>) -> Result<()> {
        // Connect to libvirt daemon, default to "qemu:///session" if no URI provided
        let uri = uri.or(Some("qemu:///session"));
        debug!("Connecting to libvirt with URI: {:?}", uri);

        match Connect::open(uri) {
            Ok(conn) => {
                debug!("Successfully connected to libvirt");
                self.connection = Some(conn);
                info!("Connected to libvirt daemon");
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to libvirt: {}", e);
                Err(anyhow::anyhow!("Failed to connect to libvirt: {}", e))
            }
        }
    }

    // Prepare the VM image for creation
    #[instrument(skip(self, config), fields(vm_name = %self.name))]
    pub async fn prepare_image(&mut self, distro: &str, config: &Config) -> Result<()> {
        info!("Preparing image for VM: {}", self.name);
        
        // Get distribution info
        let distro_info = config.get_distro(distro)?;
        debug!("Using distro: {}", distro);
        
        // Setup image manager
        let image_dir = PathBuf::from(&config.defaults.image_dir);
        let image_manager = ImageManager::new(&image_dir);
        
        // Ensure we have the cloud image
        info!("Checking for cloud image");
        let cloud_image = image_manager.ensure_image(distro_info).await?;
        debug!("Cloud image path: {}", cloud_image.display());
        
        // Create VM directory if it doesn't exist
        let vm_dir = PathBuf::from(&config.defaults.vm_dir).join(&self.name);
        if !vm_dir.exists() {
            fs::create_dir_all(&vm_dir).context("Failed to create VM directory")?;
        }
        
        // Create disk path for the VM
        self.disk_path = vm_dir.join(format!("{}.qcow2", self.name))
            .to_string_lossy()
            .to_string();
        debug!("Disk path: {}", self.disk_path);
        
        // Create disk image from the cloud image
        info!("Creating disk image for VM");
        let mut cmd = Command::new("qemu-img");
        cmd.args([
            "create",
            "-f", "qcow2",
            "-F", "qcow2",
            "-b", cloud_image.to_str().unwrap(),
            &self.disk_path,
        ]);
        
        debug!("Running command: {:?}", cmd);
        let status = cmd.status().context("Failed to execute qemu-img command")?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("Failed to create disk image"));
        }
        
        // Resize disk if needed
        if self.disk_size_gb > 10 {
            info!("Resizing disk to {}GB", self.disk_size_gb);
            let mut resize_cmd = Command::new("qemu-img");
            resize_cmd.args([
                "resize",
                &self.disk_path,
                &format!("{}G", self.disk_size_gb),
            ]);
            
            debug!("Running command: {:?}", resize_cmd);
            let resize_status = resize_cmd.status().context("Failed to resize disk")?;
            
            if !resize_status.success() {
                return Err(anyhow::anyhow!("Failed to resize disk image"));
            }
        }
        
        // Create cloud-init configuration
        info!("Creating cloud-init configuration");
        let ssh_key = CloudInitManager::find_ssh_public_key()?;
        
        let (user_data, meta_data) = CloudInitManager::create_cloud_init_config(
            &self.name,
            &config.defaults.dns_domain,
            &ssh_key,
            &distro_info.login_user,
            &config.defaults.timezone,
            &distro_info.sudo_group,
            &distro_info.cloud_init_disable,
        )?;
        
        // Create cloud-init ISO
        let iso_path = CloudInitManager::create_cloud_init_iso(
            &vm_dir,
            &self.name,
            &user_data,
            &meta_data,
        )?;
        debug!("Cloud-init ISO created at: {}", iso_path.display());
        
        info!("Image preparation completed successfully");
        Ok(())
    }

    #[instrument(skip(self), fields(vm_name = %self.name))]
    pub fn create(&mut self) -> Result<Domain> {
        info!("Creating VM: {}", self.name);
        debug!(
            "VM parameters: vcpus={}, memory={}MB, disk={}GB, disk_path={}",
            self.vcpus, self.memory_mb, self.disk_size_gb, self.disk_path
        );

        // Ensure connection is established
        let conn = match &self.connection {
            Some(c) => {
                debug!("Using existing libvirt connection");
                c
            }
            None => {
                error!("Connection not established before create() call");
                return Err(anyhow::anyhow!("Connection not established"));
            }
        };

        // Check if disk image exists and create if needed
        if !Path::new(&self.disk_path).exists() {
            debug!("Disk image doesn't exist, creating it");
            self.create_disk_image()?;
        } else {
            debug!("Using existing disk image: {}", self.disk_path);
        }

        // Generate XML definition
        debug!("Generating domain XML definition");
        let xml = self.generate_domain_xml()?;
        trace!("Generated XML: {}", xml);

        // Define domain from XML
        debug!("Defining domain from XML");
        let domain = match Domain::define_xml(&conn, &xml) {
            Ok(d) => {
                info!("Domain defined successfully");
                d
            }
            Err(e) => {
                error!("Failed to define domain from XML: {}", e);
                return Err(anyhow::anyhow!("Failed to define domain from XML: {}", e));
            }
        };

        // Start the domain
        debug!("Starting the domain");
        match domain.create() {
            Ok(_) => {
                info!("Domain started successfully");
            }
            Err(e) => {
                error!("Failed to start the domain: {}", e);
                return Err(anyhow::anyhow!("Failed to start the domain: {}", e));
            }
        };

        info!("VM creation completed successfully");
        Ok(domain)
    }

    fn create_disk_image(&self) -> Result<()> {
        info!("Creating disk image: {}", self.disk_path);
        debug!("Disk size: {}GB, Format: qcow2", self.disk_size_gb);

        // Create parent directory if it doesn't exist
        if let Some(parent) = Path::new(&self.disk_path).parent() {
            if !parent.exists() {
                debug!("Creating parent directory: {}", parent.display());
                fs::create_dir_all(parent)
                    .context(format!("Failed to create directory: {}", parent.display()))?;
            }
        }

        // Build the command
        let mut cmd = Command::new("qemu-img");
        cmd.args(&[
            "create",
            "-f",
            "qcow2",
            &self.disk_path,
            &format!("{}G", self.disk_size_gb),
        ]);

        debug!("Executing command: {:?}", cmd);

        // Execute the command
        let output = cmd
            .output()
            .context("Failed to execute qemu-img command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to create disk image: {}", stderr);
            return Err(anyhow::anyhow!("Failed to create disk image: {}", stderr));
        }

        info!("Successfully created disk image");
        Ok(())
    }

    fn generate_domain_xml(&self) -> Result<String> {
        // Generate domain XML with proper name tag
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
            <interface type='user'>
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
            info!("Stopping running domain '{}'...", name);
            println!("Stopping running domain '{}'...", name);
            match domain.destroy() {
                Ok(_) => {
                    info!("Domain stopped successfully");
                    println!("Domain stopped successfully");
                },
                Err(e) => {
                    warn!("Warning: Failed to stop domain cleanly: {}. Continuing with undefine...", e);
                    println!("Warning: Failed to stop domain cleanly: {}. Continuing with undefine...", e);
                },
            }
        } else {
            info!("Domain '{}' is already stopped", name);
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

        info!("Domain {} successfully undefined", name);
        println!("Domain {} successfully undefined", name);

        // Handle disk removal if requested
        if remove_disk && !disk_paths.is_empty() {
            info!("Removing disk images...");
            println!("Removing disk images...");
            for path in &disk_paths {
                match std::fs::remove_file(path) {
                    Ok(_) => {
                        info!("Successfully removed disk: {}", path);
                        println!("Successfully removed disk: {}", path);
                    },
                    Err(e) => {
                        warn!("Warning: Failed to remove disk {}: {}", path, e);
                        println!("Warning: Failed to remove disk {}: {}", path, e);
                    },
                }
            }
        } else if !disk_paths.is_empty() {
            info!("Note: The following disk images were not deleted:");
            println!("Note: The following disk images were not deleted:");
            for path in &disk_paths {
                info!("  - {}", path);
                println!("  - {}", path);
            }
        }

        info!("Domain {} completely destroyed", name);
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
            let id = domain.get_id();
            
            // Get state
            let state = match domain.get_state() {
                Ok((state, _)) => {
                    match state {
                        virt::sys::VIR_DOMAIN_RUNNING => DomainState::Running,
                        virt::sys::VIR_DOMAIN_PAUSED => DomainState::Paused,
                        virt::sys::VIR_DOMAIN_SHUTDOWN => DomainState::Shutdown,
                        virt::sys::VIR_DOMAIN_SHUTOFF => DomainState::Shutoff,
                        virt::sys::VIR_DOMAIN_CRASHED => DomainState::Crashed,
                        _ => DomainState::Unknown,
                    }
                }
                Err(_) => DomainState::Unknown,
            };

            domain_infos.push(DomainInfo {
                id,
                name,
                state,
            });
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

        Ok(domain_infos)
    }

    pub fn print_domain_list(uri: Option<&str>, show_all: bool, show_running: bool, show_inactive: bool) -> Result<()> {
        // Get domain list
        let domains = Self::list_domains(uri)?;

        if domains.is_empty() {
            println!("No domains found.");
            return Ok(());
        }

        // Print header
        println!("{:<5} {:<20} {:<10}", "ID", "Name", "State");
        println!("{:<5} {:<20} {:<10}", "-----", "--------------------", "----------");

        // Print domain information
        for domain in domains {
            let id = match domain.id {
                Some(id) => id.to_string(),
                None => "-".to_string(),
            };

            let is_running = domain.state == DomainState::Running;
            let is_inactive = domain.state == DomainState::Shutoff;

            if show_all || 
               (show_running && is_running) || 
               (show_inactive && is_inactive) {
                println!("{:<5} {:<20} {:<10}", id, domain.name, domain.state);
            }
        }

        Ok(())
    }
}