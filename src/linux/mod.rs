#![allow(clippy::unnecessary_cast)]
mod pci_devices;
mod sysinfo_ffi;

use self::pci_devices::get_pci_devices;
use crate::extra;
use crate::extra::get_entries;
use crate::extra::path_extension;
use crate::shared;
use crate::traits::*;
use itertools::Itertools;
use pciid_parser::Database;
use regex::Regex;
use std::ffi::OsStr;
use std::fs;
use std::fs::read_dir;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use sysctl::{Ctl, Sysctl};
use sysinfo_ffi::sysinfo;

impl From<sqlite::Error> for ReadoutError {
    fn from(e: sqlite::Error) -> Self {
        ReadoutError::Other(e.to_string())
    }
}

pub struct LinuxKernelReadout {
    os_release_ctl: Option<Ctl>,
    os_type_ctl: Option<Ctl>,
}

pub struct LinuxGeneralReadout {
    hostname_ctl: Option<Ctl>,
    sysinfo: sysinfo,
}

pub struct LinuxMemoryReadout {
    sysinfo: sysinfo,
}

pub struct LinuxBatteryReadout;
pub struct LinuxProductReadout;
pub struct LinuxPackageReadout;
pub struct LinuxNetworkReadout;

impl BatteryReadout for LinuxBatteryReadout {
    fn new() -> Self {
        LinuxBatteryReadout
    }

    fn percentage(&self) -> Result<u8, ReadoutError> {
        if let Some(entries) = get_entries(Path::new("/sys/class/power_supply")) {
            let dirs: Vec<PathBuf> = entries
                .into_iter()
                .filter(|x| {
                    x.components()
                        .last()
                        .unwrap()
                        .as_os_str()
                        .to_string_lossy()
                        .starts_with("BAT")
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
                            "Could not parse the value '{percentage_text}' into a digit: {e:?}"
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
                    x.components()
                        .last()
                        .unwrap()
                        .as_os_str()
                        .to_string_lossy()
                        .starts_with("BAT")
                })
                .collect();

            if let Some(battery) = dirs.first() {
                let path_to_status = battery.join("status");
                let status_text =
                    extra::pop_newline(fs::read_to_string(path_to_status)?).to_lowercase();

                match &status_text[..] {
                    "charging" => return Ok(BatteryState::Charging),
                    "discharging" | "full" | "not charging" => return Ok(BatteryState::Discharging),
                    s => {
                        return Err(ReadoutError::Other(format!(
                            "Got an unexpected value \"{s}\" reading battery status"
                        )))
                    }
                }
            }
        }

        Err(ReadoutError::Other("No batteries detected.".to_string()))
    }

