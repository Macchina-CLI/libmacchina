use std::os::raw::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sysinfo {
    pub uptime: c_long,
    pub loads: [c_ulong; 3],
    pub totalram: c_ulong,
    pub freeram: c_ulong,
    pub sharedram: c_ulong,
    pub bufferram: c_ulong,
    pub totalswap: c_ulong,
    pub freeswap: c_ulong,
    pub procs: c_ushort,
    pub pad: c_ushort,
    pub totalhigh: c_ulong,
    pub freehigh: c_ulong,
    pub mem_unit: c_uint,
    pub _f: [c_char; 20 - 2 * std::mem::size_of::<c_long>() - std::mem::size_of::<c_int>()],
}

extern "C" {
    pub fn sysinfo(info: *mut system_info) -> c_int;
}

impl system_info {
    pub fn new() -> Self {
        sysinfo {
            uptime: 0,
            loads: [0; 3],
            totalram: 0,
            freeram: 0,
            sharedram: 0,
            bufferram: 0,
            totalswap: 0,
            freeswap: 0,
            procs: 0,
            pad: 0,
            totalhigh: 0,
            freehigh: 0,
            mem_unit: 0,
            _f: [0; 20 - 2 * std::mem::size_of::<c_long>() - std::mem::size_of::<c_int>()],
        }
    }
}
