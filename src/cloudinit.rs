// cloudinit.rs - Simplified version without logger dependency
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, error, info, instrument, trace, warn};

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
        let meta_data = format!("instance-id: {}\nlocal-hostname: {}\n", vm_name, vm_name);

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
    #[instrument(skip(user_data, meta_data), fields(vm_name = %vm_name))]
    pub fn create_cloud_init_iso(
        work_dir: &Path,
        vm_name: &str,
        user_data: &str,
        meta_data: &str,
    ) -> Result<PathBuf> {
        info!("Creating cloud-init ISO for VM: {}", vm_name);
        println!("Creating cloud-init ISO for VM: {}", vm_name);

        debug!("Work directory: {}", work_dir.display());

        let user_data_path = work_dir.join("user-data");
        let meta_data_path = work_dir.join("meta-data");
        let iso_path = work_dir.join(format!("{}-cidata.iso", vm_name));

        debug!("User data path: {}", user_data_path.display());
        debug!("Meta data path: {}", meta_data_path.display());
        debug!("ISO path: {}", iso_path.display());

        // Make sure the directory exists
        if !work_dir.exists() {
            debug!("Creating working directory: {}", work_dir.display());
            fs::create_dir_all(work_dir).context("Failed to create working directory")?;
        }

        // Write files
        debug!("Writing user-data file");
        trace!("User data content: {}", user_data);
        fs::write(&user_data_path, user_data).context("Failed to write user-data file")?;

        debug!("Writing meta-data file");
        trace!("Meta data content: {}", meta_data);
        fs::write(&meta_data_path, meta_data).context("Failed to write meta-data file")?;

        // Check for genisoimage or mkisofs
        debug!("Checking for ISO creation tools");

        // Check if genisoimage is available
        let mut cmd;

        // Fixed approach - avoid directly chaining methods that create temporary values
        let has_genisoimage = {
            let result = Command::new("genisoimage").arg("--version").output();
            debug!("Checking for genisoimage: {:?}", result.is_ok());
            result.is_ok()
        };

        if has_genisoimage {
            info!("Using genisoimage to create ISO");
            println!("Using genisoimage to create ISO");
            cmd = Command::new("genisoimage");
            cmd.args(&[
                "-output",
                iso_path.to_str().unwrap(),
                "-volid",
                "cidata",
                "-joliet",
                "-rock",
                user_data_path.to_str().unwrap(),
                meta_data_path.to_str().unwrap(),
            ]);
            debug!("genisoimage command: {:?}", cmd);
        } else {
            // Check if mkisofs is available
            let has_mkisofs = {
                let result = Command::new("mkisofs").arg("--version").output();
                debug!("Checking for mkisofs: {:?}", result.is_ok());
                result.is_ok()
            };

            if has_mkisofs {
                info!("Using mkisofs to create ISO");
                println!("Using mkisofs to create ISO");
                cmd = Command::new("mkisofs");
                cmd.args(&[
                    "-o",
                    iso_path.to_str().unwrap(),
                    "-V",
                    "cidata",
                    "-J",
                    "-r",
                    user_data_path.to_str().unwrap(),
                    meta_data_path.to_str().unwrap(),
                ]);
                debug!("mkisofs command: {:?}", cmd);
            } else {
                error!("Neither genisoimage nor mkisofs found");
                return Err(anyhow::anyhow!(
                    "Neither genisoimage nor mkisofs found. Please install one of these tools."
                ));
            }
        }

        // Run the command
        debug!("Executing ISO creation command");
        let status = cmd
            .status()
            .context("Failed to execute ISO creation command")?;

        if !status.success() {
            error!("ISO creation command failed with status: {}", status);
            return Err(anyhow::anyhow!("Failed to create cloud-init ISO"));
        }

        info!("ISO created successfully: {}", iso_path.display());
        println!("ISO created successfully: {}", iso_path.display());

        // Remove temporary files
        debug!("Cleaning up temporary files");
        fs::remove_file(&user_data_path).context("Failed to clean up user-data file")?;
        fs::remove_file(&meta_data_path).context("Failed to clean up meta-data file")?;

        info!("Cloud-init ISO creation completed successfully");
        Ok(iso_path)
    }

    /// Find an SSH public key
    pub fn find_ssh_public_key() -> Result<String> {
        // Try to find a suitable key file
        let possible_keys = ["id_rsa.pub", "id_ed25519.pub", "id_dsa.pub"];

        if let Some(home) = dirs::home_dir() {
            let ssh_dir = home.join(".ssh");

            for key_name in possible_keys.iter() {
                let key_path = ssh_dir.join(key_name);
                if key_path.exists() {
                    return fs::read_to_string(&key_path).context(format!(
                        "Failed to read SSH key from {}",
                        key_path.display()
                    ));
                }
            }
        }

        Err(anyhow::anyhow!(
            "No SSH public key found. Please generate an SSH keypair using 'ssh-keygen' or specify one with the '-k' flag."
        ))
    }
}