    fn health(&self) -> Result<u8, ReadoutError> {
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
                        .parse::<u8>();

                let energy_full_design =
                    extra::pop_newline(fs::read_to_string(battery.join("energy_full_design"))?)
                        .parse::<u8>();

                match (energy_full, energy_full_design) {
                    (Ok(mut ef), Ok(efd)) => {
                        if ef > efd {
                            ef = efd;
                            return Ok((ef as f32 / efd as f32 * 100_f32).ceil() as u8);
                        }

                        return Ok((ef as f32 / efd as f32 * 100_f32).ceil() as u8);
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

    fn desktop_environment(&self) -> Result<String, ReadoutError> {
        shared::desktop_environment()
    }

    fn session(&self) -> Result<String, ReadoutError> {
        shared::session()
    }

    fn window_manager(&self) -> Result<String, ReadoutError> {
        shared::window_manager()
    }

    fn terminal(&self) -> Result<String, ReadoutError> {
        // This function returns the PPID of a given PID:
        //  - The file used to extract this data: /proc/<pid>/status
        //  - This function parses and returns the value of the ppid line.
        fn get_parent(pid: i32) -> i32 {
            let process_path = PathBuf::from("/proc").join(pid.to_string()).join("status");
            let file = File::open(process_path);
            match file {
                Ok(content) => {
                    let reader = BufReader::new(content);
                    for line in reader.lines().map_while(Result::ok) {
                        if line.to_uppercase().starts_with("PPID") {
                            let s_mem_kb: String =
                                line.chars().filter(|c| c.is_ascii_digit()).collect();
                            return s_mem_kb.parse::<i32>().unwrap_or(-1);
                        }
                    }

                    -1
                }

                Err(_) => -1,
            }
        }

        // This function returns the name associated with a given PPID
        fn terminal_name() -> String {
            let mut terminal_pid = get_parent(unsafe { libc::getppid() });

            let path = PathBuf::from("/proc")
                .join(terminal_pid.to_string())
                .join("comm");

            // The below loop will traverse /proc to find the
            // terminal inside of which the user is operating
            if let Ok(mut terminal_name) = fs::read_to_string(path) {
                // Any command_name we find that matches
                // one of the elements within this table
                // is effectively ignored
                while extra::common_shells().contains(&terminal_name.replace('\n', "").as_str()) {
                    let ppid = get_parent(terminal_pid);
                    terminal_pid = ppid;

                    let path = PathBuf::from("/proc").join(ppid.to_string()).join("comm");

                    if let Ok(comm) = fs::read_to_string(path) {
                        terminal_name = comm;
                    }
                }

                return terminal_name;
            }

            String::new()
        }

        let terminal = terminal_name();

        if terminal.is_empty() {
            return Err(ReadoutError::Other(
                "Querying terminal information failed".to_string(),
            ));
        }

        Ok(terminal)
    }

    fn shell(&self, format: ShellFormat, kind: ShellKind) -> Result<String, ReadoutError> {
        shared::shell(format, kind)
    }

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
            for line in reader.lines().map_while(Result::ok) {
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

    fn machine(&self) -> Result<String, ReadoutError> {
        let product_readout = LinuxProductReadout::new();

        let vendor = product_readout.vendor()?;
        let family = product_readout.family()?;
        let product = product_readout.product()?;
        let version = extra::pop_newline(fs::read_to_string("/sys/class/dmi/id/product_version")?);

        // If one field is generic, the others are likely the same, so fail the readout.
        if vendor.eq_ignore_ascii_case("system manufacturer") {
            return Err(ReadoutError::Other(String::from(
                "Your manufacturer may have not specified your machine's product information.",
            )));
        }

        let new_product =
            format!("{vendor} {family} {product} {version}").replace("To be filled by O.E.M.", "");

        if family == product && family == version {
            return Ok(family);
        } else if version.is_empty() || version.len() <= 22 {
            return Ok(new_product.split_whitespace().unique().join(" "));
        }

        Ok(version)
    }

    fn os_name(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn disk_space(&self, path: &Path) -> Result<(u64, u64), ReadoutError> {
        shared::disk_space(path)
    }

    fn gpus(&self) -> Result<Vec<String>, ReadoutError> {
        let db = match Database::read() {
            Ok(db) => db,
            _ => return Err(ReadoutError::MetricNotAvailable),
        };

        let devices = get_pci_devices()?;
        let mut gpus = vec![];

        for device in devices {
            if !device.is_gpu(&db) {
                continue;
            };

            if let Some(sub_device_name) = device.get_device_name(&db) {
                gpus.push(sub_device_name);
            };
        }

        if gpus.is_empty() {
            Err(ReadoutError::MetricNotAvailable)
        } else {
            Ok(gpus)
        }
    }
}

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
        match shared::get_meminfo_value("MemAvailable") {
            0 => {
                let free = self.free().unwrap();
                let cached = self.cached().unwrap();
                let reclaimable = self.reclaimable().unwrap();
                let buffers = self.buffers().unwrap();
                Ok(total - free - cached - reclaimable - buffers)
            }
            available => Ok(total - available),
        }
    }

    fn swap_total(&self) -> Result<u64, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut sysinfo = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            Ok(info.totalswap as u64 * info.mem_unit as u64 / 1024)
        } else {
            Err(ReadoutError::Other(
                "Something went wrong during the initialization of the sysinfo struct.".to_string(),
            ))
        }
    }

    fn swap_free(&self) -> Result<u64, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut sysinfo = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            Ok(info.freeswap as u64 * info.mem_unit as u64 / 1024)
        } else {
            Err(ReadoutError::Other(
                "Something went wrong during the initialization of the sysinfo struct.".to_string(),
            ))
        }
    }

    fn swap_used(&self) -> Result<u64, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut sysinfo = &mut info;
        let ret = unsafe { sysinfo(info_ptr) };
        if ret != -1 {
            Ok((info.totalswap as u64 - info.freeswap as u64) * info.mem_unit as u64 / 1024)
        } else {
            Err(ReadoutError::Other(
                "Something went wrong during the initialization of the sysinfo struct.".to_string(),
            ))
        }
    }
}

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
}

impl PackageReadout for LinuxPackageReadout {
    fn new() -> Self {
        LinuxPackageReadout
    }

    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        let mut packages = Vec::new();
        let mut home = PathBuf::new();

