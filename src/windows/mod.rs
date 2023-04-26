use crate::traits::*;
use std::collections::HashMap;
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;
use wmi::WMIResult;
use wmi::{COMLibrary, Variant, WMIConnection};

use windows::{
    core::{PCWSTR, PSTR},
    Win32::Foundation::{BOOL, LPARAM, RECT},
    Win32::Graphics::Gdi::{
        EnumDisplayDevicesW, EnumDisplayMonitors, EnumDisplaySettingsW, GetMonitorInfoW, DEVMODEW,
        DISPLAY_DEVICEW, DISPLAY_DEVICE_ACTIVE, ENUM_CURRENT_SETTINGS, HDC, HMONITOR, MONITORINFO,
    },
    Win32::System::Power::GetSystemPowerStatus,
    Win32::System::Power::SYSTEM_POWER_STATUS,
    Win32::System::SystemInformation::GetComputerNameExA,
    Win32::System::SystemInformation::GetTickCount64,
    Win32::System::SystemInformation::GlobalMemoryStatusEx,
    Win32::System::SystemInformation::MEMORYSTATUSEX,
    Win32::System::WindowsProgramming::GetUserNameA,
};

impl From<wmi::WMIError> for ReadoutError {
    fn from(e: wmi::WMIError) -> Self {
        ReadoutError::Other(e.to_string())
    }
}

pub struct WindowsBatteryReadout;

impl BatteryReadout for WindowsBatteryReadout {
    fn new() -> Self {
        WindowsBatteryReadout {}
    }

    fn percentage(&self) -> Result<u8, ReadoutError> {
        let power_state = WindowsBatteryReadout::get_power_status()?;

        match power_state.BatteryLifePercent {
            s if s != 255 => Ok(s),
            s => Err(ReadoutError::Warning(format!(
                "Windows reported a battery percentage of {s}, which means there is \
                no battery available. Are you on a desktop system?"
            ))),
        }
    }

    fn status(&self) -> Result<BatteryState, ReadoutError> {
        let power_state = WindowsBatteryReadout::get_power_status()?;

        match power_state.ACLineStatus {
            0 => Ok(BatteryState::Discharging),
            1 => Ok(BatteryState::Charging),
            a => Err(ReadoutError::Other(format!(
                "Unexpected value for ac_line_status from win32 api: {a}"
            ))),
        }
    }

    fn health(&self) -> Result<u64, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }
}

impl WindowsBatteryReadout {
    fn get_power_status() -> Result<SYSTEM_POWER_STATUS, ReadoutError> {
        let mut power_state = SYSTEM_POWER_STATUS::default();

        if unsafe { GetSystemPowerStatus(&mut power_state) }.as_bool() {
            return Ok(power_state);
        }

        Err(ReadoutError::Other(String::from(
            "Call to GetSystemPowerStatus failed.",
        )))
    }
}

pub struct WindowsKernelReadout;

impl KernelReadout for WindowsKernelReadout {
    fn new() -> Self {
        WindowsKernelReadout {}
    }

    fn os_release(&self) -> Result<String, ReadoutError> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let current_windows_not =
            hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")?;

        let nt_build: String = current_windows_not.get_value("CurrentBuild")?;

        Ok(nt_build)
    }

    fn os_type(&self) -> Result<String, ReadoutError> {
        Ok(String::from("Windows NT"))
    }

    fn pretty_kernel(&self) -> Result<String, ReadoutError> {
        Ok(format!("{} {}", self.os_type()?, self.os_release()?))
    }
}

pub struct WindowsMemoryReadout;

impl MemoryReadout for WindowsMemoryReadout {
    fn new() -> Self {
        WindowsMemoryReadout {}
    }

    fn total(&self) -> Result<u64, ReadoutError> {
        let memory_status = WindowsMemoryReadout::get_memory_status()?;
        Ok(memory_status.ullTotalPhys / 1024u64)
    }

    fn free(&self) -> Result<u64, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn buffers(&self) -> Result<u64, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn cached(&self) -> Result<u64, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn reclaimable(&self) -> Result<u64, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn used(&self) -> Result<u64, ReadoutError> {
        let memory_status = WindowsMemoryReadout::get_memory_status()?;
        Ok((memory_status.ullTotalPhys - memory_status.ullAvailPhys) / 1024u64)
    }
}

