#![allow(unused_imports)]

use crate::enums::ReadoutError;
#[cfg(any(feature = "kernel", feature = "general"))]
use sysctl::SysctlError;

#[cfg(feature = "battery")]
pub mod battery;
#[cfg(any(feature = "general", feature = "processor", feature = "memory"))]
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

#[cfg(any(feature = "kernel", feature="general"))]
impl From<SysctlError> for ReadoutError {
    fn from(e: SysctlError) -> Self {
        ReadoutError::Other(format!("Could not access sysctl: {:?}", e))
    }
}
