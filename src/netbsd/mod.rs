#![allow(clippy::unnecessary_cast)]
use crate::dirs;
use crate::extra;
use crate::shared;
use crate::traits::*;
use itertools::Itertools;
use nix::unistd;
use regex::Regex;
use std::ffi::CString;
use std::fs;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct NetBSDBatteryReadout;
pub struct NetBSDKernelReadout;
pub struct NetBSDGeneralReadout;
pub struct NetBSDMemoryReadout;
pub struct NetBSDProductReadout;
pub struct NetBSDPackageReadout;
pub struct NetBSDNetworkReadout;

impl BatteryReadout for NetBSDBatteryReadout {
    fn new() -> Self {
        NetBSDBatteryReadout
    }

    fn percentage(&self) -> Result<u8, ReadoutError> {
        if extra::which("envstat") {
            let envstat = Command::new("envstat")
                .args(["-s", "acpibat0:charge"])
                .stdout(Stdio::piped())
                .output()
                .expect("ERROR: failed to spawn \"envstat\" process");

            let envstat_out = String::from_utf8(envstat.stdout)
                .expect("ERROR: \"envstat\" process stdout was not valid UTF-8");
            if envstat_out.is_empty() {
                return Err(ReadoutError::MetricNotAvailable);
            } else {
                let re = Regex::new(r"\(([^()]*)\)").unwrap();
                let caps = re.captures(&envstat_out);
                match caps {
                    Some(c) => {
                        let percentage = c
                            .get(1)
                            .map_or("", |m| m.as_str())
                            .to_string()
                            .replace([' ', '%'], "");
                        let percentage_f = percentage.parse::<f32>().unwrap();
                        let percentage_i = percentage_f.round() as u8;
                        return Ok(percentage_i);
                    }
                    None => return Err(ReadoutError::MetricNotAvailable),
                }
            }
        }

        Err(ReadoutError::MetricNotAvailable)
    }

    fn status(&self) -> Result<BatteryState, ReadoutError> {
        if extra::which("envstat") {
            let envstat = Command::new("envstat")
                .args(["-s", "acpibat0:charging"])
                .stdout(Stdio::piped())
                .output()
                .expect("ERROR: failed to spawn \"envstat\" process");

            let envstat_out = String::from_utf8(envstat.stdout)
                .expect("ERROR: \"envstat\" process stdout was not valid UTF-8");

            if envstat_out.is_empty() {
                return Err(ReadoutError::MetricNotAvailable);
            } else if envstat_out.contains("TRUE") {
                return Ok(BatteryState::Charging);
            } else {
                return Ok(BatteryState::Discharging);
            }
        }

        Err(ReadoutError::Other("envstat is not installed".to_owned()))
    }

    fn health(&self) -> Result<u8, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }
}

impl KernelReadout for NetBSDKernelReadout {
    fn new() -> Self {
        NetBSDKernelReadout
    }

    fn os_release(&self) -> Result<String, ReadoutError> {
        let output = Command::new("sysctl")
            .args(["-n", "-b", "kern.osrelease"])
            .output()
            .expect("ERROR: failed to fetch \"kernel.osrelease\" using \"sysctl\"");

        let osrelease = String::from_utf8(output.stdout)
            .expect("ERROR: \"sysctl\" process stdout was not valid UTF-8");

        Ok(osrelease)
    }

    fn os_type(&self) -> Result<String, ReadoutError> {
        let output = Command::new("sysctl")
            .args(["-n", "-b", "kern.ostype"])
            .output()
            .expect("ERROR: failed to fetch \"kernel.ostype\" using \"sysctl\"");

        let osrelease = String::from_utf8(output.stdout)
            .expect("ERROR: \"sysctl\" process stdout was not valid UTF-8");

        Ok(osrelease)
    }

    fn pretty_kernel(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::Warning(String::from(
            "This information is provided by the OperatingSystem readout on NetBSD.",
        )))
    }
}

impl GeneralReadout for NetBSDGeneralReadout {
    fn new() -> Self {
        NetBSDGeneralReadout
    }

    fn resolution(&self) -> Result<String, ReadoutError> {
        shared::resolution()
    }

    fn backlight(&self) -> Result<usize, ReadoutError> {
        let output = Command::new("sysctl")
            .args(["-n", "hw.acpi.acpiout0.brightness"])
            .output()
            .expect("ERROR: failed to fetch \"hw.acpi.acpiout0.brightness\" using \"sysctl\"");

        let backlight = String::from_utf8(output.stdout)
            .expect("ERROR: \"sysctl\" process stdout was not valid UTF-8");

        if backlight.is_empty() {
            return Err(ReadoutError::Other(String::from(
                "Could not obtain backlight value through sysctl, is ACPIVGA driver installed?",
            )));
        }

        if let Ok(val) = extra::pop_newline(backlight).parse::<usize>() {
            return Ok(val);
        }

        Err(ReadoutError::Other(String::from(
            "Could not parse the obtained backlight value.",
        )))
    }

