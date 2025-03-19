// cloudinit.rs - New file
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

/// Represents a cloud-init configuration manager
pub struct CloudInitManager;

impl CloudInitManager {
    /// Create cloud-init user-data and meta-data files
    pub fn create_cloud_init_config(
        vm_name: &str,
        dns_domain: &str,
        ssh_public_key: &str,
        user: &str,
        timezone: &str,
        sudo_group: &str,
        cloud_init_disable: &str,
    ) -> Result<(String, String)> {
        // Create meta-data content
        let meta_data = format!(
            "instance-id: {}\nlocal-hostname: {}\n",
            vm_name, vm_name
        );
        
        // Create user-data content with cloud-init multipart format
        let user_data = format!(
            r#"Content-Type: multipart/mixed; boundary="==BOUNDARY=="
MIME-Version: 1.0
--==BOUNDARY==
Content-Type: text/cloud-config; charset="us-ascii"

#cloud-config

# Hostname management
preserve_hostname: False
hostname: {hostname}
fqdn: {hostname}.{dns_domain}

# Users
users:
    - default
    - name: {user}
      groups: ['{sudo_group}']
      shell: /bin/bash
      sudo: ALL=(ALL) NOPASSWD:ALL
      ssh-authorized-keys:
        - {ssh_key}

# Configure where output will go
output:
  all: ">> /var/log/cloud-init.log"

# configure interaction with ssh server
ssh_genkeytypes: ['ed25519', 'rsa']

# Install my public ssh key to the user
ssh_authorized_keys:
  - {ssh_key}

timezone: {timezone}

# Remove cloud-init when finished with it
runcmd:
  - {cloud_init_disable}

--==BOUNDARY==--
"#,
            hostname = vm_name,
            dns_domain = dns_domain,
            user = user,
            ssh_key = ssh_public_key,
            sudo_group = sudo_group,
            timezone = timezone,
            cloud_init_disable = cloud_init_disable
        );
        
        Ok((user_data, meta_data))
    }
    
    /// Create a cloud-init ISO from user-data and meta-data
    pub fn create_cloud_init_iso(
        work_dir: &Path,
        vm_name: &str,
        user_data: &str,
        meta_data: &str
    ) -> Result<PathBuf> {
        let user_data_path = work_dir.join("user-data");
        let meta_data_path = work_dir.join("meta-data");
        let iso_path = work_dir.join(format!("{}-cidata.iso", vm_name));
        
        // Make sure the directory exists
        fs::create_dir_all(work_dir)
            .context("Failed to create working directory")?;
        
        // Write files
        fs::write(&user_data_path, user_data)
            .context("Failed to write user-data file")?;
        fs::write(&meta_data_path, meta_data)
            .context("Failed to write meta-data file")?;
        
        // Check for genisoimage or mkisofs
        let mut cmd;
        if Command::new("genisoimage").arg("--version").output().is_ok() {
            cmd = Command::new("genisoimage");
            cmd.args(&[
                "-output", iso_path.to_str().unwrap(),
                "-volid", "cidata",
                "-joliet", "-rock",
                user_data_path.to_str().unwrap(),
                meta_data_path.to_str().unwrap(),
            ]);
        } else if Command::new("mkisofs").arg("--version").output().is_ok() {
            cmd = Command::new("mkisofs");
            cmd.args(&[
                "-o", iso_path.to_str().unwrap(),
                "-V", "cidata",
                "-J", "-r",
                user_data_path.to_str().unwrap(),
                meta_data_path.to_str().unwrap(),
            ]);
        } else {
            return Err(anyhow::anyhow!("Neither genisoimage nor mkisofs found. Please install one of these tools."));
        }
        
        // Run the command
        let status = cmd.status()
            .context("Failed to execute ISO creation command")?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("Failed to create cloud-init ISO"));
        }
        
        // Remove temporary files
        fs::remove_file(user_data_path)
            .context("Failed to clean up user-data file")?;
        fs::remove_file(meta_data_path)
            .context("Failed to clean up meta-data file")?;
        
        Ok(iso_path)
    }
    
    /// Find an SSH public key
    pub fn find_ssh_public_key() -> Result<String> {
        // Try to find a suitable key file
        let possible_keys = [
            "id_rsa.pub",
            "id_ed25519.pub",
            "id_dsa.pub",
        ];
        
        if let Some(home) = dirs::home_dir() {
            let ssh_dir = home.join(".ssh");
            
            for key_name in possible_keys.iter() {
                let key_path = ssh_dir.join(key_name);
                if key_path.exists() {
                    return fs::read_to_string(&key_path)
                        .context(format!("Failed to read SSH key from {}", key_path.display()));
                }
            }
        }
        
        Err(anyhow::anyhow!("No SSH public key found. Please generate an SSH keypair using 'ssh-keygen' or specify one with the '-k' flag."))
    }
}