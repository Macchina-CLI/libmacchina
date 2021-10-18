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
        Err(ReadoutError::MetricNotAvailable)
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
        Err(ReadoutError::MetricNotAvailable)
    }

    fn window_manager(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn terminal(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn shell(&self, shorthand: ShellFormat, kind: ShellKind) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn cpu_cores(&self) -> Result<usize, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn cpu_physical_cores(&self) -> Result<usize, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn cpu_usage(&self) -> Result<usize, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        Err(ReadoutError::MetricNotAvailable)
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
