#![allow(unused_imports)]

mod ffi;

#[cfg(feature = "package")]
mod package;

#[cfg(feature = "graphical")]
mod graphical;

use crate::extra;
use crate::extra::get_entries;
use crate::extra::path_extension;
use crate::shared;
use crate::traits::*;
use std::fs;
use std::fs::read_dir;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(any(feature = "general", feature = "memory"))]
use ffi::sysinfo;

#[cfg(any(feature = "general", feature = "kernel"))]
use sysctl::{Ctl, Sysctl};

#[cfg(feature = "package")]
impl From<sqlite::Error> for ReadoutError {
    fn from(e: sqlite::Error) -> Self {
        ReadoutError::Other(e.to_string())
    }
}

#[cfg(feature = "kernel")]
pub struct LinuxKernelReadout {
    os_release_ctl: Option<Ctl>,
    os_type_ctl: Option<Ctl>,
}

#[cfg(feature = "general")]
pub struct LinuxGeneralReadout {
    hostname_ctl: Option<Ctl>,
    sysinfo: sysinfo,
}

#[cfg(feature = "memory")]
pub struct LinuxMemoryReadout {
    sysinfo: sysinfo,
}

#[cfg(feature = "battery")]
pub struct LinuxBatteryReadout;

#[cfg(feature = "product")]
pub struct LinuxProductReadout;

#[cfg(feature = "graphical")]
pub struct LinuxGraphicalReadout;

#[cfg(feature = "processor")]
pub struct LinuxProcessorReadout;

#[cfg(feature = "package")]
pub struct LinuxPackageReadout;

#[cfg(feature = "network")]
pub struct LinuxNetworkReadout;

#[cfg(feature = "processor")]
impl ProcessorReadout for LinuxProcessorReadout {
    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
        Ok(shared::cpu_model_name())
    }

    fn cpu_usage(&self) -> Result<usize, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut sysinfo = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };

        if ret != -1 {
            let f_load = 1f64 / (1 << libc::SI_LOAD_SHIFT) as f64;
            let cpu_usage = info.loads[0] as f64 * f_load;
            let cpu_usage_u =
                (cpu_usage / self.cpu_cores().unwrap() as f64 * 100.0).round() as usize;
            return Ok(cpu_usage_u as usize);
        }

        Err(ReadoutError::Other(
            "Something went wrong during the initialization of the sysinfo struct.".to_string(),
        ))
    }

    fn cpu_physical_cores(&self) -> Result<usize, ReadoutError> {
        use std::io::{BufRead, BufReader};
        if let Ok(content) = File::open("/proc/cpuinfo") {
            let reader = BufReader::new(content);
            for line in reader.lines().flatten() {
                if line.to_lowercase().starts_with("cpu cores") {
                    return Ok(line
                        .split(':')
                        .nth(1)
                        .unwrap()
                        .trim()
                        .parse::<usize>()
                        .unwrap());
                }
            }
        }

        Err(ReadoutError::MetricNotAvailable)
    }

    fn cpu_cores(&self) -> Result<usize, ReadoutError> {
        Ok(unsafe { libc::sysconf(libc::_SC_NPROCESSORS_CONF) } as usize)
    }
}

#[cfg(feature = "battery")]
impl BatteryReadout for LinuxBatteryReadout {
    fn new() -> Self {
        LinuxBatteryReadout
    }

    fn percentage(&self) -> Result<u8, ReadoutError> {
        if let Some(entries) = get_entries(Path::new("/sys/class/power_supply")) {
            let dirs: Vec<PathBuf> = entries
                .into_iter()
                .filter(|x| {
                    !x.components()
                        .last()
                        .unwrap()
                        .as_os_str()
                        .to_string_lossy()
                        .starts_with("ADP")
                })
                .collect();

            if let Some(battery) = dirs.first() {
                let path_to_capacity = battery.join("capacity");
                let percentage_text = extra::pop_newline(fs::read_to_string(path_to_capacity)?);
                let percentage_parsed = percentage_text.parse::<u8>();

                match percentage_parsed {
                    Ok(p) => return Ok(p),
                    Err(e) => {
                        return Err(ReadoutError::Other(format!(
                            "Could not parse the value '{}' into a digit: {:?}",
                            percentage_text, e
                        )))
                    }
                };
            }
        };

        Err(ReadoutError::Other("No batteries detected.".to_string()))
    }