    fn machine(&self) -> Result<String, ReadoutError> {
        let product_readout = NetBSDProductReadout::new();

        let family = product_readout.family()?;
        let vendor = product_readout.vendor()?;
        let product = product_readout.product()?;

        let new_product =
            format!("{vendor} {family} {product}").replace("To be filled by O.E.M.", "");

        if product == new_product && product == vendor {
            return Ok(vendor);
        }

        Ok(new_product.split_whitespace().unique().join(" "))
    }

    fn username(&self) -> Result<String, ReadoutError> {
        shared::username()
    }

    fn hostname(&self) -> Result<String, ReadoutError> {
        let hostname_cstr = unistd::gethostname();
        match hostname_cstr {
            Ok(hostname_cstr) => {
                let hostname = hostname_cstr.to_str().unwrap_or("Unknown");
                Ok(String::from(hostname))
            }
            Err(_e) => Err(ReadoutError::Other(String::from(
                "Failed to retrieve hostname from 'gethostname'.",
            ))),
        }
    }

    fn distribution(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::Warning(String::from(
            "This information is provided by the OperatingSystem readout on NetBSD.",
        )))
    }

    fn desktop_environment(&self) -> Result<String, ReadoutError> {
        shared::desktop_environment()
    }

    fn session(&self) -> Result<String, ReadoutError> {
        shared::session()
    }

    fn window_manager(&self) -> Result<String, ReadoutError> {
        crate::winman::detect_xorg_window_manager()
    }

    fn terminal(&self) -> Result<String, ReadoutError> {
        // This function returns the PPID of a given PID:
        //  - The file used to extract this data: /proc/<pid>/status
        //  - The format of the file is: command_name command_pid command_ppid ...
        fn get_parent(pid: i32) -> i32 {
            let process_path = PathBuf::from("/proc").join(pid.to_string()).join("status");
            if let Ok(content) = fs::read_to_string(process_path) {
                if let Some(val) = content.split_whitespace().nth(2) {
                    if let Ok(c) = val.parse::<i32>() {
                        return c;
                    }
                }

                return -1;
            }

            -1
        }

        // This function returns the name associated with a given PPID
        fn terminal_name() -> String {
            let mut terminal_pid = get_parent(unsafe { libc::getppid() });

            let path = PathBuf::from("/proc")
                .join(terminal_pid.to_string())
                .join("status");

            // The below loop will traverse /proc to find the
            // terminal inside of which the user is operating
            if let Ok(mut terminal_name) = fs::read_to_string(path) {
                terminal_name = terminal_name.split_whitespace().next().unwrap().to_owned();

                // Any command_name we find that matches
                // one of the elements within this table
                // is effectively ignored
                while extra::common_shells().contains(&terminal_name.as_str()) {
                    let ppid = get_parent(terminal_pid);
                    terminal_pid = ppid;

                    let path = PathBuf::from("/proc")
                        .join(terminal_pid.to_string())
                        .join("status");

                    if let Ok(status) = fs::read_to_string(path) {
                        if let Some(name) = status.split_whitespace().next() {
                            terminal_name = name.to_string();
                        }
                    }
                }

                return terminal_name;
            }

            String::new()
        }

        let terminal = terminal_name();

        if terminal.is_empty() {
            return Err(ReadoutError::Other(
                "Could not to fetch terminal.".to_owned(),
            ));
        }

        Ok(terminal)
    }

    fn shell(&self, shorthand: ShellFormat, kind: ShellKind) -> Result<String, ReadoutError> {
        shared::shell(shorthand, kind)
    }

    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
        Ok(shared::cpu_model_name())
    }

    fn cpu_cores(&self) -> Result<usize, ReadoutError> {
        shared::cpu_cores()
    }

    fn cpu_physical_cores(&self) -> Result<usize, ReadoutError> {
        shared::cpu_physical_cores()
    }

    fn cpu_usage(&self) -> Result<usize, ReadoutError> {
        shared::cpu_usage()
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        shared::uptime()
    }

    fn os_name(&self) -> Result<String, ReadoutError> {
        let kernel_readout = NetBSDKernelReadout::new();

        let os_type = kernel_readout.os_type()?;
        let os_release = kernel_readout.os_release()?;

        if !(os_type.is_empty() || os_release.is_empty()) {
            return Ok(format!("{os_type} {os_release}"));
        }

        Err(ReadoutError::MetricNotAvailable)
    }

    fn disk_space(&self, path: &Path) -> Result<(u64, u64), ReadoutError> {
        use std::os::unix::ffi::OsStrExt;

        if !path.is_dir() || !path.is_absolute() {
            return Err(ReadoutError::Other(format!(
                "The provided path is not valid: {:?}",
                path
            )));
        }

        let mut s: std::mem::MaybeUninit<libc::statvfs> = std::mem::MaybeUninit::uninit();
        let path = CString::new(path.as_os_str().as_bytes())
            .expect("Could not create C string for disk usage path.");

        if unsafe { libc::statvfs(path.as_ptr(), s.as_mut_ptr()) } == 0 {
            let stats: libc::statvfs = unsafe { s.assume_init() };

            let disk_size = stats.f_blocks * stats.f_bsize as u64;
            let free = stats.f_bavail * stats.f_bsize as u64;

            let used_byte = disk_size - free;
            let disk_size_byte = disk_size;

            return Ok((used_byte, disk_size_byte));
        }

        Err(ReadoutError::Other(String::from(
            "Error while trying to get statfs structure.",
        )))
    }

    fn gpus(&self) -> Result<Vec<String>, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }
}

