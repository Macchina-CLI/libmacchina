use crate::extra;
use crate::traits::*;
use local_ipaddress;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use sysctl::{Ctl, Sysctl};

impl From<sqlite::Error> for ReadoutError {
    fn from(e: sqlite::Error) -> Self {
        ReadoutError::Other(e.to_string())
    }
}

pub struct LinuxBatteryReadout;

pub struct LinuxKernelReadout {
    os_release_ctl: Option<Ctl>,
    os_type_ctl: Option<Ctl>,
}

pub struct LinuxGeneralReadout {
    hostname_ctl: Option<Ctl>,
    local_ip: Option<String>,
}

pub struct LinuxMemoryReadout;

pub struct LinuxProductReadout;

pub struct LinuxPackageReadout;

impl BatteryReadout for LinuxBatteryReadout {
    fn new() -> Self {
        LinuxBatteryReadout
    }

    fn percentage(&self) -> Result<u8, ReadoutError> {
        let mut bat_path = Path::new("/sys/class/power_supply/BAT0/capacity");
        if !Path::exists(bat_path) {
            bat_path = Path::new("/sys/class/power_supply/BAT1/capacity");
        }

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
        let mut bat_path = Path::new("/sys/class/power_supply/BAT0/status");
        if !Path::exists(bat_path) {
            bat_path = Path::new("/sys/class/power_supply/BAT1/status");
        }

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

impl GeneralReadout for LinuxGeneralReadout {
    fn new() -> Self {
        LinuxGeneralReadout {
            hostname_ctl: Ctl::new("kernel.hostname").ok(),
            local_ip: local_ipaddress::get(),
        }
    }

    fn machine(&self) -> Result<String, ReadoutError> {
        let product_readout = LinuxProductReadout::new();

        let name = product_readout
            .name()?
            .replace("To be filled by O.E.M.", "")
            .trim()
            .to_string();

        let family = product_readout
            .family()
            .unwrap_or_default()
            .replace("To be filled by O.E.M.", "")
            .trim()
            .to_string();

        let version = product_readout
            .version()
            .unwrap_or_default()
            .replace("To be filled by O.E.M.", "")
            .trim()
            .to_string();

        if family == name && family == version {
            return Ok(family);
        } else if version.is_empty() || version.len() <= 15 {
            let vendor = product_readout.vendor().unwrap_or_default();

            if !vendor.is_empty() {
                return Ok(format!("{} {} {}", vendor, family, name));
            }
        }

        Ok(version)
    }

    fn local_ip(&self) -> Result<String, ReadoutError> {
        Ok(self
            .local_ip
            .as_ref()
            .ok_or(ReadoutError::MetricNotAvailable)?
            .to_string())
    }

    fn username(&self) -> Result<String, ReadoutError> {
        crate::shared::whoami()
    }

    fn hostname(&self) -> Result<String, ReadoutError> {
        Ok(self
            .hostname_ctl
            .as_ref()
            .ok_or(ReadoutError::MetricNotAvailable)?
            .value_string()?)
    }

    fn distribution(&self) -> Result<String, ReadoutError> {
        crate::shared::distribution()
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

    fn shell(&self, shorthand: bool) -> Result<String, ReadoutError> {
        crate::shared::shell(shorthand)
    }

    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
        Ok(crate::shared::cpu_model_name())
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        crate::shared::uptime()
    }
}

impl MemoryReadout for LinuxMemoryReadout {
    fn new() -> Self {
        LinuxMemoryReadout
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

impl ProductReadout for LinuxProductReadout {
    fn new() -> Self {
        LinuxProductReadout
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

impl PackageReadout for LinuxPackageReadout {
    fn new() -> Self {
        LinuxPackageReadout
    }

    /// Returns the __number of installed packages__ for the following package managers:
    /// - pacman
    /// - apk _(using apk info )_
    /// - emerge _(using qlist)_
    /// - apt _(using dpkg)_
    /// - xbps _(using xbps-query)_
    /// - rpm
    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        let mut packages = Vec::new();
        // Instead of having a condition for each distribution.
        // we will try and extract package count by checking
        // if a certain package manager is installed
        if extra::which("pacman") {
            match LinuxPackageReadout::count_pacman() {
                Some(c) => packages.push((PackageManager::Pacman, c)),
                _ => (),
            }
        } else if extra::which("dpkg") {
            match LinuxPackageReadout::count_apt() {
                Some(c) => packages.push((PackageManager::Apt, c)),
                _ => (),
            }
        } else if extra::which("qlist") {
            match LinuxPackageReadout::count_portage() {
                Some(c) => packages.push((PackageManager::Portage, c)),
                _ => (),
            }
        } else if extra::which("xbps-query") {
            match LinuxPackageReadout::count_xbps() {
                Some(c) => packages.push((PackageManager::Xbps, c)),
                _ => (),
            }
        } else if extra::which("apk") {
            match LinuxPackageReadout::count_apk() {
                Some(c) => packages.push((PackageManager::Apk, c)),
                _ => (),
            }
        } else if extra::which("rpm") {
            match LinuxPackageReadout::count_rpm() {
                Some(c) => packages.push((PackageManager::Pacman, c)),
                _ => (),
            }
        }

        packages
    }
}

impl LinuxPackageReadout {
    fn count_rpm() -> Option<usize> {
        let path = "/var/lib/rpm/rpmdb.sqlite";
        let connection = sqlite::open(path);
        match connection {
            Ok(con) => {
                let statement = con.prepare("SELECT COUNT(*) FROM Installtid");
                if let Ok(mut s) = statement {
                    if s.next().is_ok() {
                        return match s.read::<Option<i64>>(0) {
                            Ok(Some(count)) => Some(count as usize),
                            Ok(_) => Some(0),
                            Err(_) => None,
                        };
                    }
                    return None;
                }
                return None;
            }
            Err(_) => None,
        }
    }

    fn count_pacman() -> Option<usize> {
        use std::fs::read_dir;
        use std::path::Path;

        let pacman_folder = Path::new("/var/lib/pacman/local");
        if pacman_folder.exists() {
            match read_dir(pacman_folder) {
                Ok(read_dir) => return Some(read_dir.count() - 1),
                _ => (),
            };
        }

        // Returns the number of installed packages using
        // pacman -Qq | wc -l
        let pacman_output = Command::new("pacman")
            .args(&["-Q", "-q"])
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to start \"pacman\" process")
            .stdout
            .expect("ERROR: failed to open \"pacman\" stdout");

        let count = Command::new("wc")
            .arg("-l")
            .stdin(Stdio::from(pacman_output))
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to start \"wc\" process");

        let final_output = count
            .wait_with_output()
            .expect("ERROR: failed to wait for \"wc\" process to exit");

        String::from_utf8(final_output.stdout)
            .expect("ERROR: \"pacman -Qq | wc -l\" output was not valid UTF-8")
            .trim()
            .parse::<usize>()
            .ok()
    }

    fn count_apt() -> Option<usize> {
        // Returns the number of installed packages using
        // dpkg -l | wc -l
        let dpkg_output = Command::new("dpkg")
            .arg("-l")
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to spawn \"dpkg\" process")
            .stdout
            .expect("ERROR: failed to open \"dpkg\" stdout");

        let count = Command::new("wc")
            .arg("-l")
            .stdin(Stdio::from(dpkg_output))
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to spawn \"wc\" process");

        let final_output = count
            .wait_with_output()
            .expect("ERROR: failed to wait for \"wc\" process to exit");

        String::from_utf8(final_output.stdout)
            .expect("ERROR: \"dpkg -l | wc -l\" output was not valid UTF-8")
            .trim()
            .parse::<usize>()
            .ok()
    }

    fn count_portage() -> Option<usize> {
        // Returns the number of installed packages using:
        // qlist -I | wc -l
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

    fn count_xbps() -> Option<usize> {
        // Returns the number of installed packages using:
        // xbps-query | grep ii | wc -l
        let xbps_output = Command::new("xbps-query")
            .arg("-l")
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to spawn \"xbps-query\" process")
            .stdout
            .expect("ERROR: failed to open \"xbps-query\" stdout");

        let grep_output = Command::new("grep")
            .arg("ii")
            .stdin(Stdio::from(xbps_output))
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to spawn \"grep\" process")
            .stdout
            .expect("ERROR: failed to read \"grep\" stdout");

        let count = Command::new("wc")
            .arg("-l")
            .stdin(Stdio::from(grep_output))
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to spawn \"wc\" process");

