use crate::traits::*;
use std::collections::HashMap;
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;
use wmi::WMIResult;
use wmi::{COMLibrary, Variant, WMIConnection};

use windows::{
    core::{PCWSTR, PSTR},
    Win32::Graphics::Dxgi::{CreateDXGIFactory, IDXGIFactory},
    Win32::Graphics::Gdi::{EnumDisplayDevicesW, DISPLAY_DEVICEW},
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
        Err(ReadoutError::NotImplemented)
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

    fn disk_space(&self) -> Result<(u128, u128), ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn gpus(&self) -> Result<Vec<String>, ReadoutError> {
        // Convert bytes to a string
        fn bytes_to_string(value: usize) -> String {
            if value / (1024 * 1024 * 1024) > 0 {
                // Gigabytes
                format!(
                    "{} GB",
                    ((value * 100) / (1024 * 1024 * 1024)) as f64 / 100.0,
                )
            } else if value / (1024 * 1024) > 0 {
                // Megabytes
                format!("{} MB", ((value * 100) / (1024 * 1024)) as f64 / 100.0,)
            } else if value / 1024 > 0 {
                // Kilobytes
                format!("{} KB", ((value * 100) / 1024) as f64 / 100.0,)
            } else {
                "".to_string()
            }
        }

        // Convert memory values to a human-readable string
        fn memory_to_string(
            dedicated_video_memory: usize,
            dedicated_system_memory: usize,
            shared_system_memory: usize,
        ) -> String {
            match (
                dedicated_video_memory,
                dedicated_system_memory,
                shared_system_memory,
            ) {
                (0, 0, 0) => "".to_string(),
                (0, 0, _) => format!(" ({} Shared)", bytes_to_string(shared_system_memory)),
                (0, _, 0) => {
                    format!(" ({} Dedicated)", bytes_to_string(dedicated_system_memory))
                }
                (0, _, _) => format!(
                    " ({} Dedicated, {} Shared)",
                    bytes_to_string(dedicated_system_memory),
                    bytes_to_string(shared_system_memory)
                ),
                (_, 0, 0) => {
                    format!(" ({} Dedicated)", bytes_to_string(dedicated_video_memory))
                }
                (_, 0, _) => format!(
                    " ({} Dedicated, {} Shared)",
                    bytes_to_string(dedicated_video_memory),
                    bytes_to_string(shared_system_memory)
                ),
                (_, _, 0) => format!(
                    " ({} Dedicated, {} Dedicated)",
                    bytes_to_string(dedicated_video_memory),
                    bytes_to_string(dedicated_system_memory)
                ),
                (_, _, _) => format!(
                    " ({} Dedicated, {} Shared)",
                    bytes_to_string(dedicated_video_memory + dedicated_system_memory),
                    bytes_to_string(shared_system_memory)
                ),
            }
        }

        // Sources:
        // https://github.com/Carterpersall/OxiFetch/blob/main/src/main.rs#L360
        // https://github.com/lptstr/winfetch/pull/155

        // Create the Vector to store each GPU's name.
        let mut output: Vec<String> = Vec::new();

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        // Open the location where some DirectX information is stored
        if let Ok(dx_key) = hklm.open_subkey("SOFTWARE\\Microsoft\\DirectX\\") {
            // Get the parent key's LastSeen value
            if let Ok(lastseen) = dx_key.get_value::<u64, _>("LastSeen") {
                // Iterate over the parent key's subkeys and find the ones with the same LastSeen value
                for key in dx_key.enum_keys() {
                    if key.is_err() {
                        continue;
                    }
                    let key = key.unwrap();

                    let sublastseen = match dx_key.open_subkey(&key) {
                        Ok(key) => match key.get_value::<u64, _>("LastSeen") {
                            Ok(key) => key,
                            Err(_) => continue,
                        },
                        Err(_) => continue,
                    };

                    if sublastseen == lastseen {
                        // Get the GPU's name
                        let name = match dx_key.open_subkey(&key) {
                            Ok(key) => match key.get_value::<String, _>("Description") {
                                Ok(key) => key,
                                Err(_) => continue,
                            },
                            Err(_) => continue,
                        };

                        // Get the GPU's video memory
                        let dedicated_video_memory = match dx_key.open_subkey(&key) {
                            Ok(key) => match key.get_value::<u64, _>("DedicatedVideoMemory") {
                                Ok(key) => key as usize,
                                Err(_) => continue,
                            },
                            Err(_) => continue,
                        };
                        let dedicated_system_memory = match dx_key.open_subkey(&key) {
                            Ok(key) => match key.get_value::<u64, _>("DedicatedSystemMemory") {
                                Ok(key) => key as usize,
                                Err(_) => continue,
                            },
                            Err(_) => continue,
                        };
                        let shared_system_memory = match dx_key.open_subkey(&key) {
                            Ok(key) => match key.get_value::<u64, _>("SharedSystemMemory") {
                                Ok(key) => key as usize,
                                Err(_) => continue,
                            },
                            Err(_) => continue,
                        };

                        let memory = memory_to_string(
                            dedicated_video_memory,
                            dedicated_system_memory,
                            shared_system_memory,
                        );

                        // Exclude the Microsoft Basic Render Driver
                        if name == "Microsoft Basic Render Driver" {
                            continue;
                        }

                        // Add the GPU's name to the output vector
                        output.push(name + &memory);
                    }
                }
            };
        };

        // Some systems have a DirectX key that lacks a LastSeen value, so a backup method is needed.
        if !output.is_empty() {
            return Ok(output);
        }

        // Backup Implementation 1: Get GPUs by getting every display device

        let mut devices = Vec::new();
        let mut index = 0;
        let mut status = true;
        // Iterate over EnumDisplayDevicesW until it returns false
        while status {
            devices.push(DISPLAY_DEVICEW::default());
            devices[index].cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
            unsafe {
                status = EnumDisplayDevicesW(PCWSTR::null(), index as u32, &mut devices[index], 0)
                    .as_bool();
            };
            index += 1;
        }
        // Remove the last element, which will be invalid
        devices.pop();

        // Iterate over each device
        for device in devices {
            // Convert [u16; 128] to a String and add to the HashSet
            match (
                String::from_utf16(&device.DeviceString),
                String::from_utf16(&device.DeviceKey),
            ) {
                (Ok(gpu), Ok(key)) => {
                    // Check if the key ends with "\0000", which is the first entry for that GPU
                    if key.trim_matches(char::from(0)).ends_with("\\0000") {
                        output.push(gpu.trim_matches(char::from(0)).to_string());
                    }
                }
                (_, _) => continue,
            }
        }

        if !output.is_empty() {
            // Convert the HashSet to a Vec and return it
            return Ok(output);
        }

        // Backup Implementation 2: Get GPUs using DXGI
        // Sources:
        // https://github.com/SHAREVOX/sharevox_core/blob/297c6c75ea9c6a88ee9002a7848592f7a97b4f9a/crates/voicevox_core/src/publish.rs#L529
        // https://github.com/LinusDierheimer/fastfetch/blob/b3da6b0e89c0decb9ea648e1d98a75fa6ac40225/src/detection/gpu/gpu_windows.cpp#L91

        // Create a DXGI Factory
        let mut factory = unsafe { CreateDXGIFactory::<IDXGIFactory>() };

        if factory.is_ok() {
            // Get the GPU names
            let mut index = 0;
            loop {
                // Get the adapter at the current index
                let adapter = match unsafe { factory.as_mut().unwrap().EnumAdapters(index) } {
                    Ok(adapter) => adapter,
                    Err(_) => break,
                };

                // Get the adapter's information
                let adapter_info = match unsafe { adapter.GetDesc() } {
                    Ok(info) => info,
                    Err(_) => break,
                };

                // Get the name of the video adapter

                if let Ok(description) = String::from_utf16(&adapter_info.Description) {
                    if description.contains("Microsoft Basic Render Driver") {
                        index += 1;
                        continue;
                    }

                    // GPU Video Memory
                    let dedicated_video_memory = adapter_info.DedicatedVideoMemory;
                    // System RAM not available to the CPU
                    let dedicated_system_memory = adapter_info.DedicatedSystemMemory;
                    // System RAM available to both the CPU and GPU
                    let shared_system_memory = adapter_info.SharedSystemMemory;

                    let memory = memory_to_string(
                        dedicated_video_memory,
                        dedicated_system_memory,
                        shared_system_memory,
                    );

                    output.push(format!("{}{}", description.trim_end_matches('\0'), memory));
                }

                index += 1;
            }
        }

        if !output.is_empty() {
            return Ok(output);
        }

        // Backup Implementation 3: Use WMI to query Win32_VideoController

        // Create a WMI connection
        let wmi_con = wmi_connection()?;

        // Query the WMI connection
        let results: Vec<HashMap<String, Variant>> =
            wmi_con.raw_query("SELECT Name FROM Win32_VideoController")?;

        // Get each GPU's name
        for result in results {
            if let Some(Variant::String(gpu)) = result.get("Name") {
                output.push(gpu.to_string());
            }
        }

        if !output.is_empty() {
            return Ok(output);
        }

        Err(ReadoutError::Other("Failed to find any GPUs.".to_string()))
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
