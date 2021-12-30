use crate::enums::ReadoutError;
use crate::linux::ffi;
use crate::shared;
use crate::traits::ProcessorReadout;
use std::fs;
use std::io::{BufRead, BufReader};

pub struct LinuxProcessorReadout {
    sysinfo: ffi::Sysinfo,
}

impl ProcessorReadout for LinuxProcessorReadout {
    fn new() -> Self {
        LinuxProcessorReadout {
            sysinfo: ffi::Sysinfo::new(),
        }
    }
    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
        Ok(shared::cpu_model_name())
    }

    fn cpu_usage(&self) -> Result<usize, ReadoutError> {
        let mut info = self.sysinfo;
        let info_ptr: *mut ffi::Sysinfo = &mut info;
        let ret = unsafe { ffi::sysinfo(info_ptr) };

        if ret != -1 {
            let f_load = 1f64 / (1 << libc::SI_LOAD_SHIFT) as f64;
            let cpu_usage = info.loads[0] as f64 * f_load;
            let cpu_usage_u =
                (cpu_usage / self.cpu_cores().unwrap() as f64 * 100.0).round() as usize;
            return Ok(cpu_usage_u as usize);
        }

        Err(ReadoutError::Other(
            "Something went wrong during the initialization of the sysinfo struct.".to_string(),
        ))
    }

    fn cpu_physical_cores(&self) -> Result<usize, ReadoutError> {
        use std::io::{BufRead, BufReader};
        if let Ok(content) = fs::File::open("/proc/cpuinfo") {
            let reader = BufReader::new(content);
            for line in reader.lines().flatten() {
                if line.to_lowercase().starts_with("cpu cores") {
                    return Ok(line
                        .split(':')
                        .nth(1)
                        .unwrap()
                        .trim()
                        .parse::<usize>()
                        .unwrap());
                }
            }
        }

        Err(ReadoutError::MetricNotAvailable)
    }

    fn cpu_cores(&self) -> Result<usize, ReadoutError> {
        Ok(unsafe { libc::sysconf(libc::_SC_NPROCESSORS_CONF) } as usize)
    }
}
