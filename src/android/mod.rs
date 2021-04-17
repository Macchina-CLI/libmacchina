use crate::extra;
use crate::traits::*;
use itertools::Itertools;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use sysctl::{Ctl, Sysctl};

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
}

pub struct AndroidMemoryReadout;

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

    fn distribution(&self) -> Result<String, ReadoutError> {
        use os_release::OsRelease;
        let content = OsRelease::new()?;
        if !content.version_id.is_empty() {
            return Ok(format!("{} {}", content.name, content.version_id));
        }

        Ok(content.name)
    }

    // fn desktop_environment(&self) -> Result<String, ReadoutError> {
    //     crate::shared::desktop_environment()
    // }

    // fn window_manager(&self) -> Result<String, ReadoutError> {
    //     crate::shared::window_manager()
    // }

    // fn terminal(&self) -> Result<String, ReadoutError> {
    //     crate::shared::terminal()
    // }

    fn shell(&self, format: ShellFormat) -> Result<String, ReadoutError> {
        crate::shared::shell(format)
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
        use std::fs;
        use std::io::Read;
        let mut procloadavg = fs::File::open("/proc/loadavg")?;
        let mut load: [u8; 3] = [0; 3];
        procloadavg.read_exact(&mut load)?;
        let value: f64 = std::str::from_utf8(&load)?.parse::<f64>()?;
        if let Ok(phys_cores) = crate::shared::cpu_cores() {
            let cpu_usage = (value as f64 / phys_cores as f64 * 100.0).round() as usize;
            if cpu_usage <= 100 {
                return Ok(cpu_usage);
            }
            return Ok(100);
        }
        Err(ReadoutError::Other(
            "Unable to extract processor usage.".to_string(),
        ))
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        crate::shared::uptime()
    }
}

impl MemoryReadout for AndroidMemoryReadout {
    fn new() -> Self {
        AndroidMemoryReadout
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

impl ProductReadout for AndroidProductReadout {
    fn new() -> Self {
        AndroidProductReadout
    }

    fn name(&self) -> Result<String, ReadoutError> {
        Ok(android_properties::getprop("ro.product.model").to_string())
        // ro.product.model
        // ro.product.odm.model
        // ro.product.product.model
        // ro.product.system.model
        // ro.product.system_ext.model
        // ro.product.vendor.model
        // Same in all cases ( needs more testing in other devices )
    }

    fn vendor(&self) -> Result<String, ReadoutError> {
        Ok(android_properties::getprop("ro.product.brand").to_string())
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
        Ok(android_properties::getprop("ro.build.product").to_string())
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

    /// Supports: pacman, apt, apk, portage, xbps, rpm, cargo
    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        let mut packages = Vec::new();
        // Instead of having a condition for each distribution.
        // we will try and extract package count by checking
        // if a certain package manager is installed

        if extra::which("cargo") {
            if let Some(c) = AndroidPackageReadout::count_cargo() {
                packages.push((PackageManager::Cargo, c));
            }
        }

        packages
    }
}

impl AndroidPackageReadout {
    /// Returns the number of installed packages for systems
    /// that utilize `apk` as their package manager. \
    /// Including but not limited to:
    /// - Alpine Android
    fn count_apk() -> Option<usize> {
        let apk_output = Command::new("apk")
            .arg("info")
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to start \"apk\" process")
            .stdout
            .expect("ERROR: failed to open \"apk\" stdout");

        let count = Command::new("wc")
            .arg("-l")
            .stdin(Stdio::from(apk_output))
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to start \"wc\" process");

        let final_output = count
            .wait_with_output()
            .expect("ERROR: failed to wait for \"wc\" process to exit");

        String::from_utf8(final_output.stdout)
            .expect("ERROR: \"apk info | wc -l\" output was not valid UTF-8")
            .trim()
            .parse::<usize>()
            .ok()
    }

    /// Returns the number of installed packages for systems
    /// that have `cargo` installed.
    fn count_cargo() -> Option<usize> {
        crate::shared::count_cargo()
    }
}
