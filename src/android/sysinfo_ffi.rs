use std::os::raw::*;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct system_info {
    pub uptime: c_long,
    pub loads: [c_ulong; 3],
    pub totalram: c_ulong,
    pub freeram: c_ulong,
    pub sharedram: c_ulong,
    pub bufferram: c_ulong,
    pub totalswap: c_ulong,
    pub freeswap: c_ulong,
    pub procs: c_ushort,
    pub totalhigh: c_ulong,
    pub freehigh: c_ulong,
    pub mem_unit: c_uint,
    pub _f: [c_char; 0],
}

extern "C" {
    pub fn sysinfo(info: *mut system_info) -> c_int;
}

impl system_info {
    pub fn new() -> Self {
        system_info {
            uptime: 0,
            loads: [0; 3],
            totalram: 0,
            freeram: 0,
            sharedram: 0,
            bufferram: 0,
            totalswap: 0,
            freeswap: 0,
            procs: 0,
            totalhigh: 0,
            freehigh: 0,
            mem_unit: 0,
            _f: [0; 0],
        }
    }
}
