use crate::extra;
use crate::traits::*;
use std::fs;
// use std::fs::read_dir;
// use std::path::Path;
// use std::process::{Command, Stdio};
use sysctl::{Ctl, Sysctl};

pub struct OpenWrtBatteryReadout;

pub struct OpenWrtKernelReadout {
    os_release_ctl: Option<Ctl>,
    os_type_ctl: Option<Ctl>,
}

pub struct OpenWrtGeneralReadout {
    hostname_ctl: Option<Ctl>,
}

pub struct OpenWrtMemoryReadout;

pub struct OpenWrtProductReadout;

pub struct OpenWrtPackageReadout;

impl KernelReadout for OpenWrtKernelReadout {
    fn new() -> Self {
        OpenWrtKernelReadout {
            os_release_ctl: Ctl::new("kernel.osrelease").ok(),
            os_type_ctl: Ctl::new("kernel.ostype").ok(),
        }
    }

    fn os_release(&self) -> Result<String, ReadoutError> {
        Ok(self
            .os_release_ctl
            .as_ref()
            .ok_or(ReadoutError::MetricNotAvailable)?
            .value_string()?)
    }

    fn os_type(&self) -> Result<String, ReadoutError> {
        Ok(self
            .os_type_ctl
            .as_ref()
            .ok_or(ReadoutError::MetricNotAvailable)?
            .value_string()?)
    }
}

impl BatteryReadout for OpenWrtBatteryReadout {
    fn new() -> Self {
        OpenWrtBatteryReadout
    }

    fn percentage(&self) -> Result<u8, ReadoutError> {
        unimplemented!();
    }

    fn status(&self) -> Result<BatteryState, ReadoutError> {
        unimplemented!();
    }
}

impl GeneralReadout for OpenWrtGeneralReadout {
    fn new() -> Self {
        OpenWrtGeneralReadout {
            hostname_ctl: Ctl::new("kernel.hostname").ok(),
        }
    }

    fn machine(&self) -> Result<String, ReadoutError> {
        use std::io::{BufRead, BufReader};
        let file = fs::File::open("/proc/cpuinfo");
        if let Ok(content) = file {
            let reader = BufReader::new(content);
            for line in reader.lines().into_iter().flatten() {
                if line.starts_with("machine") {
                    return Ok(line
                        .replace("machine", "")
                        .replace(":", "")
                        .trim()
                        .to_string());
                }
            }
        }
        Err(ReadoutError::Other(
            "Machine not available in /proc/cpuinfo".to_string(),
        ))
    }

    fn local_ip(&self) -> Result<String, ReadoutError> {
        crate::shared::local_ip()
    }

    fn username(&self) -> Result<String, ReadoutError> {
        crate::shared::username()
    }

    fn hostname(&self) -> Result<String, ReadoutError> {
        Ok(self
            .hostname_ctl
            .as_ref()
            .ok_or(ReadoutError::MetricNotAvailable)?
            .value_string()?)
    }

    fn distribution(&self) -> Result<String, ReadoutError> {
        use os_release::OsRelease;
        let content = OsRelease::new()?;
        if !content.version_id.is_empty() {
            return Ok(format!("{} {}", content.name, content.version_id));
        }

        Ok(content.name)
    }

    // fn desktop_environment(&self) -> Result<String, ReadoutError> {
    //     unimplemented!();
    // }

    // fn window_manager(&self) -> Result<String, ReadoutError> {
    //     unimplemented!();
    // }

    // fn terminal(&self) -> Result<String, ReadoutError> {
    //     unimplemented!();
    // }

    fn shell(&self, format: ShellFormat) -> Result<String, ReadoutError> {
        crate::shared::shell(format)
    }

    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
        // If cpu_model_name is unavialable use cpu_model
        use std::io::{BufRead, BufReader};
        let file = fs::File::open("/proc/cpuinfo");
        if let Ok(content) = file {
            let reader = BufReader::new(content);
            for line in reader.lines().into_iter().flatten() {
                if line.starts_with("cpu model") {
                    return Ok(line
                        .replace("cpu model", "")
                        .replace(":", "")
                        .trim()
                        .to_string());
                }
            }
        }
        Err(ReadoutError::Other(
            "Cannot read model from /proc/cpuinfo".to_string(),
        ))
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        unimplemented!();
    }
}

impl MemoryReadout for OpenWrtMemoryReadout {
    fn new() -> Self {
        OpenWrtMemoryReadout
    }

    fn total(&self) -> Result<u64, ReadoutError> {
        Ok(crate::shared::get_meminfo_value("MemTotal"))
    }

    fn free(&self) -> Result<u64, ReadoutError> {
        Ok(crate::shared::get_meminfo_value("MemFree"))
    }

    fn buffers(&self) -> Result<u64, ReadoutError> {
        Ok(crate::shared::get_meminfo_value("Buffers"))
    }

    fn cached(&self) -> Result<u64, ReadoutError> {
        Ok(crate::shared::get_meminfo_value("Cached"))
    }

    fn reclaimable(&self) -> Result<u64, ReadoutError> {
        Ok(crate::shared::get_meminfo_value("SReclaimable"))
    }

    fn used(&self) -> Result<u64, ReadoutError> {
        let total = self.total().unwrap();
        let free = self.free().unwrap();
        let cached = self.cached().unwrap();
        let reclaimable = self.reclaimable().unwrap();
        let buffers = self.buffers().unwrap();

        if reclaimable != 0 {
            return Ok(total - free - cached - reclaimable - buffers);
        }

        Ok(total - free - cached - buffers)
    }
}

impl PackageReadout for OpenWrtPackageReadout {
    fn new() -> Self {
        OpenWrtPackageReadout
    }

    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        let mut packages = Vec::new();
        // Instead of having a condition for each distribution.
        // we will try and extract package count by checking
        // if a certain package manager is installed
        if extra::which("opkg") {
            match OpenWrtPackageReadout::count_opkg() {
                Some(c) => packages.push((PackageManager::Opkg, c)),
                _ => (),
            }
        }

        if extra::which("cargo") {
            match OpenWrtPackageReadout::count_cargo() {
                Some(c) => packages.push((PackageManager::Cargo, c)),
                _ => (),
            }
        }

        packages
    }
}

impl OpenWrtPackageReadout {
    /// Returns the number of installed packages for systems
    /// that utilize `opkg` as their package manager. \
    /// Including but not limited to:
    /// - [OpenWrt](https://openwrt.org)
    fn count_opkg() -> Option<usize> {
        use std::io::{BufRead, BufReader};
        let mut count: usize = 0;
        let file = fs::File::open("/usr/lib/opkg/status");
        if let Ok(content) = file {
            let reader = BufReader::new(content);
            for line in reader.lines() {
                if let Ok(l) = line {
                    if l.starts_with("Package:") {
                        count += 1
                    }
                }
            }
            return Some(count);
        }
        None
    }

    /// Returns the number of installed packages for systems
    /// that have `cargo` installed.
    fn count_cargo() -> Option<usize> {
        crate::shared::count_cargo()
    }
}
