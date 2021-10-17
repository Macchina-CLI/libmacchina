use crate::traits::*;
use byte_unit::AdjustedByte;

pub struct FreeBSDBatteryReadout;

pub struct FreeBSDKernelReadout;

pub struct FreeBSDGeneralReadout;

pub struct FreeBSDMemoryReadout;

pub struct FreeBSDProductReadout;

pub struct FreeBSDPackageReadout;

impl BatteryReadout for FreeBSDBatteryReadout {
    fn new() -> Self {
        FreeBSDBatteryReadout
    }

    fn percentage(&self) -> Result<u8, ReadoutError> {
        todo!()
    }

    fn status(&self) -> Result<BatteryState, ReadoutError> {
        todo!()
    }
}

impl KernelReadout for FreeBSDKernelReadout {
    fn new() -> Self {
        FreeBSDKernelReadout
    }

    fn os_release(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn os_type(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn pretty_kernel(&self) -> Result<String, ReadoutError> {
        todo!()
    }
}

impl GeneralReadout for FreeBSDGeneralReadout {
    fn new() -> Self {
        FreeBSDGeneralReadout
    }

    fn resolution(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn backlight(&self) -> Result<usize, ReadoutError> {
        todo!()
    }

    fn machine(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn local_ip(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn username(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn hostname(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn distribution(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn desktop_environment(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn window_manager(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn terminal(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn shell(&self, shorthand: ShellFormat, kind: ShellKind) -> Result<String, ReadoutError> {
        todo!()
    }

    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn cpu_cores(&self) -> Result<usize, ReadoutError> {
        todo!()
    }

    fn cpu_physical_cores(&self) -> Result<usize, ReadoutError> {
        todo!()
    }

    fn cpu_usage(&self) -> Result<usize, ReadoutError> {
        todo!()
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        todo!()
    }

    fn os_name(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn disk_space(&self) -> Result<(AdjustedByte, AdjustedByte), ReadoutError> {
        todo!()
    }
}

impl MemoryReadout for FreeBSDMemoryReadout {
    fn new() -> Self {
        FreeBSDMemoryReadout
    }

    fn total(&self) -> Result<u64, ReadoutError> {
        todo!()
    }

    fn free(&self) -> Result<u64, ReadoutError> {
        todo!()
    }

    fn used(&self) -> Result<u64, ReadoutError> {
        todo!()
    }
}

impl ProductReadout for FreeBSDProductReadout {
    fn new() -> Self {
        FreeBSDProductReadout
    }

    fn version(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn vendor(&self) -> Result<String, ReadoutError> {
        todo!()
    }

    fn product(&self) -> Result<String, ReadoutError> {
        todo!()
    }
}

impl PackageReadout for FreeBSDPackageReadout {
    fn new() -> Self {
        FreeBSDPackageReadout
    }

    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        todo!()
    }
}
