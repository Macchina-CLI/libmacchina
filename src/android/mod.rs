mod sysinfo_ffi;

use crate::extra;
use crate::traits::*;
use itertools::Itertools;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use sysctl::{Ctl, Sysctl};
use sysinfo_ffi::sysinfo;
use sysinfo_ffi::system_info;

impl From<std::str::Utf8Error> for ReadoutError {
    fn from(e: std::str::Utf8Error) -> Self {
        ReadoutError::Other(e.to_string())
    }
}
impl From<std::num::ParseFloatError> for ReadoutError {
    fn from(e: std::num::ParseFloatError) -> Self {
        ReadoutError::Other(e.to_string())
    }
}

pub struct AndroidBatteryReadout;

pub struct AndroidKernelReadout {
    os_release_ctl: Option<Ctl>,
    os_type_ctl: Option<Ctl>,
}

pub struct AndroidGeneralReadout {
    hostname_ctl: Option<Ctl>,
    sysinfo: system_info,
}

pub struct AndroidMemoryReadout {
    sysinfo: system_info,
}

pub struct AndroidProductReadout;

pub struct AndroidPackageReadout;

impl BatteryReadout for AndroidBatteryReadout {
    fn new() -> Self {
        AndroidBatteryReadout
    }

    fn percentage(&self) -> Result<u8, ReadoutError> {
        let bat_path = Path::new("/sys/class/power_supply/battery/capacity");
        let percentage_text = extra::pop_newline(fs::read_to_string(bat_path)?);
        let percentage_parsed = percentage_text.parse::<u8>();

        match percentage_parsed {
            Ok(p) => Ok(p),
            Err(e) => Err(ReadoutError::Other(format!(
                "Could not parse the value '{}' of {} into a \
            digit: {:?}",
                percentage_text,
                bat_path.to_str().unwrap_or_default(),
                e
            ))),
        }
    }

    fn status(&self) -> Result<BatteryState, ReadoutError> {
        let bat_path = Path::new("/sys/class/power_supply/battery/status");

        let status_text = extra::pop_newline(fs::read_to_string(bat_path)?).to_lowercase();
        match &status_text[..] {
            "charging" => Ok(BatteryState::Charging),
            "discharging" | "full" => Ok(BatteryState::Discharging),
            s => Err(ReadoutError::Other(format!(
                "Got unexpected value '{}' from {}.",
                s,
                bat_path.to_str().unwrap_or_default()
            ))),
        }
    }
}

impl KernelReadout for AndroidKernelReadout {
    fn new() -> Self {
        AndroidKernelReadout {
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

impl GeneralReadout for AndroidGeneralReadout {
    fn new() -> Self {
        AndroidGeneralReadout {
            hostname_ctl: Ctl::new("kernel.hostname").ok(),
            sysinfo: system_info::new(),
        }
    }

    fn machine(&self) -> Result<String, ReadoutError> {
        let product_readout = AndroidProductReadout::new();

        let name = product_readout.name()?;
        let version = product_readout.version()?;
        let vendor = product_readout.vendor()?;

        let product = format!("{} {} ({})", vendor, name, version);
        let new_product: Vec<_> = product.split_whitespace().into_iter().unique().collect();

        if version.is_empty() || version.len() <= 15 {
            return Ok(new_product.into_iter().join(" "));
        }

        Ok(version)
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

    fn shell(&self, format: ShellFormat) -> Result<String, ReadoutError> {
        if let Some(shell) = std::env::var_os("SHELL") {
            Ok(shell.to_string_lossy().to_string())
        } else {
            crate::shared::shell(format)
        }
    }

    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
        Ok(crate::shared::cpu_model_name())
    }

    fn cpu_physical_cores(&self) -> Result<usize, ReadoutError> {
        crate::shared::cpu_physical_cores()
    }

    fn cpu_cores(&self) -> Result<usize, ReadoutError> {
        crate::shared::cpu_cores()
    }

    fn cpu_usage(&self) -> Result<usize, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut system_info = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            let f_load = 1f64 / (1 << libc::SI_LOAD_SHIFT) as f64;
            let cpu_usage = info.loads[0] as f64 * f_load;
            let cpu_usage_u = (cpu_usage / num_cpus::get() as f64 * 100.0).round() as usize;
            return Ok(cpu_usage_u as usize);
        } else {
            return Err(ReadoutError::Other(format!(
                "Failed to get system statistics"
            )));
        }
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut system_info = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            return Ok(info.uptime as usize);
        } else {
            return Err(ReadoutError::Other(format!(
                "Failed to get system statistics"
            )));
        }
    }
}

impl MemoryReadout for AndroidMemoryReadout {
    fn new() -> Self {
        AndroidMemoryReadout {
            sysinfo: system_info::new(),
        }
    }

