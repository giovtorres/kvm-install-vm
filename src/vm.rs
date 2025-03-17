use anyhow::Result;

pub struct VirtualMachine {
    pub name: String,
    pub vcpus: u32,
    pub memory_mb: u32,
    pub disk_size_gb: u32,
}

impl VirtualMachine {
    pub fn new(name: String, vcpus: u32, memory_mb: u32, disk_size_gb: u32) -> Self {
        VirtualMachine {
            name,
            vcpus,
            memory_mb,
            disk_size_gb,
        }
    }

    pub fn create(&self) -> Result<()> {
        println!("Creating VM: {}", self.name);
        Ok(())
    }
}