        // Acquire the value of HOME early on to avoid
        // doing it multiple times.
        if let Ok(path) = std::env::var("HOME") {
            home = PathBuf::from(path);
        }

        if let Some(c) = LinuxPackageReadout::count_pacman() {
            packages.push((PackageManager::Pacman, c));
        }

        if let Some(c) = LinuxPackageReadout::count_dpkg() {
            packages.push((PackageManager::Dpkg, c));
        }

        if let Some(c) = LinuxPackageReadout::count_rpm() {
            packages.push((PackageManager::Rpm, c));
        }

        if let Some(c) = LinuxPackageReadout::count_portage() {
            packages.push((PackageManager::Portage, c));
        }

        if let Some(c) = LinuxPackageReadout::count_cargo() {
            packages.push((PackageManager::Cargo, c));
        }

        if let Some(c) = LinuxPackageReadout::count_xbps() {
            packages.push((PackageManager::Xbps, c));
        }

        if let Some(c) = LinuxPackageReadout::count_eopkg() {
            packages.push((PackageManager::Eopkg, c));
        }

        if let Some(c) = LinuxPackageReadout::count_apk() {
            packages.push((PackageManager::Apk, c));
        }

        if let Some(c) = LinuxPackageReadout::count_flatpak(&home) {
            packages.push((PackageManager::Flatpak, c));
        }

        if let Some(c) = LinuxPackageReadout::count_snap() {
            packages.push((PackageManager::Snap, c));
        }

        if let Some(c) = LinuxPackageReadout::count_homebrew(&home) {
            packages.push((PackageManager::Homebrew, c));
        }

        if let Some(c) = LinuxPackageReadout::count_nix() {
            packages.push((PackageManager::Nix, c));
        }

        packages
    }
}

impl LinuxPackageReadout {
    /// Returns the number of installed packages for systems
    /// that utilize `rpm` as their package manager.
    fn count_rpm() -> Option<usize> {
        // Return the number of installed packages using sqlite (~1ms)
        // as directly calling rpm or dnf is too expensive (~500ms)
        let count_sqlite = 'sqlite: {
            let db = "/var/lib/rpm/rpmdb.sqlite";
            if !Path::new(db).is_file() {
                break 'sqlite None;
            }

            let connection = sqlite::open(db);
            if let Ok(con) = connection {
                let statement = con.prepare("SELECT COUNT(*) FROM Installtid");
                if let Ok(mut s) = statement {
                    if s.next().is_ok() {
                        break 'sqlite match s.read::<Option<i64>, _>(0) {
                            Ok(Some(count)) => Some(count as usize),
                            _ => None,
                        };
                    }
                }
            }

