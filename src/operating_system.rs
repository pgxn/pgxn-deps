use std::{
    fs,
    io::{BufRead, BufReader},
    result::Result as StdResult,
    str::FromStr,
};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy)]
pub enum OperatingSystem {
    Mac,
    Debian,
    RedHat,
    Windows,
}

impl FromStr for OperatingSystem {
    type Err = String;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mac" | "osx" | "macos" => Ok(OperatingSystem::Mac),
            "debian" => Ok(OperatingSystem::Debian),
            "redhat" | "rhel" => Ok(OperatingSystem::RedHat),
            "windows" | "win" => Ok(OperatingSystem::Windows),
            _ => Err(format!("Invalid operating system: '{}'", s)),
        }
    }
}

impl OperatingSystem {
    /// Package manager(s) for this operating system family
    pub fn package_managers(&self) -> &[PackageManager] {
        match self {
            OperatingSystem::Mac => &[PackageManager::Homebrew],
            OperatingSystem::Debian => &[PackageManager::Apt],
            OperatingSystem::RedHat => &[PackageManager::Dnf, PackageManager::Yum],
            OperatingSystem::Windows => &[PackageManager::Chocolatey],
        }
    }

    /// Detect the current operating system, if it's supported
    pub fn detect() -> Result<OperatingSystem> {
        let os = if cfg!(target_os = "linux") {
            Self::detect_linux_distribution()
        } else if cfg!(windows) {
            Some(OperatingSystem::Windows)
        } else if cfg!(target_os = "macos") {
            Some(OperatingSystem::Mac)
        } else {
            None
        };

        os.ok_or(Error::UnsupportedOperatingSystem)
    }

    /// Check `os-release` to detect current Linux distro
    fn detect_linux_distribution() -> Option<OperatingSystem> {
        let os_release = fs::File::open("/etc/os-release").ok()?;
        let reader = BufReader::new(os_release);

        for maybe_line in reader.lines() {
            let Ok(line) = maybe_line else {
                continue;
            };

            match &*line {
                "ID=debian" => return Some(OperatingSystem::Debian),
                "ID=fedora" | "ID=centos" | "ID=rhel" => return Some(OperatingSystem::RedHat),
                _ => continue,
            }
        }

        None
    }
}

pub enum PackageManager {
    Apt,
    Dnf,
    Yum,
    Chocolatey,
    Homebrew,
}

impl PackageManager {
    pub fn install(&self, package_name: &str) -> String {
        let install_command = match self {
            PackageManager::Apt => "apt-get install -y",
            PackageManager::Dnf => "dnf install -y",
            PackageManager::Yum => "yum install -y",
            PackageManager::Homebrew => "brew install",
            PackageManager::Chocolatey => "choco install",
        };

        format!(
            "{sudo}{install_command} {package_name}",
            sudo = if self.requires_sudo() { "sudo " } else { "" }
        )
    }

    pub fn requires_sudo(&self) -> bool {
        match self {
            PackageManager::Apt | PackageManager::Dnf | PackageManager::Yum => true,
            PackageManager::Homebrew | PackageManager::Chocolatey => false,
        }
    }

    pub fn repology_repository_prefix(&self) -> &[&str] {
        match self {
            PackageManager::Apt => &["debian_", "ubuntu_"],
            PackageManager::Dnf | PackageManager::Yum => &["fedora_", "centos_"],
            PackageManager::Chocolatey => &["chocolatey"],
            PackageManager::Homebrew => &["homebrew"],
        }
    }
}
