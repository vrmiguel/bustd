use std::fmt::Display;

#[derive(Debug)]
pub struct LinuxVersion {
    pub major: u8,
    pub minor: u8,
}

impl Display for LinuxVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (major, minor) = (self.major, self.minor);
        write!(f, "Linux {major}.{minor}")
    }
}
