use crate::traits::KernelReadout;
use crate::enums::ReadoutError;
use sysctl::Ctl;
use sysctl::Sysctl;
use sysctl::SysctlError;

pub struct LinuxKernelReadout {
    os_release_ctl: Option<Ctl>,
    os_type_ctl: Option<Ctl>,
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
