#![allow(clippy::unnecessary_cast)]
use crate::extra;
use crate::macos::mach_ffi::{io_registry_entry_t, DisplayServicesGetBrightness, IOObjectRelease};
use crate::macos::mach_ffi::{
    kIOMasterPortDefault, vm_statistics64, IORegistryEntryCreateCFProperties,
    IOServiceGetMatchingService, IOServiceMatching,
};
use crate::shared;
use crate::traits::ReadoutError::MetricNotAvailable;
use crate::traits::*;
use core_foundation::base::{TCFType, ToVoid};
use core_foundation::dictionary::{CFMutableDictionary, CFMutableDictionaryRef};
use core_foundation::number::{CFNumber, CFNumberRef};
use core_foundation::string::CFString;
use core_graphics::display::{CGDisplay, CGMainDisplayID};
use core_video_sys::{
    kCVTimeIsIndefinite, CVDisplayLinkCreateWithCGDisplay,
    CVDisplayLinkGetNominalOutputVideoRefreshPeriod, CVDisplayLinkRef, CVDisplayLinkRelease,
};
use mach2::kern_return::KERN_SUCCESS;
use std::ffi::CString;
use std::fs::DirEntry;
use std::path::Path;
use sysctl::{Ctl, Sysctl};

mod mach_ffi;

pub struct MacOSBatteryReadout {
    power_info: Result<MacOSIOPMPowerSource, ReadoutError>,
}

pub struct MacOSProductReadout {
    hw_model_ctl: Option<Ctl>,
}

pub struct MacOSKernelReadout {
    os_type_ctl: Option<Ctl>,
    os_release_ctl: Option<Ctl>,
}

pub struct MacOSGeneralReadout {
    cpu_brand_ctl: Option<Ctl>,
    boot_time_ctl: Option<Ctl>,
    hostname_ctl: Option<Ctl>,
    os_product_version_ctl: Option<Ctl>,
}

pub struct MacOSMemoryReadout {
    page_size: i64,
    physical_memory: i64,
}

#[derive(Debug, Default)]
struct MacOSIOPMPowerSource {
    battery_installed: Option<bool>,
    state_of_charge: Option<usize>,
    charging: Option<bool>,
}

pub struct MacOSPackageReadout;

pub struct MacOSNetworkReadout;

impl BatteryReadout for MacOSBatteryReadout {
    fn new() -> Self {
        MacOSBatteryReadout {
            power_info: MacOSIOPMPowerSource::new(),
        }
    }

    fn percentage(&self) -> Result<u8, ReadoutError> {
        let power_info = self.power_info.as_ref()?;

        Ok(power_info
            .state_of_charge
            .ok_or_else(|| ReadoutError::Other(String::from(
                "Percentage property was not present in the dictionary that was returned from IOKit.",
            )))? as u8)
    }

    fn status(&self) -> Result<BatteryState, ReadoutError> {
        let power_info = self.power_info.as_ref()?;

        if let Some(charging) = power_info.charging {
            return Ok(if charging {
                BatteryState::Charging
            } else {
                BatteryState::Discharging
            });
        }

        Err(ReadoutError::Other(String::from(
            "Status property was not present in the dictionary that was returned from IOKit.",
        )))
    }

    fn health(&self) -> Result<u64, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }
}

impl MacOSIOPMPowerSource {
    fn new() -> Result<Self, ReadoutError> {
        let battery_data_key = CFString::new("BatteryData");
        let power_source_dict = MacOSIOPMPowerSource::get_power_source_dict()?;

        if !power_source_dict.contains_key(battery_data_key.to_void()) {
            return Err(ReadoutError::Other(String::from("Dictionary does not contain information about the battery. Are you using a third-party battery?")));
        }

        let battery_data_dict =
            (*power_source_dict.get(&battery_data_key.to_void())) as CFMutableDictionaryRef;

        let battery_data_dict: CFMutableDictionary<_> =
            unsafe { CFMutableDictionary::wrap_under_get_rule(battery_data_dict) };

        let mut instance: MacOSIOPMPowerSource = std::default::Default::default();

        unsafe {
            if let Some(battery_installed) =
                power_source_dict.find(&CFString::new("BatteryInstalled").to_void())
            {
                let number = CFNumber::wrap_under_get_rule((*battery_installed) as CFNumberRef);
                instance.battery_installed = Some(number.to_i32() != Some(0));
            }

            if let Some(state_of_charge) =
                battery_data_dict.find(&CFString::new("StateOfCharge").to_void())
            {
                let number = CFNumber::wrap_under_get_rule((*state_of_charge) as CFNumberRef);
                instance.state_of_charge = Some(number.to_i32().unwrap() as usize);
            }

            if let Some(charging) = power_source_dict.find(&CFString::new("IsCharging").to_void()) {
                let number = CFNumber::wrap_under_get_rule((*charging) as CFNumberRef);
                instance.charging = Some(number.to_i32() != Some(0));
            }
        }

        Ok(instance)
    }