        let final_output = count
            .wait_with_output()
            .expect("ERROR: failed to wait for \"wc\" process to exit");

        String::from_utf8(final_output.stdout)
            .expect("ERROR: \"xbps-query -l | grep ii | wc -l\" output was not valid UTF-8")
            .trim()
            .parse::<usize>()
            .ok()
    }

    fn count_apk() -> Option<usize> {
        // Returns the number of installed packages using:
        // apk info | wc -l
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_pkgs() {
        if extra::which("pacman") {
            assert_eq!(LinuxPackageReadout::count_pacman().is_some(), true);
        } else if extra::which("xbps-query") {
            assert_eq!(LinuxPackageReadout::count_xbps().is_some(), true);
        } else if extra::which("apk") {
            assert_eq!(LinuxPackageReadout::count_apk().is_some(), true);
        } else if extra::which("dpkg") {
            assert_eq!(LinuxPackageReadout::count_apt().is_some(), true);
        } else if extra::which("portage-utils") {
            assert_eq!(LinuxPackageReadout::count_apt().is_some(), true);
        } else if extra::which("qlist") {
            assert_eq!(LinuxPackageReadout::count_portage().is_some(), true);
        } else if extra::which("rpm") {
            assert_eq!(LinuxPackageReadout::count_rpm().is_some(), true);
        }
    }

    #[test]
    fn test_shell() {
        let general_readout = &LinuxGeneralReadout::new();
        assert_eq!(
            LinuxGeneralReadout::shell(general_readout, true).is_ok(),
            true
        );
    }
    #[test]
    fn test_terminal() {
        let general_readout = &LinuxGeneralReadout::new();
        assert_eq!(LinuxGeneralReadout::terminal(general_readout).is_ok(), true);
    }

    #[test]
    fn test_battery_percentage() {
        let battery_readout = &LinuxBatteryReadout::new();
        assert_eq!(
            LinuxBatteryReadout::percentage(battery_readout).is_ok(),
            true
        );
    }

    #[test]
    fn test_battery_status() {
        let battery_readout = &LinuxBatteryReadout::new();
        assert_eq!(LinuxBatteryReadout::status(battery_readout).is_ok(), true);
    }

    #[test]
    fn test_kernel_osrelease() {
        let kernel_readout = &LinuxKernelReadout::new();
        assert_eq!(LinuxKernelReadout::os_release(kernel_readout).is_ok(), true);
    }

    #[test]
    fn test_kernel_ostype() {
        let kernel_readout = &LinuxKernelReadout::new();
        assert_eq!(LinuxKernelReadout::os_type(kernel_readout).is_ok(), true);
    }

    #[test]
    fn test_username() {
        let general_readout = &LinuxGeneralReadout::new();
        assert_eq!(LinuxGeneralReadout::username(general_readout).is_ok(), true);
    }

    #[test]
    fn test_hostname() {
        let general_readout = &LinuxGeneralReadout::new();
        assert_eq!(LinuxGeneralReadout::hostname(general_readout).is_ok(), true);
    }

    #[test]
    fn test_distribution() {
        let general_readout = &LinuxGeneralReadout::new();
        assert_eq!(
            LinuxGeneralReadout::distribution(general_readout).is_ok(),
            true
        );
    }

    #[test]
    fn test_desktop_environment() {
        let general_readout = &LinuxGeneralReadout::new();
        assert_eq!(
            LinuxGeneralReadout::desktop_environment(general_readout).is_ok(),
            true
        );
    }

    #[test]
    fn test_window_manager() {
        let general_readout = &LinuxGeneralReadout::new();
        assert_eq!(
            LinuxGeneralReadout::window_manager(general_readout).is_ok(),
            true
        );
    }

    #[test]
    fn test_cpu_model_name() {
        let general_readout = &LinuxGeneralReadout::new();
        assert_eq!(
            LinuxGeneralReadout::cpu_model_name(general_readout).is_ok(),
            true
        );
    }

    #[test]
    fn test_uptime() {
        let general_readout = &LinuxGeneralReadout::new();
        assert_eq!(LinuxGeneralReadout::uptime(general_readout).is_ok(), true);
    }
    #[test]
    fn test_version() {
        let product_readout = &LinuxProductReadout::new();
        assert_eq!(LinuxProductReadout::version(product_readout).is_ok(), true);
    }
    #[test]
    fn test_vendor() {
        let product_readout = &LinuxProductReadout::new();
        assert_eq!(LinuxProductReadout::vendor(product_readout).is_ok(), true);
    }
    #[test]
    fn test_family() {
        let product_readout = &LinuxProductReadout::new();
        assert_eq!(LinuxProductReadout::family(product_readout).is_ok(), true);
    }
    #[test]
    fn test_name() {
        let product_readout = &LinuxProductReadout::new();
        assert_eq!(LinuxProductReadout::name(product_readout).is_ok(), true);
    }
    #[test]
    fn test_total() {
        let memory_readout = &LinuxMemoryReadout::new();
        assert_eq!(LinuxMemoryReadout::total(memory_readout).is_ok(), true);
    }
    #[test]
    fn test_free() {
        let memory_readout = &LinuxMemoryReadout::new();
        assert_eq!(LinuxMemoryReadout::free(memory_readout).is_ok(), true);
    }
    #[test]
    fn test_buffers() {
        let memory_readout = &LinuxMemoryReadout::new();
        assert_eq!(LinuxMemoryReadout::buffers(memory_readout).is_ok(), true);
    }
    #[test]
    fn test_cached() {
        let memory_readout = &LinuxMemoryReadout::new();
        assert_eq!(LinuxMemoryReadout::cached(memory_readout).is_ok(), true);
    }
    #[test]
    fn test_reclaimable() {
        let memory_readout = &LinuxMemoryReadout::new();
        assert_eq!(
            LinuxMemoryReadout::reclaimable(memory_readout).is_ok(),
            true
        );
    }
}
