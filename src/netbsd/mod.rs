use crate::extra;
use crate::traits::*;
use itertools::Itertools;
use nix::unistd;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct NetBSDBatteryReadout;

pub struct NetBSDKernelReadout;

pub struct NetBSDGeneralReadout;

pub struct NetBSDMemoryReadout;

pub struct NetBSDProductReadout;

pub struct NetBSDPackageReadout;

impl BatteryReadout for NetBSDBatteryReadout {
    fn new() -> Self {
        NetBSDBatteryReadout
    }

    fn percentage(&self) -> Result<u8, ReadoutError> {
        if extra::which("envstat") {
            let envstat = Command::new("envstat")
                .args(&["-s", "acpibat0:charge"])
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
                            .replace(" ", "")
                            .replace("%", "");
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
                .args(&["-s", "acpibat0:charging"])
                .stdout(Stdio::piped())
                .output()
                .expect("ERROR: failed to spawn \"envstat\" process");

            let envstat_out = String::from_utf8(envstat.stdout)
                .expect("ERROR: \"envstat\" process stdout was not valid UTF-8");

            if envstat_out.is_empty() {
                return Err(ReadoutError::MetricNotAvailable);
            } else {
                if envstat_out.contains("TRUE") {
                    return Ok(BatteryState::Charging);
                } else {
                    return Ok(BatteryState::Discharging);
                }
            }
        }

        Err(ReadoutError::Other(format!("envstat is not installed")))
    }
}

impl KernelReadout for NetBSDKernelReadout {
    fn new() -> Self {
        NetBSDKernelReadout
    }

    fn os_release(&self) -> Result<String, ReadoutError> {
        let output = Command::new("sysctl")
            .args(&["-n", "-b", "kern.osrelease"])
            .output()
            .expect("ERROR: failed to fetch \"kernel.osrelease\" using \"sysctl\"");

        let osrelease = String::from_utf8(output.stdout)
            .expect("ERROR: \"sysctl\" process stdout was not valid UTF-8");

        Ok(String::from(osrelease))
    }

    fn os_type(&self) -> Result<String, ReadoutError> {
        let output = Command::new("sysctl")
            .args(&["-n", "-b", "kern.ostype"])
            .output()
            .expect("ERROR: failed to fetch \"kernel.ostype\" using \"sysctl\"");

        let osrelease = String::from_utf8(output.stdout)
            .expect("ERROR: \"sysctl\" process stdout was not valid UTF-8");

        Ok(String::from(osrelease))
    }

    fn pretty_kernel(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }
}

impl GeneralReadout for NetBSDGeneralReadout {
    fn new() -> Self {
        NetBSDGeneralReadout
    }

    fn resolution(&self) -> Result<String, ReadoutError> {
        use std::os::raw::c_char;
        use x11_ffi::*;

        let display_name: *const c_char = std::ptr::null_mut();
        let display = unsafe { XOpenDisplay(display_name) };

        if cfg!(feature = "xserver") && !display.is_null() {
            let screen = unsafe { XDefaultScreen(display) };
            let width = unsafe { XDisplayWidth(display, screen) };
            let height = unsafe { XDisplayHeight(display, screen) };

            unsafe { XCloseDisplay(display) };
            unsafe {
                libc::free(display_name as *mut libc::c_void);
            }

            return Ok(format!("{}x{}", width, height));
        } else {
            let drm = std::path::Path::new("/sys/class/drm");
            if drm.is_dir() {
                let dirs = extra::list_dir_entries(drm);
                let mut resolution = String::new();
                for entry in dirs {
                    if entry.read_link().is_ok() {
                        let modes = std::path::PathBuf::from(entry).join("modes");
                        if modes.is_file() {
                            if let Ok(mut this_res) = std::fs::read_to_string(modes) {
                                if !this_res.is_empty() {
                                    if this_res.ends_with("\n") {
                                        this_res.pop();
                                    }
                                    resolution.push_str(&this_res);
                                    resolution.push_str(", ");
                                }
                            }
                        }
                    }
                }
                if resolution.trim_end().ends_with(",") {
                    resolution.pop();
                }

                return Ok(resolution);
            }
        }

        Err(ReadoutError::Other(String::from(
            "Could not obtain screen resolution.",
        )))
    }

