mod sysinfo_ffi;

use crate::extra;
use crate::shared;
use crate::traits::*;
use byte_unit::AdjustedByte;
use std::fs;
use std::io::{BufRead, BufReader};
use sysctl::{Ctl, Sysctl};
use sysinfo_ffi::sysinfo;

pub struct OpenWrtBatteryReadout;

pub struct OpenWrtKernelReadout {
    os_release_ctl: Option<Ctl>,
    os_type_ctl: Option<Ctl>,
}

pub struct OpenWrtGeneralReadout {
    hostname_ctl: Option<Ctl>,
    sysinfo: sysinfo,
}

pub struct OpenWrtMemoryReadout {
    sysinfo: sysinfo,
}

pub struct OpenWrtProductReadout;
pub struct OpenWrtPackageReadout;
pub struct OpenWrtNetworkReadout;

impl BatteryReadout for OpenWrtBatteryReadout {
    fn new() -> Self {
        OpenWrtBatteryReadout
    }
}

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

impl GeneralReadout for OpenWrtGeneralReadout {
    fn new() -> Self {
        OpenWrtGeneralReadout {
            hostname_ctl: Ctl::new("kernel.hostname").ok(),
            sysinfo: sysinfo::new(),
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

        Err(ReadoutError::Other(String::from(
            "Machine information not available in /proc/cpuinfo",
        )))
    }

    fn username(&self) -> Result<String, ReadoutError> {
        shared::username()
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

    fn shell(&self, format: ShellFormat) -> Result<String, ReadoutError> {
        shared::shell(format)
    }

    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
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

        Err(ReadoutError::Other(String::from(
            "Cannot read model from /proc/cpuinfo",
        )))
    }

    fn cpu_cores(&self) -> Result<usize, ReadoutError> {
        shared::cpu_cores()
    }

    fn cpu_physical_cores(&self) -> Result<usize, ReadoutError> {
        shared::cpu_physical_cores()
    }

    fn cpu_usage(&self) -> Result<usize, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut sysinfo = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            let f_load = 1f64 / (1 << libc::SI_LOAD_SHIFT) as f64;
            let cpu_usage = info.loads[0] as f64 * f_load;
            let cpu_usage_u = (cpu_usage / num_cpus::get() as f64 * 100.0).round() as usize;
            return Ok(cpu_usage_u as usize);
        } else {
            return Err(ReadoutError::Other(String::from(
                "sysinfo struct returned an error.",
            )));
        }
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut sysinfo = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            return Ok(info.uptime as usize);
        } else {
            return Err(ReadoutError::Other(String::from(
                "sysinfo struct returned an error.",
            )));
        }
    }

    fn disk_space(&self) -> Result<(AdjustedByte, AdjustedByte), ReadoutError> {
        shared::disk_space(String::from("/"))
    }
}

impl MemoryReadout for OpenWrtMemoryReadout {
    fn new() -> Self {
        OpenWrtMemoryReadout {
            sysinfo: sysinfo::new(),
        }
    }

    fn total(&self) -> Result<u64, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut sysinfo = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            return Ok(info.totalram as u64 * info.mem_unit as u64 / 1024);
        } else {
            return Err(ReadoutError::Other(String::from(
                "sysinfo struct returned an error.",
            )));
        }
    }

    fn free(&self) -> Result<u64, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut sysinfo = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            return Ok(info.freeram as u64 * info.mem_unit as u64 / 1024);
        } else {
            return Err(ReadoutError::Other(String::from(
                "sysinfo struct returned an error.",
            )));
        }
    }

    fn buffers(&self) -> Result<u64, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut sysinfo = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            return Ok(info.bufferram as u64 * info.mem_unit as u64 / 1024);
        } else {
            return Err(ReadoutError::Other(format!(
                "Failed to get system statistics"
            )));
        }
    }

    fn cached(&self) -> Result<u64, ReadoutError> {
        Ok(shared::get_meminfo_value("Cached"))
    }

    fn reclaimable(&self) -> Result<u64, ReadoutError> {
        Ok(shared::get_meminfo_value("SReclaimable"))
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

        if let Some(c) = OpenWrtPackageReadout::count_opkg() {
            packages.push((PackageManager::Opkg, c));
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
        let mut count: usize = 0;
        let file = fs::File::open("/usr/lib/opkg/status");
        if let Ok(content) = file {
            let reader = BufReader::new(content);
            for line in reader.lines().flatten() {
                if line.starts_with("Package:") {
                    count += 1
                }
            }

            return Some(count);
        }

        None
    }
}

impl NetworkReadout for OpenWrtNetworkReadout {
    fn new() -> Self {
        OpenWrtNetworkReadout
    }

    fn logical_address(&self, interface: Option<&str>) -> Result<String, ReadoutError> {
        shared::logical_address(interface)
    }
}