    fn get_power_source_dict() -> Result<CFMutableDictionary, ReadoutError> {
        let io_service_name = CString::new("IOPMPowerSource").expect("Unable to create c string");
        let service = unsafe { IOServiceMatching(io_service_name.as_ptr()) };
        let entry: io_registry_entry_t =
            unsafe { IOServiceGetMatchingService(kIOMasterPortDefault, service) };
        let mut dict_data: Option<CFMutableDictionary> = None;

        if entry != 0 {
            let mut dict: CFMutableDictionaryRef = std::ptr::null_mut();
            let dict_ptr = (&mut dict) as *mut CFMutableDictionaryRef;

            let kern_return =
                unsafe { IORegistryEntryCreateCFProperties(entry, dict_ptr, std::ptr::null(), 0) };

            if kern_return == KERN_SUCCESS {
                dict_data = Some(unsafe { CFMutableDictionary::wrap_under_create_rule(dict) });
            }

            unsafe {
                IOObjectRelease(entry);
            }

            if kern_return != KERN_SUCCESS {
                return Err(ReadoutError::Other(format!(
                    "Creating the dictionary for the IOService failed with return code: {kern_return}"
                )));
            }
        }

        dict_data.ok_or_else(|| ReadoutError::Other(String::from(
            "Unable to get the 'IOPMPowerSource' service from IOKit :( Are you on a desktop system?",
        )))
    }
}

impl KernelReadout for MacOSKernelReadout {
    fn new() -> Self {
        MacOSKernelReadout {
            os_type_ctl: Ctl::new("kern.ostype").ok(),
            os_release_ctl: Ctl::new("kern.osrelease").ok(),
        }
    }

    fn os_release(&self) -> Result<String, ReadoutError> {
        Ok(self
            .os_release_ctl
            .as_ref()
            .ok_or(MetricNotAvailable)?
            .value_string()?)
    }

    fn os_type(&self) -> Result<String, ReadoutError> {
        Ok(self
            .os_type_ctl
            .as_ref()
            .ok_or(MetricNotAvailable)?
            .value_string()?)
    }

    fn pretty_kernel(&self) -> Result<String, ReadoutError> {
        Ok(format!("{} {}", self.os_type()?, self.os_release()?))
    }
}

impl GeneralReadout for MacOSGeneralReadout {
    fn new() -> Self {
        MacOSGeneralReadout {
            cpu_brand_ctl: Ctl::new("machdep.cpu.brand_string").ok(),
            boot_time_ctl: Ctl::new("kern.boottime").ok(),
            hostname_ctl: Ctl::new("kern.hostname").ok(),
            os_product_version_ctl: Ctl::new("kern.osproductversion").ok(),
        }
    }

    fn backlight(&self) -> Result<usize, ReadoutError> {
        let main_display = unsafe { CGMainDisplayID() };
        let mut display_brightness: f32 = 0.0;

        let return_value =
            unsafe { DisplayServicesGetBrightness(main_display, &mut display_brightness) };

        if return_value == 0 {
            Ok((display_brightness * 100.0) as usize)
        } else {
            Err(ReadoutError::Other(format!(
                "Could not query display brightness of main display, got return code {return_value}"
            )))
        }
    }