    fn machine(&self) -> Result<String, ReadoutError> {
        let product_readout = NetBSDProductReadout::new();

        let product = product_readout.product()?;
        let vendor = product_readout.vendor()?;
        let version = product_readout.version()?;

        let product =
            format!("{} {} {}", vendor, product, version).replace("To be filled by O.E.M.", "");

        let new_product: Vec<_> = product.split_whitespace().into_iter().unique().collect();

        if version == product && version == vendor {
            return Ok(vendor);
        }

        Ok(new_product.into_iter().join(" "))
    }

    fn local_ip(&self) -> Result<String, ReadoutError> {
        crate::shared::local_ip()
    }

    fn username(&self) -> Result<String, ReadoutError> {
        crate::shared::username()
    }

    fn hostname(&self) -> Result<String, ReadoutError> {
        let mut buf = [0u8; 64];
        let hostname_cstr = unistd::gethostname(&mut buf);
        match hostname_cstr {
            Ok(hostname_cstr) => {
                let hostname = hostname_cstr.to_str().unwrap_or("Unknown");
                return Ok(String::from(hostname));
            }
            Err(_e) => Err(ReadoutError::Other(String::from(
                "Failed to retrieve hostname from 'gethostname'.",
            ))),
        }
    }

    fn distribution(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::Warning(String::from(
            "Since you're on NetBSD, there is no distribution to be read from the system.",
        )))
    }

    fn desktop_environment(&self) -> Result<String, ReadoutError> {
        crate::shared::desktop_environment()
    }

    fn window_manager(&self) -> Result<String, ReadoutError> {
        crate::shared::window_manager()
    }

    fn terminal(&self) -> Result<String, ReadoutError> {
        // Fetching terminal information is a three step process:
        // 1. Get the shell PID, i.e. the PPID of this program
        // 2. Get the PPID of the shell, i.e. the controlling terminal
        // 3. Get the command associated with the shell's PPID

        // 1. Get the shell PID, i.e. the PPID of this program.
        // This function is always successful.
        fn get_shell_pid() -> i32 {
            let pid = unsafe { libc::getppid() };
            return pid;
        }

        // 2. Get the PPID of the shell, i.e. the cotrolling terminal.
        fn get_shell_ppid(ppid: i32) -> i32 {
            let process_path = PathBuf::from("/proc").join(ppid.to_string()).join("status");
            if let Ok(content) = fs::read_to_string(process_path) {
                let ppid = content.split_whitespace().nth(2);
                if let Some(val) = ppid {
                    if let Ok(c) = val.parse::<i32>() {
                        return c;
                    }
                } else {
                    return -1;
                }
            }

            -1
        }

        // 3. Get the command associated with the shell's PPID.
        fn terminal_name() -> String {
            let terminal_pid = get_shell_ppid(get_shell_pid());
            if terminal_pid != -1 {
                let path = PathBuf::from("/proc")
                    .join(terminal_pid.to_string())
                    .join("status");

                if let Ok(content) = fs::read_to_string(path) {
                    let terminal = content.split_whitespace().next();
                    if let Some(val) = terminal {
                        return String::from(val);
                    }
                }
            }

            String::new()
        }

        let terminal = terminal_name();

        if !terminal.is_empty() {
            return Ok(terminal);
        } else {
            return Err(ReadoutError::Other(format!("Failed to get terminal name")));
        }
    }

    fn shell(&self, shorthand: ShellFormat) -> Result<String, ReadoutError> {
        crate::shared::shell(shorthand)
    }

    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
        Ok(crate::shared::cpu_model_name())
    }

    fn cpu_cores(&self) -> Result<usize, ReadoutError> {
        crate::shared::cpu_cores()
    }

    fn cpu_physical_cores(&self) -> Result<usize, ReadoutError> {
        crate::shared::cpu_physical_cores()
    }

    fn cpu_usage(&self) -> Result<usize, ReadoutError> {
        crate::shared::cpu_usage()
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        crate::shared::uptime()
    }

    fn os_name(&self) -> Result<String, ReadoutError> {
        let kernel_readout = NetBSDKernelReadout::new();

        let os_type = kernel_readout.os_type()?;
        let os_release = kernel_readout.os_release()?;

        if !(os_type.is_empty() || os_release.is_empty()) {
            return Ok(format!("{} {}", os_type, os_release));
        }

        Err(ReadoutError::MetricNotAvailable)
    }
}