    fn total(&self) -> Result<u64, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut system_info = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            return Ok(info.totalram);
        } else {
            return Err(ReadoutError::Other(format!(
                "Failed to get system statistics"
            )));
        }
    }

    fn free(&self) -> Result<u64, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut system_info = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            return Ok(info.freeram);
        } else {
            return Err(ReadoutError::Other(format!(
                "Failed to get system statistics"
            )));
        }
    }

    fn buffers(&self) -> Result<u64, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut system_info = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            return Ok(info.bufferram);
        } else {
            return Err(ReadoutError::Other(format!(
                "Failed to get system statistics"
            )));
        }
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

impl ProductReadout for AndroidProductReadout {
    fn new() -> Self {
        AndroidProductReadout
    }

    fn name(&self) -> Result<String, ReadoutError> {
        let getprop = Command::new("getprop")
            .arg("ro.product.model")
            .stdout(Stdio::piped())
            .output()
            .expect("ERROR: could not collect \"getprop ro.product.model\" process output");

        Ok(String::from_utf8(getprop.stdout)
            .expect("ERROR: \"getprop ro.product.model\" output was not valid UTF-8")
            .trim()
            .to_string())
        // ro.product.model
        // ro.product.odm.model
        // ro.product.product.model
        // ro.product.system.model
        // ro.product.system_ext.model
        // ro.product.vendor.model
        // Same in all cases ( needs more testing in other devices )
    }

    fn vendor(&self) -> Result<String, ReadoutError> {
        let getprop = Command::new("getprop")
            .arg("ro.product.brand")
            .stdout(Stdio::piped())
            .output()
            .expect("ERROR: could not collect \"getprop ro.product.brand\" process output");

        Ok(String::from_utf8(getprop.stdout)
            .expect("ERROR: \"getprop ro.product.brand\" output was not valid UTF-8")
            .trim()
            .to_string())
        // ro.product.brand
        // ro.product.manufacturer
        // ro.product.odm.brand
        // ro.product.odm.manufacturer
        // ro.product.product.brand
        // ro.product.product.manufacturer
        // ro.product.system.brand
        // ro.product.system.manufacturer
        // ro.product.system_ext.brand
        // ro.product.system_ext.manufacturer
        // ro.product.vendor.brand
        // ro.product.vendor.manufacturer
        // Same in all cases ( needs more testing in other devices )
    }

    fn version(&self) -> Result<String, ReadoutError> {
        let getprop = Command::new("getprop")
            .arg("ro.build.product")
            .stdout(Stdio::piped())
            .output()
            .expect("could not collect \"getprop ro.build.product\" process output");

        Ok(String::from_utf8(getprop.stdout)
            .expect("ERROR: \"getprop ro.build.product\" output was not valid UTF-8")
            .trim()
            .to_string())
        // ro.build.product
        // ro.product.device
        // ro.product.odm.device
        // ro.product.product.device
        // ro.product.system.device
        // ro.product.system_ext.device
        // ro.product.vendor.device
        // Same in all cases ( needs more testing in other devices )
    }
}

impl PackageReadout for AndroidPackageReadout {
    fn new() -> Self {
        AndroidPackageReadout
    }

    /// Supports: pm, dpkg, cargo
    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        let mut packages = Vec::new();
        // Since the target is Android we can assume that pm is available
        if let Some(c) = AndroidPackageReadout::count_pm() {
            packages.push((PackageManager::Android, c));
        }

        if extra::which("dpkg") {
            if let Some(c) = AndroidPackageReadout::count_dpkg() {
                packages.push((PackageManager::Dpkg, c));
            }
        }

        if extra::which("cargo") {
            if let Some(c) = AndroidPackageReadout::count_cargo() {
                packages.push((PackageManager::Cargo, c));
            }
        }

        packages
    }
}

impl AndroidPackageReadout {
    /// Returns the number of installed apps for the system
    /// Includes all apps ( user + system )
    fn count_pm() -> Option<usize> {
        let pm_output = Command::new("pm")
            .args(&["list", "packages"])
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        extra::count_lines(
            String::from_utf8(pm_output.stdout)
                .expect("ERROR: \"pm list packages\" output was not valid UTF-8"),
        )
    }
    /// Return the number of installed packages for systems
    /// that have `dpkg` installed.
    /// In android that's mainly termux.
    fn count_dpkg() -> Option<usize> {
        let dpkg_output = Command::new("dpkg")
            .arg("-l")
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        crate::extra::count_lines(
            String::from_utf8(dpkg_output.stdout)
                .expect("ERROR: \"dpkg -l\" output was not valid UTF-8"),
        )
    }

    /// Returns the number of installed packages for systems
    /// that have `cargo` installed.
    fn count_cargo() -> Option<usize> {
        crate::shared::count_cargo()
    }
}