    fn resolution(&self) -> Result<String, ReadoutError> {
        let displays = CGDisplay::active_displays();
        if let Err(e) = displays {
            return Err(ReadoutError::Other(format!(
                "Error while querying active displays: {e}"
            )));
        }

        let displays: Vec<CGDisplay> = displays
            .unwrap()
            .iter()
            .map(|id| CGDisplay::new(*id))
            .filter(|d| d.is_active())
            .collect();

        let mut output: Vec<String> = Vec::with_capacity(displays.len());

        for display in displays {
            let (ui_width, ui_height) = (display.pixels_wide(), display.pixels_high());
            let mut out_string: String = format!("{ui_width}x{ui_height}");

            if let Some(mode) = display.display_mode() {
                let (real_width, real_height) = (mode.pixel_width(), mode.pixel_height());

                let mut refresh_rate: i32 = mode.refresh_rate().round() as i32;
                if refresh_rate == 0 {
                    unsafe {
                        let mut link: CVDisplayLinkRef = std::mem::zeroed();
                        CVDisplayLinkCreateWithCGDisplay(display.id, &mut link);

                        let time = CVDisplayLinkGetNominalOutputVideoRefreshPeriod(link);
                        if (time.flags & kCVTimeIsIndefinite) == 0 {
                            refresh_rate =
                                ((time.timeScale as f64) / (time.timeValue as f64)) as i32;
                        }

                        CVDisplayLinkRelease(link);
                    }
                }

                if real_width != ui_width || real_height != ui_height {
                    out_string = format!(
                        "{real_width}x{real_height}@{refresh_rate}fps (as {ui_width}x{ui_height})"
                    );
                }
            }

            output.push(out_string);
        }

        Ok(output.join("\n"))
    }

    fn username(&self) -> Result<String, ReadoutError> {
        shared::username()
    }

    fn hostname(&self) -> Result<String, ReadoutError> {
        Ok(self
            .hostname_ctl
            .as_ref()
            .ok_or(MetricNotAvailable)?
            .value_string()?)
    }

    fn distribution(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::Warning(String::from(
            "Since you're on macOS, there is no distribution to be read from the system.",
        )))
    }

    fn desktop_environment(&self) -> Result<String, ReadoutError> {
        Ok(String::from("Aqua"))
    }

    fn session(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn window_manager(&self) -> Result<String, ReadoutError> {
        Ok(String::from("Quartz Compositor"))
    }

    fn terminal(&self) -> Result<String, ReadoutError> {
        use std::env::var;

        let mut terminal: Option<String> = None;
        if let Ok(mut terminal_str) = var("TERM_PROGRAM") {
            terminal_str = terminal_str.to_lowercase();
            terminal = match terminal_str.as_str() {
                "iterm.app" => Some(String::from("iTerm2")),
                "apple_terminal" => Some(String::from("Apple Terminal")),
                "hyper" => Some(String::from("HyperTerm")),
                s => Some(String::from(s)),
            }
        }

        if let Some(terminal) = terminal {
            if let Ok(version) = var("TERM_PROGRAM_VERSION") {
                return Ok(format!("{terminal} (Version {version})"));
            }

            return Ok(terminal);
        }

        if let Ok(terminal_env) = var("TERM") {
            return Ok(terminal_env);
        }

        Err(MetricNotAvailable)
    }

    fn shell(&self, shorthand: ShellFormat, kind: ShellKind) -> Result<String, ReadoutError> {
        shared::shell(shorthand, kind)
    }

    fn cpu_model_name(&self) -> Result<String, ReadoutError> {
        Ok(self
            .cpu_brand_ctl
            .as_ref()
            .ok_or(MetricNotAvailable)?
            .value_string()?)
    }

    fn cpu_usage(&self) -> Result<usize, ReadoutError> {
        shared::cpu_usage()
    }

    fn cpu_physical_cores(&self) -> Result<usize, ReadoutError> {
        shared::cpu_physical_cores()
    }

    fn cpu_cores(&self) -> Result<usize, ReadoutError> {
        shared::cpu_cores()
    }

    fn uptime(&self) -> Result<usize, ReadoutError> {
        use libc::timeval;
        use std::time::{Duration, SystemTime, UNIX_EPOCH};

        let time = self
            .boot_time_ctl
            .as_ref()
            .ok_or(MetricNotAvailable)?
            .value_as::<timeval>()?;
        let duration = Duration::new(time.tv_sec as u64, (time.tv_usec * 1000) as u32);
        let bootup_timestamp = UNIX_EPOCH + duration;

        if let Ok(duration) = SystemTime::now().duration_since(bootup_timestamp) {
            let seconds_since_boot = duration.as_secs();
            return Ok(seconds_since_boot as usize);
        }

        Err(ReadoutError::Other(String::from(
            "Error calculating boot time since unix \
            epoch.",
        )))
    }

    fn machine(&self) -> Result<String, ReadoutError> {
        let product_readout = MacOSProductReadout::new();
        product_readout.product()
    }

    fn os_name(&self) -> Result<String, ReadoutError> {
        let version: String = self.operating_system_version()?.into();
        let major_version_name = macos_version_to_name(&self.operating_system_version()?);

        Ok(format!("macOS {version} {major_version_name}"))
    }

    fn disk_space(&self, path: &Path) -> Result<(u64, u64), ReadoutError> {
        shared::disk_space(path)
    }

    fn gpus(&self) -> Result<Vec<String>, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }
}

