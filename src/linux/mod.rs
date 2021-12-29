#![allow(unused_imports)]

use crate::traits::BatteryReadout;
use crate::traits::GeneralReadout;
use crate::traits::GraphicalReadout;
use crate::traits::KernelReadout;
use crate::traits::MemoryReadout;
use crate::traits::NetworkReadout;
use crate::traits::PackageReadout;
use crate::traits::ProcessorReadout;
use crate::traits::ProductReadout;
use crate::traits::ReadoutError;
#[cfg(any(feature = "general", feature = "memory"))]
use ffi::sysinfo;
#[cfg(any(feature = "battery", feature = "kernel", feature = "general"))]
use sysctl::SysctlError;

pub mod ffi;
#[cfg(feature = "general")]
pub mod general;
#[cfg(feature = "graphical")]
pub mod graphical;
#[cfg(feature = "kernel")]
pub mod kernel;
#[cfg(feature = "memory")]
pub mod memory;
#[cfg(feature = "network")]
pub mod network;
#[cfg(feature = "package")]
pub mod package;
#[cfg(feature = "processor")]
pub mod processor;
#[cfg(feature = "product")]
pub mod product;
#[cfg(feature = "battery")]
pub mod battery;

#[cfg(any(feature = "battery", feature="kernel", feature="sysctl"))]
impl From<SysctlError> for ReadoutError {
    fn from(e: SysctlError) -> Self {
        ReadoutError::Other(format!("Could not access sysctl: {:?}", e))
    }
}
