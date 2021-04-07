use crate::traits::*;
use itertools::Itertools;
use std::fs;
use std::fs::read_dir;
use std::path::Path;
use std::process::{Command, Stdio};
use sysctl::{Ctl, Sysctl};

pub struct OpenwrtBatteryReadout;

pub struct OpenwrtKernelReadout {
    os_release_ctl: Option<Ctl>,
    os_type_ctl: Option<Ctl>,
}

pub struct OpenwrtGeneralReadout {
    hostname_ctl: Option<Ctl>,
}

pub struct OpenwrtMemoryReadout;

pub struct OpenwrtProductReadout;

pub struct OpenwrtPackageReadout;

impl KernelReadout for OpenwrtKernelReadout {
    fn new() -> Self {
        OpenwrtKernelReadout {
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