impl MacOSGeneralReadout {
    fn operating_system_version(&self) -> Result<NSOperatingSystemVersion, ReadoutError> {
        let os_string = self
            .os_product_version_ctl
            .as_ref()
            .ok_or(ReadoutError::MetricNotAvailable)?
            .value_string()?;

        let mut string_parts = os_string.split('.');

        let mut operating_system_version = NSOperatingSystemVersion::default();

        if let Some(major) = string_parts.next() {
            operating_system_version.major_version = major.parse().unwrap_or_default()
        }

        if let Some(minor) = string_parts.next() {
            operating_system_version.minor_version = minor.parse().unwrap_or_default()
        }

        if let Some(patch) = string_parts.next() {
            operating_system_version.patch_version = patch.parse().unwrap_or_default()
        }

        Ok(operating_system_version)
    }
}

impl MemoryReadout for MacOSMemoryReadout {
    fn new() -> Self {
        let page_size = match Ctl::new("hw.pagesize").unwrap().value().unwrap() {
            sysctl::CtlValue::S64(s) => s,
            _ => panic!("Could not get vm page size."),
        };

        let physical_mem = match Ctl::new("hw.memsize").unwrap().value().unwrap() {
            sysctl::CtlValue::S64(s) => s,
            _ => panic!("Could not get physical memory size."),
        };

        MacOSMemoryReadout {
            page_size,
            physical_memory: physical_mem,
        }
    }

    fn total(&self) -> Result<u64, ReadoutError> {
        Ok(self.physical_memory as u64 / 1024)
    }

    fn free(&self) -> Result<u64, ReadoutError> {
        let vm_stats = MacOSMemoryReadout::mach_vm_stats()?;
        let free_count: u64 =
            (vm_stats.free_count + vm_stats.inactive_count - vm_stats.speculative_count) as u64;

        Ok(((free_count * self.page_size as u64) / 1024) as u64)
    }

    fn buffers(&self) -> Result<u64, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn cached(&self) -> Result<u64, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn reclaimable(&self) -> Result<u64, ReadoutError> {
        let vm_stats = MacOSMemoryReadout::mach_vm_stats()?;
        Ok((vm_stats.purgeable_count as u64 * self.page_size as u64 / 1024) as u64)
    }

    fn used(&self) -> Result<u64, ReadoutError> {
        let vm_stats = MacOSMemoryReadout::mach_vm_stats()?;
        let used: u64 = ((vm_stats.active_count + vm_stats.wire_count) as u64
            * self.page_size as u64
            / 1024) as u64;

        Ok(used)
    }
}

impl MacOSMemoryReadout {
    fn mach_vm_stats() -> Result<vm_statistics64, ReadoutError> {
        use mach2::kern_return::KERN_SUCCESS;
        use mach2::message::mach_msg_type_number_t;
        use mach2::vm_types::integer_t;
        use mach_ffi::*;

        const HOST_VM_INFO_COUNT: mach_msg_type_number_t =
            (std::mem::size_of::<vm_statistics64>() / std::mem::size_of::<integer_t>()) as u32;

        const HOST_VM_INFO64: integer_t = 4;

        let mut vm_stat: vm_statistics64 = std::default::Default::default();
        let vm_stat_ptr: *mut vm_statistics64 = &mut vm_stat;
        let mut count: mach_msg_type_number_t = HOST_VM_INFO_COUNT;

        let ret_val = unsafe {
            host_statistics64(
                mach_host_self(),
                HOST_VM_INFO64,
                vm_stat_ptr as *mut integer_t,
                &mut count as *mut mach_msg_type_number_t,
            )
        };

        if ret_val == KERN_SUCCESS {
            return Ok(vm_stat);
        }

        Err(ReadoutError::Other(String::from(
            "Could not retrieve vm statistics from host.",
        )))
    }
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
struct NSOperatingSystemVersion {
    major_version: u64,
    minor_version: u64,
    patch_version: u64,
}

impl From<NSOperatingSystemVersion> for String {
    fn from(s: NSOperatingSystemVersion) -> String {
        format!(
            "{}.{}.{}",
            s.major_version, s.minor_version, s.patch_version
        )
    }
}

impl ProductReadout for MacOSProductReadout {
    fn new() -> Self {
        MacOSProductReadout {
            hw_model_ctl: Ctl::new("hw.model").ok(),
        }
    }

