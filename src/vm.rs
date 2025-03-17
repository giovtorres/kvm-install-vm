use anyhow::Result;

pub struct VirtualMachine {
    pub name: String,
    pub vcpus: u32,
    pub memory: u32,
    pub disk_size: u32,
}

impl VirtualMachine {
    pub fn new(name: String, vcpus: u32, memory: u32, disk_size: u32) -> Self {
        Self {
            name,
            vcpus,
            memory,
            disk_size,
        }
    }

    pub fn create(&self) -> Result<()> {
        println!("Creating VM: {}", self.name);
        Ok(())
    }
}
