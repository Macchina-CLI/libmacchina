use crate::extra;
use crate::shared;
use crate::traits::*;
use byte_unit::AdjustedByte;
use std::fs;
use std::path::PathBuf;
use sysctl::{Ctl, Sysctl};

impl From<sqlite::Error> for ReadoutError {
    fn from(e: sqlite::Error) -> Self {
        ReadoutError::Other(e.to_string())
    }
}

pub struct FreeBSDBatteryReadout;

pub struct FreeBSDKernelReadout {
    os_release_ctl: Option<Ctl>,
    os_type_ctl: Option<Ctl>,
}

pub struct FreeBSDGeneralReadout {
    hostname_ctl: Option<Ctl>,
    hwmodel_ctl: Option<Ctl>,
}

pub struct FreeBSDMemoryReadout {
    // available memory
    physmem_ctl: Option<Ctl>,
    // used memory
    usermem_ctl: Option<Ctl>,
}

pub struct FreeBSDProductReadout;

pub struct FreeBSDPackageReadout;

impl BatteryReadout for FreeBSDBatteryReadout {
    fn new() -> Self {
        FreeBSDBatteryReadout
    }

    fn percentage(&self) -> Result<u8, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn status(&self) -> Result<BatteryState, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }
}

impl KernelReadout for FreeBSDKernelReadout {
    fn new() -> Self {
        FreeBSDKernelReadout {
            os_release_ctl: Ctl::new("kernel.osrelease").ok(),
            os_type_ctl: Ctl::new("kernel.ostype").ok(),
        }
    }

    fn os_release(&self) -> Result<String, ReadoutError> {
        Ok(self
            .os_release_ctl
            .as_ref()
            .ok_or(ReadoutError::MetricNotAvailable)?
            .value_string()
            .unwrap())
    }

    fn os_type(&self) -> Result<String, ReadoutError> {
        Ok(self
            .os_type_ctl
            .as_ref()
            .ok_or(ReadoutError::MetricNotAvailable)?
            .value_string()
            .unwrap())
    }

    fn pretty_kernel(&self) -> Result<String, ReadoutError> {
        Ok(format!("{} {}", self.os_type()?, self.os_release()?))
    }
}

impl GeneralReadout for FreeBSDGeneralReadout {
    fn new() -> Self {
        FreeBSDGeneralReadout {
            hostname_ctl: Ctl::new("kernel.hostname").ok(),
            hwmodel_ctl: Ctl::new("hw.model").ok(),
        }
    }

    fn resolution(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn backlight(&self) -> Result<usize, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn machine(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn local_ip(&self) -> Result<String, ReadoutError> {
        shared::local_ip()
    }

    fn username(&self) -> Result<String, ReadoutError> {
        shared::username()
    }

    fn hostname(&self) -> Result<String, ReadoutError> {
        Ok(self
            .hostname_ctl
            .as_ref()
            .ok_or(ReadoutError::MetricNotAvailable)?
            .value_string()
            .unwrap())
    }

    fn distribution(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn desktop_environment(&self) -> Result<String, ReadoutError> {
        shared::desktop_environment()
    }

    fn window_manager(&self) -> Result<String, ReadoutError> {
        shared::window_manager()
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

            // Any command_name we find that matches
            // one of the elements within this table
            // is effectively ignored
            let shells = [
                "sh", "su", "nu", "bash", "fish", "dash", "tcsh", "zsh", "ksh", "csh",
            ];

            // The below loop will traverse /proc to find the
            // terminal inside of which the user is operating
            if let Ok(mut terminal_name) = fs::read_to_string(path) {
                terminal_name = terminal_name.split_whitespace().next().unwrap().to_owned();
                while shells.contains(&terminal_name.as_str()) {
                    let ppid = get_parent(terminal_pid);
                    terminal_pid = ppid;

                    let path = PathBuf::from("/proc")
                        .join(terminal_pid.to_string())
                        .join("status");

                    if let Ok(status) = fs::read_to_string(path) {
                        if let Some(name) = status.split_whitespace().nth(0) {
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
            return Err(ReadoutError::Other(format!("Could not to fetch terminal.")));
        }

        Ok(terminal)
    }

    fn shell(&self, shorthand: ShellFormat, kind: ShellKind) -> Result<String, ReadoutError> {
        shared::shell(shorthand, kind)
    }

    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
        Ok(self
            .hwmodel_ctl
            .as_ref()
            .ok_or(ReadoutError::MetricNotAvailable)?
            .value_string()
            .unwrap())
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
        Err(ReadoutError::MetricNotAvailable)
    }

    fn os_name(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn disk_space(&self) -> Result<(AdjustedByte, AdjustedByte), ReadoutError> {
        shared::disk_space(String::from("/"))
    }
}

impl MemoryReadout for FreeBSDMemoryReadout {
    fn new() -> Self {
        FreeBSDMemoryReadout {
            physmem_ctl: Ctl::new("hw.physmem").ok(),
            usermem_ctl: Ctl::new("hw.usermem").ok(),
        }
    }

    fn total(&self) -> Result<u64, ReadoutError> {
        if let Ok(ctl) = self.physmem {
            if let Ok(sysctl::CtlValue::Long(val)) = ctl.value() {
                return val as u64;
            }
        }

        Err(ReadoutError::Warning(String::from(
            "Couldn't query hw.physmem",
        )))
    }

    fn free(&self) -> Result<u64, ReadoutError> {
        if let Ok(ctl) = self.usermem {
            if let Ok(sysctl::CtlValue::Long(val)) = ctl.value() {
                return val as u64;
            }
        }

        Err(ReadoutError::Warning(String::from(
            "Couldn't query hw.usermem",
        )))
    }

    fn used(&self) -> Result<u64, ReadoutError> {
        let total = self.total().unwrap();
        let free = self.free().unwrap();

        Ok(total - free)
    }
}

impl ProductReadout for FreeBSDProductReadout {
    fn new() -> Self {
        FreeBSDProductReadout
    }

    fn version(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn vendor(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn product(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }
}

impl PackageReadout for FreeBSDPackageReadout {
    fn new() -> Self {
        FreeBSDPackageReadout
    }

    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        let mut packages = Vec::new();

        if extra::which("pkg") {
            if let Some(c) = FreeBSDPackageReadout::count_pkg() {
                packages.push((PackageManager::Pkg, c));
            }
        }

        packages
    }
}

impl FreeBSDPackageReadout {
    fn count_pkg() -> Option<usize> {
        let path = "/var/db/pkg/local.sqlite";
        let connection = sqlite::open(path);
        if let Ok(con) = connection {
            let statement = con.prepare("select count(*) from db.packages");
            if let Ok(mut s) = statement {
                if s.next().is_ok() {
                    return match s.read::<Option<i64>>(0) {
                        Ok(Some(count)) => Some(count as usize),
                        _ => None,
                    };
                }
            }
        }

        None
    }
}
