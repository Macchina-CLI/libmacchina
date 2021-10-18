use std::fs;
use std::path::PathBuf;
use crate::shared;
use crate::traits::*;
use byte_unit::AdjustedByte;
use sysctl::{Ctl, Sysctl};

pub struct FreeBSDBatteryReadout;

pub struct FreeBSDKernelReadout;

pub struct FreeBSDGeneralReadout {
    hostname_ctl: Option<Ctl>,
}

pub struct FreeBSDMemoryReadout;

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
        FreeBSDKernelReadout
    }

    fn os_release(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn os_type(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn pretty_kernel(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }
}

impl GeneralReadout for FreeBSDGeneralReadout {
    fn new() -> Self {
        FreeBSDGeneralReadout {
            hostname_ctl: Ctl::new("kernel.hostname").ok(),
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
        Err(ReadoutError::MetricNotAvailable)
    }

    fn hostname(&self) -> Result<String, ReadoutError> {
        Ok(
            self
            .hostname_ctl
            .as_ref()
            .ok_or(ReadoutError::MetricNotAvailable)?
            .value_string().unwrap())
    }

    fn distribution(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn desktop_environment(&self) -> Result<String, ReadoutError> {
        shared::desktop_environment()
    }

    fn window_manager(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
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
        Err(ReadoutError::MetricNotAvailable)
    }

    fn disk_space(&self) -> Result<(AdjustedByte, AdjustedByte), ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }
}

impl MemoryReadout for FreeBSDMemoryReadout {
    fn new() -> Self {
        FreeBSDMemoryReadout
    }

    fn total(&self) -> Result<u64, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn free(&self) -> Result<u64, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn used(&self) -> Result<u64, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
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
}