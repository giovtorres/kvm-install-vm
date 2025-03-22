use std::fmt;

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

pub fn extract_disk_paths_from_xml(xml: &str) -> Vec<String> {
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