            None
        };

        // If counting with sqlite failed, try using librpm instead
        count_sqlite.or_else(|| unsafe { rpm_pkg_count::count() }.map(|count| count as usize))
    }

    /// Returns the number of installed packages for systems
    /// that utilize `pacman` as their package manager.
    fn count_pacman() -> Option<usize> {
        let pacman_dir = Path::new("/var/lib/pacman/local");
        if pacman_dir.is_dir() {
            if let Ok(read_dir) = read_dir(pacman_dir) {
                return Some(read_dir.count() - 1); // Ignore ALPM_DB_VERSION
            };
        }

        None
    }

    /// Returns the number of installed packages for systems
    /// that utilize `eopkg` as their package manager.
    fn count_eopkg() -> Option<usize> {
        let eopkg_dir = Path::new("/var/lib/eopkg/package");
        if eopkg_dir.is_dir() {
            if let Ok(read_dir) = read_dir(eopkg_dir) {
                return Some(read_dir.count());
            };
        }

        None
    }

    /// Returns the number of installed packages for systems
    /// that utilize `portage` as their package manager.
    fn count_portage() -> Option<usize> {
        let pkg_dir = Path::new("/var/db/pkg");
        if pkg_dir.exists() {
            return Some(
                walkdir::WalkDir::new(pkg_dir)
                    .min_depth(2)
                    .max_depth(2)
                    .into_iter()
                    .count(),
            );
        }

        None
    }

    /// Returns the number of installed packages for systems
    /// that utilize `dpkg` as their package manager.
    fn count_dpkg() -> Option<usize> {
        let dpkg_dir = Path::new("/var/lib/dpkg/info");

        get_entries(dpkg_dir).map(|entries| {
            entries
                .iter()
                .filter(|x| extra::path_extension(x).unwrap_or_default() == "list")
                .count()
        })
    }

    /// Returns the number of installed packages for systems
    /// that have `homebrew` installed.
    fn count_homebrew(home: &Path) -> Option<usize> {
        let keepme = OsStr::new(".keepme");
        let mut base = home.join(".linuxbrew");

        if !base.is_dir() {
            base = PathBuf::from("/home/linuxbrew/.linuxbrew");
        }

        if !base.is_dir() {
            return None;
        }

        match read_dir(base.join("Cellar")) {
            Ok(dir) => Some(
                dir.filter(|entry| match entry {
                    Err(_) => false,
                    Ok(file) => file.file_name() != keepme,
                })
                .count(),
            ),
            Err(_) => None,
        }
    }

    /// Returns the number of installed packages for systems
    /// that utilize `xbps` as their package manager.
    fn count_xbps() -> Option<usize> {
        if !extra::which("xbps-query") {
            return None;
        }

        let xbps_output = Command::new("xbps-query")
            .arg("-l")
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        extra::count_lines(
            String::from_utf8(xbps_output.stdout)
                .expect("ERROR: \"xbps-query -l\" output was not valid UTF-8"),
        )
    }

    /// Returns the number of installed packages for systems
    /// that utilize `apk` as their package manager.
    fn count_apk() -> Option<usize> {
        // faster method for alpine: count empty lines in /lib/apk/db/installed
        if let Ok(content) = fs::read_to_string(Path::new("/lib/apk/db/installed")) {
            return Some(content.lines().filter(|l| l.is_empty()).count());
        }

        // fallback to command invocation
        if !extra::which("apk") {
            return None;
        }

        let apk_output = Command::new("apk")
            .arg("info")
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        extra::count_lines(
            String::from_utf8(apk_output.stdout)
                .expect("ERROR: \"apk info\" output was not valid UTF-8"),
        )
    }

    /// Returns the number of installed packages for systems
    /// that have `cargo` installed.
    fn count_cargo() -> Option<usize> {
        shared::count_cargo()
    }

    /// Returns the number of installed packages for systems
    /// that have `flatpak` installed.
    fn count_flatpak(home: &Path) -> Option<usize> {
        let mut total: usize = 0;
        let filter = Regex::new(r".*\.(Locale|Debug)").unwrap();
        for install in [
            Path::new("/var/lib/flatpak"),
            &home.join(".local/share/flatpak"),
        ] {
            if install.exists() {
                for dir in ["app", "runtime"] {
                    let pkgdir = install.join(dir);
                    if pkgdir.exists() {
                        for package in walkdir::WalkDir::new(&pkgdir)
                            .min_depth(1)
                            .max_depth(1)
                            .into_iter()
                            .filter_entry(|e| !filter.is_match(&e.path().to_string_lossy()))
                        {
                            total += walkdir::WalkDir::new(package.ok()?.path())
                                .min_depth(2)
                                .max_depth(2)
                                .into_iter()
                                .count();
                        }
                    }
                }
            }
        }
        if total > 0 {
            return Some(total);
        }

        None
    }

    /// Returns the number of installed packages for systems
    /// that have `snap` installed.
    fn count_snap() -> Option<usize> {
        let snap_dir = Path::new("/var/lib/snapd/snaps");
        if let Some(entries) = get_entries(snap_dir) {
            return Some(
                entries
                    .iter()
                    .filter(|&x| path_extension(x).unwrap_or_default() == "snap")
                    .count(),
            );
        }

        None
    }

    /// Returns the number of installed packages for systems
    /// that utilize `nix` as their package manager.
    fn count_nix() -> Option<usize> {
        return 'sqlite: {
            let db = "/nix/var/nix/db/db.sqlite";
            if !Path::new(db).is_file() {
                break 'sqlite None;
            }

            let connection = sqlite::Connection::open_with_flags(
                // The nix store is immutable, so we need to inform sqlite about it
                "file:".to_owned() + db + "?immutable=1",
                sqlite::OpenFlags::new().with_read_only().with_uri(),
            );

            if let Ok(con) = connection {
                let statement =
                    con.prepare("SELECT COUNT(path) FROM ValidPaths WHERE sigs IS NOT NULL");

                if let Ok(mut s) = statement {
                    if s.next().is_ok() {
                        break 'sqlite match s.read::<Option<i64>, _>(0) {
                            Ok(Some(count)) => Some(count as usize),
                            _ => None,
                        };
                    }
                }
            }

            None
        };
    }
}
