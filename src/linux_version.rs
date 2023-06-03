use std::{cmp::Ordering, fmt::Display};

#[derive(Debug, PartialEq, Eq)]
pub struct LinuxVersion {
    pub major: u8,
    pub minor: u8,
}

impl LinuxVersion {
    /// Given a release string (e.g. as given by `uname -r`), attempt
    /// to extract the major and minor values of the Linux version
    pub fn from_str(release: &str) -> Option<Self> {
        // The position of the first dot in the 'release' string
        let dot_idx = release.find('.')?;

        let (major, minor): (&str, &str) = release.split_at(dot_idx);

        let major: u8 = major.parse().ok()?;

        // Eat the leading dot in front of minor
        let minor = &minor[1..];
        let dot_idx = minor.find('.')?;

        let minor: u8 = minor[0..dot_idx].parse().ok()?;

        Some(Self { major, minor })
    }
}

impl Display for LinuxVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (major, minor) = (self.major, self.minor);
        write!(f, "Linux {major}.{minor}")
    }
}

impl PartialOrd for LinuxVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.major.partial_cmp(&other.major) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }

        self.minor.partial_cmp(&other.minor)
    }
}

#[cfg(test)]
mod tests {
    use crate::linux_version::LinuxVersion;

    #[test]
    fn should_be_able_to_parse_linux_versions() {
        assert_eq!(
            LinuxVersion::from_str("5.16.18-1-MANJARO").unwrap(),
            LinuxVersion {
                major: 5,
                minor: 16
            }
        );

        assert_eq!(
            LinuxVersion::from_str("3.8.3-Fedora").unwrap(),
            LinuxVersion { major: 3, minor: 8 }
        );
    }

    #[test]
    fn should_be_able_to_compare_linux_versions() {
        assert!(LinuxVersion::from_str("3.8.3") >= LinuxVersion::from_str("3.6.9"));

        // We do not require PATCH accuracy
        assert!(
            LinuxVersion::from_str("3.8.3").unwrap() == LinuxVersion::from_str("3.8.9").unwrap()
        );

        assert!(
            LinuxVersion::from_str("5.8.3").unwrap() > LinuxVersion::from_str("3.13.9").unwrap()
        );
        assert!(
            LinuxVersion::from_str("5.8.3").unwrap() < LinuxVersion::from_str("5.13.9").unwrap()
        );
        assert!(
            LinuxVersion::from_str("5.8.3").unwrap() > LinuxVersion::from_str("4.20.0").unwrap()
        );
        assert!(
            LinuxVersion::from_str("4.21.0").unwrap() > LinuxVersion::from_str("4.20.0").unwrap()
        );
        assert!(
            LinuxVersion::from_str("4.15.0").unwrap() < LinuxVersion::from_str("4.20.0").unwrap()
        );
    }
}