    fn status(&self) -> Result<BatteryState, ReadoutError> {
        if let Some(entries) = get_entries(Path::new("/sys/class/power_supply")) {
            let dirs: Vec<PathBuf> = entries
                .into_iter()
                .filter(|x| {
                    !x.components()
                        .last()
                        .unwrap()
                        .as_os_str()
                        .to_string_lossy()
                        .starts_with("ADP")
                })
                .collect();

            if let Some(battery) = dirs.first() {
                let path_to_status = battery.join("status");
                let status_text =
                    extra::pop_newline(fs::read_to_string(path_to_status)?).to_lowercase();

                match &status_text[..] {
                    "charging" => return Ok(BatteryState::Charging),
                    "discharging" | "full" => return Ok(BatteryState::Discharging),
                    s => {
                        return Err(ReadoutError::Other(format!(
                            "Got an unexpected value \"{}\" reading battery status",
                            s,
                        )))
                    }
                }
            }
        }

        Err(ReadoutError::Other("No batteries detected.".to_string()))
    }

    fn health(&self) -> Result<u64, ReadoutError> {
        if let Some(entries) = get_entries(Path::new("/sys/class/power_supply")) {
            let dirs: Vec<PathBuf> = entries
                .into_iter()
                .filter(|x| {
                    !x.components()
                        .last()
                        .unwrap()
                        .as_os_str()
                        .to_string_lossy()
                        .starts_with("ADP")
                })
                .collect();

            if let Some(battery) = dirs.first() {
                let energy_full =
                    extra::pop_newline(fs::read_to_string(battery.join("energy_full"))?)
                        .parse::<u64>();

                let energy_full_design =
                    extra::pop_newline(fs::read_to_string(battery.join("energy_full_design"))?)
                        .parse::<u64>();

                match (energy_full, energy_full_design) {
                    (Ok(mut ef), Ok(efd)) => {
                        if ef > efd {
                            ef = efd;
                            return Ok(((ef as f64 / efd as f64) * 100_f64) as u64);
                        }

                        return Ok(((ef as f64 / efd as f64) * 100_f64) as u64);
                    }
                    _ => {
                        return Err(ReadoutError::Other(
                            "Error calculating battery health.".to_string(),
                        ))
                    }
                }
            }
        }

        Err(ReadoutError::Other("No batteries detected.".to_string()))
    }
}