impl MemoryReadout for NetBSDMemoryReadout {
    fn new() -> Self {
        NetBSDMemoryReadout
    }

    fn total(&self) -> Result<u64, ReadoutError> {
        Ok(shared::get_meminfo_value("MemTotal"))
    }

    fn free(&self) -> Result<u64, ReadoutError> {
        Ok(shared::get_meminfo_value("MemFree"))
    }

    fn buffers(&self) -> Result<u64, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn cached(&self) -> Result<u64, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn reclaimable(&self) -> Result<u64, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn used(&self) -> Result<u64, ReadoutError> {
        let total = self.total().unwrap();
        let free = self.free().unwrap();

        Ok(total - free)
    }

    fn swap_total(&self) -> Result<u64, ReadoutError> {
        Ok(shared::get_meminfo_value("SwapTotal"))
    }

    fn swap_free(&self) -> Result<u64, ReadoutError> {
        Ok(shared::get_meminfo_value("SwapFree"))
    }

    fn swap_used(&self) -> Result<u64, ReadoutError> {
        let total = self.swap_total()?;
        let free = self.swap_free()?;

        Ok(total - free)
    }
}

impl ProductReadout for NetBSDProductReadout {
    fn new() -> Self {
        NetBSDProductReadout
    }

    fn product(&self) -> Result<String, ReadoutError> {
        let output = Command::new("sysctl")
            .args(["-n", "-b", "machdep.dmi.system-version"])
            .output()
            .expect("ERROR: failed to start \"sysctl\" process");

        let sysver = String::from_utf8(output.stdout)
            .expect("ERROR: \"sysctl\" process stdout was not valid UTF-8");

        Ok(sysver)
    }

    fn vendor(&self) -> Result<String, ReadoutError> {
        let output = Command::new("sysctl")
            .args(["-n", "-b", "machdep.dmi.system-vendor"])
            .output()
            .expect("ERROR: failed to start \"sysctl\" process");

        let sysven = String::from_utf8(output.stdout)
            .expect("ERROR: \"sysctl\" process stdout was not valid UTF-8");

        Ok(sysven)
    }

    fn family(&self) -> Result<String, ReadoutError> {
        let output = Command::new("sysctl")
            .args(["-n", "-b", "machdep.dmi.system-product"])
            .output()
            .expect("ERROR: failed to start \"sysctl\" process");

        let sysprod = String::from_utf8(output.stdout)
            .expect("ERROR: \"sysctl\" process stdout was not valid UTF-8");

        Ok(sysprod)
    }
}

impl PackageReadout for NetBSDPackageReadout {
    fn new() -> Self {
        NetBSDPackageReadout
    }

    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        let mut packages = Vec::new();

        if let Some(c) = NetBSDPackageReadout::count_pkgin() {
            packages.push((PackageManager::Pkgsrc, c));
        }

        if let Some(c) = NetBSDPackageReadout::count_cargo() {
            packages.push((PackageManager::Cargo, c));
        }

        packages
    }
}

impl NetBSDPackageReadout {
    fn count_pkgin() -> Option<usize> {
        if let Some(pkg_dbdir) = dirs::pkgdb_dir() {
            if let Ok(read_dir) = read_dir(pkg_dbdir) {
                return Some(read_dir.count() - 1);
            };
        }

        if let Some(localbase_dir) = dirs::localbase_dir() {
            if let Ok(read_dir) = read_dir(localbase_dir.join("pkgdb")) {
                return Some(read_dir.count() - 1);
            }
        }

        None
    }

    fn count_cargo() -> Option<usize> {
        shared::count_cargo()
    }
}

impl NetworkReadout for NetBSDNetworkReadout {
    fn new() -> Self {
        NetBSDNetworkReadout
    }

    fn tx_bytes(&self, _: Option<&str>) -> Result<usize, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn tx_packets(&self, _: Option<&str>) -> Result<usize, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn rx_bytes(&self, _: Option<&str>) -> Result<usize, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn rx_packets(&self, _: Option<&str>) -> Result<usize, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn logical_address(&self, interface: Option<&str>) -> Result<String, ReadoutError> {
        shared::logical_address(interface)
    }

    fn physical_address(&self, _: Option<&str>) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }
}
