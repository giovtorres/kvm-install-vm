use crate::vm::DistroInfo;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub distros: HashMap<String, DistroInfo>,

    #[serde(default)]
    pub defaults: DefaultConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultConfig {
    pub memory_mb: u32,
    pub vcpus: u32,
    pub disk_size_gb: u32,
    pub image_dir: String,
    pub vm_dir: String,
    pub dns_domain: String,
    pub timezone: String,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        DefaultConfig {
            memory_mb: 1024,
            vcpus: 1,
            disk_size_gb: 10,
            image_dir: home.join("virt/images").to_string_lossy().to_string(),
            vm_dir: home.join("virt/vms").to_string_lossy().to_string(),
            dns_domain: "example.local".to_string(),
            timezone: "UTC".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from a specified path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path).context(format!(
            "Failed to read config file: {}",
            path.as_ref().display()
        ))?;

        // Changed from serde_yaml to toml
        let config: Config =
            toml::from_str(&content).context("Failed to parse TOML configuration")?;

        Ok(config)
    }

    /// Load configuration from the default locations
    pub fn load() -> Result<Self> {
        // Check for config in user's config directory
        if let Some(config_dir) = dirs::config_dir() {
            // Changed file extension from yaml to toml
            let user_config = config_dir.join("kvm-install-vm/config.toml");
            if user_config.exists() {
                return Self::from_file(user_config);
            }
        }

        // Check for config in user's home directory
        if let Some(home_dir) = dirs::home_dir() {
            // Changed file extension from yaml to toml
            let home_config = home_dir.join(".config/kvm-install-vm/config.toml");
            if home_config.exists() {
                return Self::from_file(home_config);
            }

            // Legacy location
            let old_config = home_dir.join(".kivrc");
            if old_config.exists() {
                // TODO: Convert legacy config format if needed
                println!(
                    "Legacy .kivrc file found but not supported. Please convert to TOML format."
                );
            }
        }

        // Check for system-wide config
        // Changed file extension from yaml to toml
        let system_config = Path::new("/etc/kvm-install-vm/config.toml");
        if system_config.exists() {
            return Self::from_file(system_config);
        }

        // If no config files found, return default config
        Ok(Self::default())
    }

    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        // Changed from serde_yaml to toml
        let toml_content =
            toml::to_string_pretty(self).context("Failed to serialize configuration to TOML")?;

        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create directory: {}", parent.display()))?;
        }

        fs::write(&path, toml_content).context(format!(
            "Failed to write config to: {}",
            path.as_ref().display()
        ))?;

        Ok(())
    }

    /// Save configuration to the default user location
    pub fn save_to_user_config(&self) -> Result<()> {
        if let Some(config_dir) = dirs::config_dir() {
            // Changed file extension from yaml to toml
            let user_config = config_dir.join("kvm-install-vm/config.toml");
            self.save_to_file(user_config)
        } else {
            Err(anyhow::anyhow!("Could not determine user config directory"))
        }
    }

    /// Get distribution info by name
    pub fn get_distro(&self, name: &str) -> Result<&DistroInfo> {
        self.distros
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Distribution '{}' not found in configuration", name))
    }
}

impl Default for Config {
    fn default() -> Self {
        // Create a default configuration with some common distributions
        let mut distros = HashMap::new();

        // CentOS 8
        distros.insert(
            "centos8".to_string(),
            DistroInfo {
                qcow_filename: "CentOS-8-GenericCloud-8.1.1911-20200113.3.x86_64.qcow2".to_string(),
                os_variant: "centos8".to_string(),
                image_url: "https://cloud.centos.org/centos/8/x86_64/images".to_string(),
                login_user: "centos".to_string(),
                sudo_group: "wheel".to_string(),
                cloud_init_disable: "systemctl disable cloud-init.service".to_string(),
            },
        );

        // Ubuntu 20.04
        distros.insert(
            "ubuntu2004".to_string(),
            DistroInfo {
                qcow_filename: "ubuntu-20.04-server-cloudimg-amd64.img".to_string(),
                os_variant: "ubuntu20.04".to_string(),
                image_url: "https://cloud-images.ubuntu.com/releases/20.04/release".to_string(),
                login_user: "ubuntu".to_string(),
                sudo_group: "sudo".to_string(),
                cloud_init_disable: "systemctl disable cloud-init.service".to_string(),
            },
        );

        // Fedora 35
        distros.insert("fedora35".to_string(), DistroInfo {
            qcow_filename: "Fedora-Cloud-Base-35-1.2.x86_64.qcow2".to_string(),
            os_variant: "fedora35".to_string(),
            image_url: "https://download.fedoraproject.org/pub/fedora/linux/releases/35/Cloud/x86_64/images".to_string(),
            login_user: "fedora".to_string(),
            sudo_group: "wheel".to_string(),
            cloud_init_disable: "systemctl disable cloud-init.service".to_string(),
        });

        Config {
            distros,
            defaults: DefaultConfig::default(),
        }
    }
}
