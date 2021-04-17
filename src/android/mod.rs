use crate::extra;
use crate::traits::*;
use itertools::Itertools;
use std::fs;
use std::fs::read_dir;
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
        let family = product_readout.family()?;
        let version = product_readout.version()?;
        let vendor = product_readout.vendor()?;

        let product = format!("{} {} {} {}", vendor, family, name, version)
            .replace("To be filled by O.E.M.", "");

        let new_product: Vec<_> = product.split_whitespace().into_iter().unique().collect();

        if family == name && family == version {
            return Ok(family);
        } else if version.is_empty() || version.len() <= 15 {
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

    fn desktop_environment(&self) -> Result<String, ReadoutError> {
        crate::shared::desktop_environment()
    }

    fn window_manager(&self) -> Result<String, ReadoutError> {
        crate::shared::window_manager()
    }

    fn terminal(&self) -> Result<String, ReadoutError> {
        crate::shared::terminal()
    }

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

    fn version(&self) -> Result<String, ReadoutError> {
        Ok(extra::pop_newline(fs::read_to_string(
            "/sys/class/dmi/id/product_version",
        )?))
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
    fn name(&self) -> Result<String, ReadoutError> {
        Ok(extra::pop_newline(fs::read_to_string(
            "/sys/class/dmi/id/product_name",
        )?))
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
        if extra::which("pacman") {
            if let Some(c) = AndroidPackageReadout::count_pacman() {
                packages.push((PackageManager::Pacman, c));
            }
        } else if extra::which("dpkg") {
            if let Some(c) = AndroidPackageReadout::count_dpkg() {
                packages.push((PackageManager::Dpkg, c));
            }
        } else if extra::which("qlist") {
            if let Some(c) = AndroidPackageReadout::count_portage() {
                packages.push((PackageManager::Portage, c));
            }
        } else if extra::which("xbps-query") {
            if let Some(c) = AndroidPackageReadout::count_xbps() {
                packages.push((PackageManager::Xbps, c));
            }
        } else if extra::which("rpm") {
            if let Some(c) = AndroidPackageReadout::count_rpm() {
                packages.push((PackageManager::Rpm, c));
            }
        } else if extra::which("eopkg") {
            if let Some(c) = AndroidPackageReadout::count_eopkg() {
                packages.push((PackageManager::Eopkg, c));
            }
        } else if extra::which("apk") {
            if let Some(c) = AndroidPackageReadout::count_apk() {
                packages.push((PackageManager::Apk, c));
            }
        }

        if extra::which("cargo") {
            if let Some(c) = AndroidPackageReadout::count_cargo() {
                packages.push((PackageManager::Cargo, c));
            }
        }
        if extra::which("flatpak") {
            if let Some(c) = AndroidPackageReadout::count_flatpak() {
                packages.push((PackageManager::Flatpak, c));
            }
        }
        if extra::which("snap") {
            if let Some(c) = AndroidPackageReadout::count_snap() {
                packages.push((PackageManager::Snap, c));
            }
        }

        packages
    }
}

impl AndroidPackageReadout {
    /// Returns the number of installed packages for systems
    /// that utilize `rpm` as their package manager. \
    /// Including but not limited to:
    /// - Fedora
    /// - OpenSUSE
    fn count_rpm() -> Option<usize> {
        // Return the number of installed packages using sqlite (~1ms)
        // as directly calling rpm or dnf is too expensive (~500ms)
        let path = "/var/lib/rpm/rpmdb.sqlite";
        let connection = sqlite::open(path);
        if let Ok(con) = connection {
            let statement = con.prepare("SELECT COUNT(*) FROM Installtid");
            if let Ok(mut s) = statement {
                if s.next().is_ok() {
                    return match s.read::<Option<i64>>(0) {
                        Ok(Some(count)) => Some(count as usize),
                        Ok(_) => Some(0),
                        Err(_) => None,
                    };
                }
            }
        }

        None
    }

    /// Returns the number of installed packages for systems
    /// that utilize `pacman` as their package manager. \
    /// Including but not limited to:
    /// - Arch Android
    /// - Manjaro
    fn count_pacman() -> Option<usize> {
        let pacman_dir = Path::new("/var/lib/pacman/local");
        if pacman_dir.exists() {
            if let Ok(read_dir) = read_dir(pacman_dir) {
                return Some(read_dir.count() - 1);
            };
        }

        None
    }

    /// Returns the number of installed packages for systems
    /// that utilize `eopkg` as their package manager. \
    /// Including but not limited to:
    /// - Solus
    fn count_eopkg() -> Option<usize> {
        let eopkg_dir = Path::new("/var/lib/eopkg/package");
        if eopkg_dir.exists() {
            if let Ok(read_dir) = read_dir(eopkg_dir) {
                return Some(read_dir.count() - 1);
            };
        }

        None
    }

    /// Returns the number of installed packages for systems
    /// that utilize `dpkg` as their package manager. \
    /// Including but not limited to:
    /// - Debian
    /// - Ubuntu
    fn count_dpkg() -> Option<usize> {
        let dpkg_dir = Path::new("/var/lib/dpkg/info");
        let dir_entries = extra::list_dir_entries(dpkg_dir);
        Some(
            dir_entries
                .iter()
                .filter(|x| x.ends_with(".list"))
                .into_iter()
                .count(),
        )
    }

    /// Returns the number of installed packages for systems
    /// that utilize `portage` as their package manager. \
    /// Including but not limited to:
    /// - Gentoo
    /// - Funtoo Android
    fn count_portage() -> Option<usize> {
        let qlist_output = Command::new("qlist")
            .arg("-I")
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to spawn \"qlist\" process")
            .stdout
            .expect("ERROR: failed to open \"qlist\" stdout");

        let count = Command::new("wc")
            .arg("-l")
            .stdin(Stdio::from(qlist_output))
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to spawn \"wc\" process");

        let final_output = count
            .wait_with_output()
            .expect("ERROR: failed to wait for \"wc\" process to exit");

        String::from_utf8(final_output.stdout)
            .expect("ERROR: \"qlist -I | wc -l\" output was not valid UTF-8")
            .trim()
            .parse::<usize>()
            .ok()
    }

    /// Returns the number of installed packages for systems
    /// that utilize `xbps` as their package manager. \
    /// Including but not limited to:
    /// - Void Android
    fn count_xbps() -> Option<usize> {
        let xbps_output = Command::new("xbps-query")
            .arg("-l")
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to spawn \"xbps-query\" process")
            .stdout
            .expect("ERROR: failed to open \"xbps-query\" stdout");

        let count = Command::new("wc")
            .arg("-l")
            .stdin(Stdio::from(xbps_output))
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to spawn \"wc\" process");

        let final_output = count
            .wait_with_output()
            .expect("ERROR: failed to wait for \"wc\" process to exit");

        String::from_utf8(final_output.stdout)
            .expect("ERROR: \"xbps-query -l | wc -l\" output was not valid UTF-8")
            .trim()
            .parse::<usize>()
            .ok()
    }

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

    /// Returns the number of installed packages for systems
    /// that have `flatpak` installed.
    fn count_flatpak() -> Option<usize> {
        // Return the number of system-wide installed flatpaks
        let global_flatpak_dir = Path::new("/var/lib/flatpak/app");
        let mut global_packages = 0;
        if let Ok(dir) = read_dir(global_flatpak_dir) {
            global_packages = dir.count();
        }

        // Return the number of per-user installed flatpaks
        let mut user_packages: usize = 0;
        if let Some(home_dir) = home::home_dir() {
            let user_flatpak_dir = home_dir.join(".local/share/flatpak/app");
            if let Ok(dir) = read_dir(user_flatpak_dir) {
                user_packages = dir.count();
            }
        }

        Some(global_packages + user_packages)
    }

    /// Returns the number of installed packages for systems
    /// that have `snap` installed.
    fn count_snap() -> Option<usize> {
        let snap_dir = Path::new("/var/lib/snapd/snaps");
        if snap_dir.is_dir() {
            let dir_entries = extra::list_dir_entries(snap_dir);
            return Some(
                dir_entries
                    .iter()
                    .filter(|x| {
                        if x.is_file() && x.ends_with(".snap") {
                            return true;
                        }
                        false
                    })
                    .into_iter()
                    .count(),
            );
        }
        None
    }
}