#[cfg(feature = "kernel")]
impl KernelReadout for LinuxKernelReadout {
    fn new() -> Self {
        LinuxKernelReadout {
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

#[cfg(feature = "network")]
impl NetworkReadout for LinuxNetworkReadout {
    fn new() -> Self {
        LinuxNetworkReadout
    }

    fn tx_bytes(&self, interface: Option<&str>) -> Result<usize, ReadoutError> {
        if let Some(ifname) = interface {
            let rx_file = PathBuf::from("/sys/class/net")
                .join(ifname)
                .join("statistics/tx_bytes");
            let content = std::fs::read_to_string(rx_file)?;
            let bytes = extra::pop_newline(content)
                .parse::<usize>()
                .unwrap_or_default();
            Ok(bytes)
        } else {
            Err(ReadoutError::Other(String::from(
                "Please specify a network interface to query.",
            )))
        }
    }

    fn tx_packets(&self, interface: Option<&str>) -> Result<usize, ReadoutError> {
        if let Some(ifname) = interface {
            let rx_file = PathBuf::from("/sys/class/net")
                .join(ifname)
                .join("statistics/tx_packets");
            let content = std::fs::read_to_string(rx_file)?;
            let packets = extra::pop_newline(content)
                .parse::<usize>()
                .unwrap_or_default();
            Ok(packets)
        } else {
            Err(ReadoutError::Other(String::from(
                "Please specify a network interface to query.",
            )))
        }
    }

    fn rx_bytes(&self, interface: Option<&str>) -> Result<usize, ReadoutError> {
        if let Some(ifname) = interface {
            let rx_file = PathBuf::from("/sys/class/net")
                .join(ifname)
                .join("statistics/rx_bytes");
            let content = std::fs::read_to_string(rx_file)?;
            let bytes = extra::pop_newline(content)
                .parse::<usize>()
                .unwrap_or_default();
            Ok(bytes)
        } else {
            Err(ReadoutError::Other(String::from(
                "Please specify a network interface to query.",
            )))
        }
    }

    fn rx_packets(&self, interface: Option<&str>) -> Result<usize, ReadoutError> {
        if let Some(ifname) = interface {
            let rx_file = PathBuf::from("/sys/class/net")
                .join(ifname)
                .join("statistics/rx_packets");
            let content = std::fs::read_to_string(rx_file)?;
            let packets = extra::pop_newline(content)
                .parse::<usize>()
                .unwrap_or_default();
            Ok(packets)
        } else {
            Err(ReadoutError::Other(String::from(
                "Please specify a network interface to query.",
            )))
        }
    }

    fn physical_address(&self, interface: Option<&str>) -> Result<String, ReadoutError> {
        if let Some(ifname) = interface {
            let rx_file = PathBuf::from("/sys/class/net").join(ifname).join("address");
            let content = std::fs::read_to_string(rx_file)?;
            Ok(content)
        } else {
            Err(ReadoutError::Other(String::from(
                "Please specify a network interface to query.",
            )))
        }
    }

    fn logical_address(&self, interface: Option<&str>) -> Result<String, ReadoutError> {
        shared::logical_address(interface)
    }
}

#[cfg(feature = "general")]
impl GeneralReadout for LinuxGeneralReadout {
    fn new() -> Self {
        LinuxGeneralReadout {
            hostname_ctl: Ctl::new("kernel.hostname").ok(),
            sysinfo: sysinfo::new(),
        }
    }

    fn backlight(&self) -> Result<usize, ReadoutError> {
        if let Some(base) = get_entries(Path::new("/sys/class/backlight/")) {
            if let Some(backlight_path) = base.into_iter().next() {
                let max_brightness_path = backlight_path.join("max_brightness");
                let current_brightness_path = backlight_path.join("brightness");

                let max_brightness_value =
                    extra::pop_newline(fs::read_to_string(max_brightness_path)?)
                        .parse::<usize>()
                        .ok();

                let current_brightness_value =
                    extra::pop_newline(fs::read_to_string(current_brightness_path)?)
                        .parse::<usize>()
                        .ok();

                match (current_brightness_value, max_brightness_value) {
                    (Some(c), Some(m)) => {
                        let brightness = c as f64 / m as f64 * 100f64;
                        return Ok(brightness.round() as usize);
                    }
                    _ => {
                        return Err(ReadoutError::Other(String::from(
                            "Error occurred while calculating backlight (brightness) value.",
                        )));
                    }
                }
            }
        }

        Err(ReadoutError::Other(String::from(
            "Could not obtain backlight information.",
        )))
    }

    fn resolution(&self) -> Result<String, ReadoutError> {
        let drm = Path::new("/sys/class/drm");

        if let Some(entries) = get_entries(drm) {
            let mut resolutions: Vec<String> = Vec::new();
            entries.into_iter().for_each(|entry| {
                // Append "modes" to /sys/class/drm/<card>/
                let modes = entry.join("modes");
                if let Ok(file) = File::open(modes) {
                    // Push the resolution to the resolutions vector.
                    if let Some(Ok(res)) = BufReader::new(file).lines().next() {
                        resolutions.push(res);
                    }
                }
            });

            return Ok(resolutions.join(", "));
        }

        Err(ReadoutError::Other(
            "Could not obtain screen resolution from /sys/class/drm".to_string(),
        ))
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

        if !content.version.is_empty() {
            return Ok(format!("{} {}", content.name, content.version));
        } else if !content.version_id.is_empty() {
            return Ok(format!("{} {}", content.name, content.version_id));
        }

        Ok(content.name)
    }

    fn shell(&self, format: ShellFormat, kind: ShellKind) -> Result<String, ReadoutError> {
        shared::shell(format, kind)
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut sysinfo = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };

        if ret != -1 {
            return Ok(info.uptime as usize);
        }

        Err(ReadoutError::Other(
            "Something went wrong during the initialization of the sysinfo struct.".to_string(),
        ))
    }

    fn operating_system(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn disk_space(&self) -> Result<(u128, u128), ReadoutError> {
        shared::disk_space(String::from("/"))
    }
}

#[cfg(feature = "memory")]
impl MemoryReadout for LinuxMemoryReadout {
    fn new() -> Self {
        LinuxMemoryReadout {
            sysinfo: sysinfo::new(),
        }
    }

    fn total(&self) -> Result<u64, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut sysinfo = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            Ok(info.totalram as u64 * info.mem_unit as u64 / 1024)
        } else {
            Err(ReadoutError::Other(
                "Something went wrong during the initialization of the sysinfo struct.".to_string(),
            ))
        }
    }

    fn free(&self) -> Result<u64, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut sysinfo = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            Ok(info.freeram as u64 * info.mem_unit as u64 / 1024)
        } else {
            Err(ReadoutError::Other(
                "Something went wrong during the initialization of the sysinfo struct.".to_string(),
            ))
        }
    }

    fn buffers(&self) -> Result<u64, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut sysinfo = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            Ok(info.bufferram as u64 * info.mem_unit as u64 / 1024)
        } else {
            Err(ReadoutError::Other(
                "Something went wrong during the initialization of the sysinfo struct.".to_string(),
            ))
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
        Ok(total - free - cached - reclaimable - buffers)
    }
}