    fn vendor(&self) -> Result<String, ReadoutError> {
        Ok(String::from("Apple"))
    }

    fn family(&self) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }

    fn product(&self) -> Result<String, ReadoutError> {
        let mac_model = self
            .hw_model_ctl
            .as_ref()
            .ok_or(MetricNotAvailable)?
            .value_string()?;

        Ok(mac_model)
    }
}

impl PackageReadout for MacOSPackageReadout {
    fn new() -> Self {
        MacOSPackageReadout
    }

    fn count_pkgs(&self) -> Vec<(PackageManager, usize)> {
        let mut packages = Vec::new();
        if extra::which("brew") {
            if let Some(c) = MacOSPackageReadout::count_homebrew() {
                packages.push((PackageManager::Homebrew, c))
            }
        }

        if let Some(c) = MacOSPackageReadout::count_cargo() {
            packages.push((PackageManager::Cargo, c))
        }

        packages
    }
}

impl MacOSPackageReadout {
    /// This method returns the total entries of `/usr/local/Cellar` and `/usr/local/Caskroom` directories
    /// which contain all installed packages of the Homebrew package manager.
    /// A manual call via `homebrew list` would be too expensive, since it is pretty slow.
    fn count_homebrew() -> Option<usize> {
        use std::fs::read_dir;
        use std::path::Path;

        // Homebrew stores packages in /usr/local on older-generation Apple hardware.
        let homebrew_root = Path::new("/usr/local");
        let cellar_folder = homebrew_root.join("Cellar");
        let caskroom_folder = homebrew_root.join("Caskroom");

        let hidden_files_filter = |f: &Result<DirEntry, std::io::Error>| match f {
            Ok(entry) => !entry.file_name().to_str().unwrap().starts_with('.'),
            Err(_) => false,
        };

        let cellar_count = match read_dir(cellar_folder) {
            Ok(read_dir) => read_dir.filter(hidden_files_filter).count(),
            Err(_) => 0,
        };

        let caskroom_count = match read_dir(caskroom_folder) {
            Ok(read_dir) => read_dir.filter(hidden_files_filter).count(),
            Err(_) => 0,
        };

        // Homebrew stores packages in /opt/homebrew on Apple Silicon machines.
        let opt_homebrew_root = Path::new("/opt/homebrew");
        let opt_cellar_folder = opt_homebrew_root.join("Cellar");
        let opt_caskroom_folder = opt_homebrew_root.join("Caskroom");

        let opt_cellar_count = match read_dir(opt_cellar_folder) {
            Ok(read_dir) => read_dir.filter(hidden_files_filter).count(),
            Err(_) => 0,
        };

        let opt_caskroom_count = match read_dir(opt_caskroom_folder) {
            Ok(read_dir) => read_dir.filter(hidden_files_filter).count(),
            Err(_) => 0,
        };

        Some(cellar_count + caskroom_count + opt_cellar_count + opt_caskroom_count)
    }

    fn count_cargo() -> Option<usize> {
        shared::count_cargo()
    }
}

impl NetworkReadout for MacOSNetworkReadout {
    fn new() -> Self {
        MacOSNetworkReadout
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
        shared::logical_address(interface)
    }

    fn physical_address(&self, _: Option<&str>) -> Result<String, ReadoutError> {
        Err(ReadoutError::NotImplemented)
    }
}

fn macos_version_to_name(version: &NSOperatingSystemVersion) -> &'static str {
    match (version.major_version, version.minor_version) {
        (10, 1) => "Puma",
        (10, 2) => "Jaguar",
        (10, 3) => "Panther",
        (10, 4) => "Tiger",
        (10, 5) => "Leopard",
        (10, 6) => "Snow Leopard",
        (10, 7) => "Lion",
        (10, 8) => "Mountain Lion",
        (10, 9) => "Mavericks",
        (10, 10) => "Yosemite",
        (10, 11) => "El Capitan",
        (10, 12) => "Sierra",
        (10, 13) => "High Sierra",
        (10, 14) => "Mojave",
        (10, 15) => "Catalina",
        (11, _) | (10, 16) => "Big Sur",
        (12, _) => "Monterey",
        (13, _) => "Ventura",
        _ => "Unknown",
    }
}
