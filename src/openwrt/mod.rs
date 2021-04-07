use crate::extra;
use crate::traits::*;
use itertools::Itertools;
use std::fs;
use std::fs::read_dir;
use std::path::Path;
use std::process::{Command, Stdio};
use sysctl::{Ctl, Sysctl};

pub struct OpenWrtBatteryReadout;

pub struct OpenWrtKernelReadout {
    os_release_ctl: Option<Ctl>,
    os_type_ctl: Option<Ctl>,
}

pub struct OpenWrtGeneralReadout {
    hostname_ctl: Option<Ctl>,
}

pub struct OpenWrtMemoryReadout;

pub struct OpenWrtProductReadout;

pub struct OpenWrtPackageReadout;

impl BatteryReadout for OpenWrtBatteryReadout {
    fn new() -> Self {
        OpenWrtBatteryReadout
    }

    fn percentage(&self) -> Result<u8, ReadoutError> {
        unimplemented!();
    }

    fn status(&self) -> Result<BatteryState, ReadoutError> {
        unimplemented!();
    }
}

impl KernelReadout for OpenWrtKernelReadout {
    fn new() -> Self {
        OpenWrtKernelReadout
    }

    fn os_release(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }

    fn os_type(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }
}

impl GeneralReadout for OpenWrtGeneralReadout {
    fn new() -> Self {
        OpenWrtGeneralReadout
    }

    fn machine(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }

    fn local_ip(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }

    fn username(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }

    fn hostname(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }

    fn distribution(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }

    fn desktop_environment(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }

    fn window_manager(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }

    fn terminal(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }

    fn shell(&self, format: ShellFormat) -> Result<String, ReadoutError> {
        unimplemented!();
    }

    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        unimplemented!();
    }
}

impl MemoryReadout for OpenWrtMemoryReadout {
    fn new() -> Self {
        OpenWrtMemoryReadout
    }

    fn total(&self) -> Result<u64, ReadoutError> {
        unimplemented!();
    }

    fn free(&self) -> Result<u64, ReadoutError> {
        unimplemented!();
    }

    fn buffers(&self) -> Result<u64, ReadoutError> {
        unimplemented!();
    }

    fn cached(&self) -> Result<u64, ReadoutError> {
        unimplemented!();
    }

    fn reclaimable(&self) -> Result<u64, ReadoutError> {
        unimplemented!();
    }

    fn used(&self) -> Result<u64, ReadoutError> {
        unimplemented!();
    }
}

impl ProductReadout for OpenWrtProductReadout {
    fn new() -> Self {
        OpenWrtProductReadout
    }

    fn version(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }

    fn vendor(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }

    fn family(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }

    fn name(&self) -> Result<String, ReadoutError> {
        unimplemented!();
    }
}

impl PackageReadout for OpenWrtPackageReadout {
    fn new() -> Self {
        OpenWrtPackageReadout
    }

    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        let mut packages = Vec::new();
        // Instead of having a condition for each distribution.
        // we will try and extract package count by checking
        // if a certain package manager is installed
        if extra::which("opkg") {
            match OpenWrtPackageReadout::count_opkg() {
                Some(c) => packages.push((PackageManager::Opkg, c)),
                _ => (),
            }
        }

        if extra::which("cargo") {
            match OpenWrtPackageReadout::count_cargo() {
                Some(c) => packages.push((PackageManager::Cargo, c)),
                _ => (),
            }
        }

        packages
    }
}

impl OpenWrtPackageReadout {
    /// Returns the number of installed packages for systems
    /// that utilize `opkg` as their package manager. \
    /// Including but not limited to:
    /// - [Openwrt](https://openwrt.org)
    fn count_opkg() -> Option<usize> {
        let opkg_count = Command::new("opkg")
            .arg("list-installed")
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to start \"opkg\" process")
            .stdout
            .expect("ERROR: failed to open \"opkg\" stdout");

        let count = Command::new("wc")
            .arg("-l")
            .stdin(Stdio::from(opkg_count))
            .stdout(Stdio::piped())
            .spawn()
            .expect("ERROR: failed to start \"wc\" process");

        let final_output = count
            .wait_with_output()
            .expect("ERROR: failed to wait for \"wc\" process to exit");

        String::from_utf8(final_output.stdout)
            .expect("ERROR: \"opkg list-installed | wc -l\" output was not valid UTF-8")
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