#[cfg(feature = "product")]
impl ProductReadout for LinuxProductReadout {
    fn new() -> Self {
        LinuxProductReadout
    }

    fn vendor(&self) -> Result<String, ReadoutError> {
        Ok(extra::pop_newline(fs::read_to_string(
            "/sys/class/dmi/id/sys_vendor",
        )?))
    }

    fn family(&self) -> Result<String, ReadoutError> {
        Ok(extra::pop_newline(fs::read_to_string(
            "/sys/class/dmi/id/product_family",
        )?))
    }

    fn product(&self) -> Result<String, ReadoutError> {
        Ok(extra::pop_newline(fs::read_to_string(
            "/sys/class/dmi/id/product_name",
        )?))
    }

    fn machine(&self) -> Result<String, ReadoutError> {
        use itertools::Itertools;

        let vendor = self.vendor()?;
        let family = self.family()?;
        let product = self.product()?;
        let version = extra::pop_newline(fs::read_to_string("/sys/class/dmi/id/product_version")?);

        // If one field is generic, the others are likely the same, so fail the readout.
        if vendor.eq_ignore_ascii_case("system manufacturer") {
            return Err(ReadoutError::Other(String::from(
                "Your manufacturer may have not specified your machine's product information.",
            )));
        }

        let new_product = format!("{} {} {} {}", vendor, family, product, version)
            .replace("To be filled by O.E.M.", "");

        if family == product && family == version {
            return Ok(family);
        } else if version.is_empty() || version.len() <= 22 {
            return Ok(new_product
                .split_whitespace()
                .into_iter()
                .unique()
                .join(" "));
        }

        Ok(version)
    }
}