impl MemoryReadout for NetBSDMemoryReadout {
    fn new() -> Self {
        NetBSDMemoryReadout
    }

    fn total(&self) -> Result<u64, ReadoutError> {
        Ok(crate::shared::get_meminfo_value("MemTotal"))
    }

    fn free(&self) -> Result<u64, ReadoutError> {
        Ok(crate::shared::get_meminfo_value("MemFree"))
    }

    fn used(&self) -> Result<u64, ReadoutError> {
        let total = self.total().unwrap();
        let free = self.free().unwrap();

        Ok(total - free)
    }
}

impl ProductReadout for NetBSDProductReadout {
    fn new() -> Self {
        NetBSDProductReadout
    }

    fn version(&self) -> Result<String, ReadoutError> {
        let output = Command::new("sysctl")
            .args(&["-n", "-b", "machdep.dmi.system-version"])
            .output()
            .expect("ERROR: failed to start \"sysctl\" process");

        let sysver = String::from_utf8(output.stdout)
            .expect("ERROR: \"sysctl\" process stdout was not valid UTF-8");

        Ok(String::from(sysver))
    }

    fn vendor(&self) -> Result<String, ReadoutError> {
        let output = Command::new("sysctl")
            .args(&["-n", "-b", "machdep.dmi.system-vendor"])
            .output()
            .expect("ERROR: failed to start \"sysctl\" process");

        let sysven = String::from_utf8(output.stdout)
            .expect("ERROR: \"sysctl\" process stdout was not valid UTF-8");

        Ok(String::from(sysven))
    }

    fn product(&self) -> Result<String, ReadoutError> {
        let output = Command::new("sysctl")
            .args(&["-n", "-b", "machdep.dmi.system-product"])
            .output()
            .expect("ERROR: failed to start \"sysctl\" process");

        let sysprod = String::from_utf8(output.stdout)
            .expect("ERROR: \"sysctl\" process stdout was not valid UTF-8");

        Ok(String::from(sysprod))
    }
}

impl PackageReadout for NetBSDPackageReadout {
    fn new() -> Self {
        NetBSDPackageReadout
    }

    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        let mut packages = Vec::new();
        // Instead of having a condition for each distribution.
        // we will try and extract package count by checking
        // if a certain package manager is installed
        if extra::which("pkgin") {
            match NetBSDPackageReadout::count_pkgin() {
                Some(c) => packages.push((PackageManager::Pkgsrc, c)),
                _ => (),
            }
        }

        if extra::which("cargo") {
            match NetBSDPackageReadout::count_cargo() {
                Some(c) => packages.push((PackageManager::Cargo, c)),
                _ => (),
            }
        }

        packages
    }
}

impl NetBSDPackageReadout {
    fn count_pkgin() -> Option<usize> {
        let pkg_info = Command::new("pkg_info")
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        extra::count_lines(
            String::from_utf8(pkg_info.stdout)
                .expect("ERROR: \"pkg_info\" output was not valid UTF-8"),
        )
    }

    fn count_cargo() -> Option<usize> {
        crate::shared::count_cargo()
    }
}