impl WindowsMemoryReadout {
    fn get_memory_status() -> Result<MEMORYSTATUSEX, ReadoutError> {
        let mut memory_status = MEMORYSTATUSEX::default();
        memory_status.dwLength = std::mem::size_of_val(&memory_status) as u32;

        if !unsafe { GlobalMemoryStatusEx(&mut memory_status) }.as_bool() {
            return Err(ReadoutError::Other(String::from(
                "GlobalMemoryStatusEx returned a zero \
            return \
            code.",
            )));
        }

        Ok(memory_status)
    }
}

thread_local! {
    static COM_LIB: COMLibrary = COMLibrary::new().unwrap();
}

fn wmi_connection() -> WMIResult<WMIConnection> {
    let com_lib = COM_LIB.with(|com| *com);
    WMIConnection::new(com_lib)
}

pub struct WindowsGeneralReadout;

impl GeneralReadout for WindowsGeneralReadout {
    fn new() -> Self {
        WindowsGeneralReadout
    }

    fn backlight(&self) -> Result<usize, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn resolution(&self) -> Result<String, ReadoutError> {
        // Sources:
        // https://github.com/lptstr/winfetch/pull/156/
        // https://patriksvensson.se/posts/2020/06/enumerating-monitors-in-rust-using-win32-api
        // https://github.com/CarterLi/fastfetch/blob/e5f851dcbb94de35c34fb8c5e3dd8300bb56a1cc/src/detection/displayserver/displayserver_windows.c

        // Struct to store each monitor's resolution and refresh rate in
        struct MonitorInfo {
            x_resolution: u32,
            y_resolution: u32,
            refresh_rate: u32,
        }

        // Create a vector to store each monitor's information in
        let mut resolutions = Vec::<MonitorInfo>::new();

        let mut display_device = DISPLAY_DEVICEW {
            cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };
        // The index of the current display device
        let mut devnum = 0;

        // Iterate over every display device
        while unsafe {
            EnumDisplayDevicesW(PCWSTR::null(), devnum, &mut display_device, 0).as_bool()
        } {
            // Skip inactive devices
            if display_device.StateFlags & DISPLAY_DEVICE_ACTIVE == 0 {
                devnum += 1;
                continue;
            }

            // Create a DEVMODEW struct to store the current settings for the device in
            let mut devmode = DEVMODEW {
                dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                ..Default::default()
            };

            // Get the current settings for the device
            if !unsafe {
                EnumDisplaySettingsW(
                    PCWSTR::from_raw(display_device.DeviceName.as_ptr()),
                    ENUM_CURRENT_SETTINGS,
                    &mut devmode,
                )
                .as_bool()
            } {
                devnum += 1;
                continue;
            }

            // Add the resolution and refresh rate to the vector
            resolutions.push(MonitorInfo {
                x_resolution: devmode.dmPelsWidth,
                y_resolution: devmode.dmPelsHeight,
                refresh_rate: devmode.dmDisplayFrequency,
            });

            // Increment the device number to move on to the next device
            devnum += 1;
        }

        if !resolutions.is_empty() {
            // Create callback function for EnumDisplayMonitors to use
            #[allow(non_snake_case, unused_variables)]
            extern "system" fn EnumProc(
                hMonitor: HMONITOR,
                hdcMonitor: HDC,
                lprcMonitor: *mut RECT,
                dwData: LPARAM,
            ) -> BOOL {
                unsafe {
                    // Get the userdata where we will store the result
                    let monitors: &mut Vec<MONITORINFO> = std::mem::transmute(dwData);

                    // Initialize the MONITORINFO structure and get a pointer to it
                    let mut monitor_info: MONITORINFO = std::mem::zeroed();
                    monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                    let monitor_info_ptr = <*mut _>::cast(&mut monitor_info);

                    // Call the GetMonitorInfoW Win32 API
                    let result = GetMonitorInfoW(hMonitor, monitor_info_ptr);
                    if result.as_bool() {
                        // Push the information we received to the vector
                        monitors.push(monitor_info);
                    }
                }

                true.into()
            }

            // Get the scaled resolution of each monitor

            // Create a vector to store the scaled resolutions in
            let mut scaled_res = Vec::<MONITORINFO>::new();
            // Create a pointer to the vector
            let userdata = &mut scaled_res as *mut _;

            // Enumerate over every monitor
            let result = unsafe {
                EnumDisplayMonitors(
                    HDC(0),
                    std::ptr::null(),
                    Some(EnumProc),
                    LPARAM(userdata as isize),
                )
            }
            .as_bool();

            // Check if the number of resolutions and monitors match
            if !result || resolutions.len() != scaled_res.len() {
                return Ok(resolutions
                    .iter()
                    .map(|resolution| {
                        if resolution.refresh_rate == 0 {
                            format!("{}x{}", resolution.x_resolution, resolution.y_resolution)
                        } else {
                            format!(
                                "{}x{}@{}Hz",
                                resolution.x_resolution,
                                resolution.y_resolution,
                                resolution.refresh_rate
                            )
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(", "));
            }

            let mut index = 0;
            // Combine the resolutions and scaled resolutions into a single string
            return Ok(resolutions
                .iter()
                .map(|resolution| {
                    let result = match (
                        resolution.refresh_rate,
                        (scaled_res[index].rcMonitor.bottom - scaled_res[index].rcMonitor.top)
                            as u32
                            == resolution.y_resolution
                            && (scaled_res[index].rcMonitor.right
                                - scaled_res[index].rcMonitor.left)
                                as u32
                                == resolution.x_resolution,
                    ) {
                        (0, true) => {
                            format!("{}x{}", resolution.x_resolution, resolution.y_resolution)
                        }
                        (0, false) => format!(
                            "{}x{} (as {}x{})",
                            resolution.x_resolution,
                            resolution.y_resolution,
                            scaled_res[index].rcMonitor.right - scaled_res[index].rcMonitor.left,
                            scaled_res[index].rcMonitor.bottom - scaled_res[index].rcMonitor.top
                        ),
                        (_, true) => format!(
                            "{}x{}@{}Hz",
                            resolution.x_resolution,
                            resolution.y_resolution,
                            resolution.refresh_rate
                        ),
                        (_, false) => format!(
                            "{}x{}@{}Hz (as {}x{})",
                            resolution.x_resolution,
                            resolution.y_resolution,
                            resolution.refresh_rate,
                            scaled_res[index].rcMonitor.right - scaled_res[index].rcMonitor.left,
                            scaled_res[index].rcMonitor.bottom - scaled_res[index].rcMonitor.top
                        ),
                    };

                    index += 1;

                    result
                })
                .collect::<Vec<String>>()
                .join(", "));
        }

        // If every implementation failed
        Err(ReadoutError::Other(
            "Failed to get display information".to_string(),
        ))
    }

    fn username(&self) -> Result<String, ReadoutError> {
        let mut size = 0;
        unsafe { GetUserNameA(PSTR(std::ptr::null_mut()), &mut size) };

        if size == 0 {
            return Err(ReadoutError::Other(
                "Call to \"GetUserNameA\" failed.".to_string(),
            ));
        }

        let mut username = Vec::with_capacity(size as usize);
        if !unsafe { GetUserNameA(PSTR(username.as_mut_ptr()), &mut size) }.as_bool() {
            return Err(ReadoutError::Other(
                "Call to \"GetUserNameA\" failed.".to_string(),
            ));
        }

        unsafe {
            username.set_len(size as usize);
        }

        let mut str = match String::from_utf8(username) {
            Ok(str) => str,
            Err(e) => {
                return Err(ReadoutError::Other(format!(
                    "String from \"GetUserNameA\" \
            was not valid UTF-8: {e}"
                )))
            }
        };

        str.pop(); //remove null terminator from string.

        Ok(str)
    }

    fn hostname(&self) -> Result<String, ReadoutError> {
        use windows::Win32::System::SystemInformation::ComputerNameDnsHostname;

        let mut size = 0;
        unsafe {
            GetComputerNameExA(
                ComputerNameDnsHostname,
                PSTR(std::ptr::null_mut()),
                &mut size,
            )
        };

        if size == 0 {
            return Err(ReadoutError::Other(String::from(
                "Call to \"GetComputerNameExA\" failed.",
            )));
        }

        let mut hostname = Vec::with_capacity(size as usize);
        if unsafe {
            GetComputerNameExA(
                ComputerNameDnsHostname,
                PSTR(hostname.as_mut_ptr()),
                &mut size,
            )
        } == false
        {
            return Err(ReadoutError::Other(String::from(
                "Call to \"GetComputerNameExA\" failed.",
            )));
        }

        unsafe { hostname.set_len(size as usize) };

        let str = match String::from_utf8(hostname) {
            Ok(str) => str,
            Err(e) => {
                return Err(ReadoutError::Other(format!(
                    "String from \"GetComputerNameExA\" \
            was not valid UTF-8: {e}"
                )))
            }
        };

        Ok(str)
    }

    fn distribution(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn desktop_environment(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn session(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn window_manager(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn terminal(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn shell(&self, _shorthand: ShellFormat, _: ShellKind) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let central_processor =
            hklm.open_subkey("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0")?;

        let processor_name: String = central_processor.get_value("ProcessorNameString")?;

        Ok(processor_name)
    }

    fn cpu_usage(&self) -> Result<usize, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn cpu_physical_cores(&self) -> Result<usize, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn cpu_cores(&self) -> Result<usize, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        let tick_count = unsafe { GetTickCount64() };
        let duration = std::time::Duration::from_millis(tick_count);

        Ok(duration.as_secs() as usize)
    }

    fn machine(&self) -> Result<String, ReadoutError> {
        let product_readout = WindowsProductReadout::new();

        Ok(format!(
            "{} {}",
            product_readout.vendor()?,
            product_readout.product()?
        ))
    }

    fn os_name(&self) -> Result<String, ReadoutError> {
        let wmi_con = wmi_connection()?;

        let results: Vec<HashMap<String, Variant>> =
            wmi_con.raw_query("SELECT Caption FROM Win32_OperatingSystem")?;

        if let Some(os) = results.first() {
            if let Some(Variant::String(caption)) = os.get("Caption") {
                return Ok(caption.to_string());
            }
        }

        Err(ReadoutError::Other(
            "Trying to get the operating system name \
            from WMI failed"
                .to_string(),
        ))
    }

    fn disk_space(&self) -> Result<(u64, u64), ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn gpus(&self) -> Result<Vec<String>, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }
}

pub struct WindowsProductReadout {
    manufacturer: Option<String>,
    model: Option<String>,
}

impl ProductReadout for WindowsProductReadout {
    fn new() -> Self {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let sys_info = hklm
            .open_subkey("SYSTEM\\CurrentControlSet\\Control\\SystemInformation")
            .unwrap();

        WindowsProductReadout {
            manufacturer: sys_info.get_value("SystemManufacturer").ok(),
            model: sys_info.get_value("SystemProductName").ok(),
        }
    }

    fn vendor(&self) -> Result<String, ReadoutError> {
        match &self.manufacturer {
            Some(v) => Ok(v.clone()),
            None => Err(ReadoutError::Other(
                "Trying to get the system manufacturer \
                from the registry failed"
                    .to_string(),
            )),
        }
    }

    fn family(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn product(&self) -> Result<String, ReadoutError> {
        match &self.model {
            Some(v) => Ok(v.clone()),
            None => Err(ReadoutError::Other(
                "Trying to get the system product name \
                from the registry failed"
                    .to_string(),
            )),
        }
    }
}

pub struct WindowsPackageReadout;

impl PackageReadout for WindowsPackageReadout {
    fn new() -> Self {
        WindowsPackageReadout {}
    }

    /// Returns the __number of installed packages__ for the following package managers:
    /// - cargo
    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        let mut packages = Vec::new();
        if let Some(c) = WindowsPackageReadout::count_cargo() {
            packages.push((PackageManager::Cargo, c));
        }
        if let Some(c) = WindowsPackageReadout::count_scoop() {
            packages.push((PackageManager::Scoop, c));
        }
        packages
    }
}

impl WindowsPackageReadout {
    fn count_cargo() -> Option<usize> {
        crate::shared::count_cargo()
    }

    fn count_scoop() -> Option<usize> {
        let scoop = match std::env::var("SCOOP") {
            Ok(scoop_var) => PathBuf::from(scoop_var),
            _ => home::home_dir().unwrap().join("scoop"),
        };
        match scoop.join("apps").read_dir() {
            Ok(dir) => Some(dir.count() - 1), // One entry belongs to scoop itself
            _ => None,
        }
    }
}

pub struct WindowsNetworkReadout;

impl NetworkReadout for WindowsNetworkReadout {
    fn new() -> Self {
        WindowsNetworkReadout
    }

    fn tx_bytes(&self, _: Option<&str>) -> Result<usize, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn tx_packets(&self, _: Option<&str>) -> Result<usize, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn rx_bytes(&self, _: Option<&str>) -> Result<usize, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn rx_packets(&self, _: Option<&str>) -> Result<usize, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn logical_address(&self, interface: Option<&str>) -> Result<String, ReadoutError> {
        match interface {
            Some(it) => {
                if let Ok(addresses) = local_ip_address::list_afinet_netifas() {
                    if let Some((_, ip)) = local_ip_address::find_ifa(addresses, it) {
                        return Ok(ip.to_string());
                    }
                }
            }
            None => {
                if let Ok(local_ip) = local_ip_address::local_ip() {
                    return Ok(local_ip.to_string());
                }
            }
        };

        Err(ReadoutError::Other(
            "Unable to get local IP address.".to_string(),
        ))
    }

    fn physical_address(&self, _: Option<&str>) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }
}
